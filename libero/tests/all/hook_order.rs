//! Every `use_*` call runs on every render, in the same order (todos 2134, 2146).
//! A hook after an early return or inside a branch shifts the slots when the branch flips,
//! and dioxus panics (a wasm `unreachable` in release).

use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::{TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ExprCall, ExprMethodCall, ImplItemFn, ItemFn, Local, Macro, TraitItemFn};

/// Branches that cannot flip on a mounted component, each with why: (file, function, reason).
const FIXED: [(&str, &str, &str); 8] = [
    (
        "src/components/buttons/toolbar.rs",
        "Toolbar",
        "`silent_focus()` is Blitz's alone: fixed per build",
    ),
    (
        "src/hooks/focus_within.rs",
        "use_focus_within",
        "`silent_focus()` is Blitz's alone: fixed per build",
    ),
    (
        "src/hooks/silent_focus.rs",
        "use_silent_focus",
        "`silent_focus()` is Blitz's alone: fixed per build",
    ),
    (
        "src/platform/backend/mod.rs",
        "Outlet",
        "the returns are behind `#[cfg]`",
    ),
    (
        "src/components/form/native_select.rs",
        "NativeSelect",
        "`opens_select_picker()` is a const",
    ),
    (
        "src/components/navigation/bottom_navigation.rs",
        "BottomNavigation",
        "`cfg!(debug_assertions)`",
    ),
    (
        "src/components/navigation/bottom_navigation.rs",
        "BottomNavigationItem",
        "`cfg!(debug_assertions)`",
    ),
    (
        "src/components/form/use_field/prepare.rs",
        "prepare",
        "a field either always passes `bound` or never does",
    ),
];

fn fixed(hit: &str) -> bool {
    FIXED
        .iter()
        .any(|(file, function, _)| hit.starts_with(&format!("{file}: {function}: ")))
}

#[derive(Default)]
struct Audit {
    file: String,
    function: String,
    /// Inside an `if`, `match` arm, loop body or the right side of `&&` / `||`.
    branch: usize,
    /// Inside a closure or async block: runs later, if ever.
    deferred: usize,
    /// A `return`, `?` or `let ... else` came before, at this function's own level.
    returned: bool,
    hits: Vec<String>,
}

impl Audit {
    fn hook(&mut self, name: &str) {
        let place = if self.deferred > 0 {
            "inside a closure"
        } else if self.branch > 0 {
            "inside a branch"
        } else if self.returned {
            "after an early return"
        } else {
            return;
        };
        self.hits
            .push(format!("{}: {}: {name} {place}", self.file, self.function));
    }

    fn function(&mut self, name: String, body: impl FnOnce(&mut Self)) {
        let outer = (
            std::mem::replace(&mut self.function, name),
            std::mem::take(&mut self.branch),
            std::mem::take(&mut self.deferred),
            std::mem::take(&mut self.returned),
        );
        body(self);
        (self.function, self.branch, self.deferred, self.returned) = outer;
    }

    fn branch(&mut self, body: impl FnOnce(&mut Self)) {
        self.branch += 1;
        body(self);
        self.branch -= 1;
    }

    fn deferred(&mut self, body: impl FnOnce(&mut Self)) {
        let returned = self.returned;
        self.deferred += 1;
        body(self);
        self.deferred -= 1;
        self.returned = returned;
    }

    fn argument(&mut self, arg: &Expr) {
        match arg {
            Expr::Closure(closure) => self.deferred(|audit| audit.visit_expr_closure(closure)),
            _ => self.visit_expr(arg),
        }
    }

    /// Macro bodies are tokens to syn: `rsx!` and friends are scanned for `use_x(` by hand.
    fn tokens(&mut self, tokens: TokenStream) {
        // The last `use_x`, kept across a turbofish (`use_context::<T>`).
        let mut hook: Option<String> = None;
        let mut angles = 0usize;
        for tree in tokens {
            match tree {
                TokenTree::Ident(ident) if angles == 0 => {
                    let name = ident.to_string();
                    hook = name.starts_with("use_").then_some(name);
                }
                TokenTree::Group(group) => {
                    if angles == 0
                        && let Some(name) = hook.take()
                    {
                        self.branch(|audit| audit.hook(&name));
                    }
                    self.tokens(group.stream());
                }
                TokenTree::Punct(punct) if hook.is_some() => match punct.as_char() {
                    '<' => angles += 1,
                    '>' => angles = angles.saturating_sub(1),
                    ':' | ',' | '&' | '\'' => {}
                    _ if angles > 0 => {}
                    _ => hook = None,
                },
                TokenTree::Ident(_) => {}
                TokenTree::Punct(_) | TokenTree::Literal(_) => {
                    hook = None;
                    angles = 0;
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for Audit {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.function(node.sig.ident.to_string(), |audit| {
            visit::visit_item_fn(audit, node)
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast ImplItemFn) {
        self.function(node.sig.ident.to_string(), |audit| {
            visit::visit_impl_item_fn(audit, node)
        });
    }

    fn visit_trait_item_fn(&mut self, node: &'ast TraitItemFn) {
        self.function(node.sig.ident.to_string(), |audit| {
            visit::visit_trait_item_fn(audit, node)
        });
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        match node {
            Expr::If(node) => {
                self.visit_expr(&node.cond);
                self.branch(|audit| {
                    audit.visit_block(&node.then_branch);
                    if let Some((_, other)) = &node.else_branch {
                        audit.visit_expr(other);
                    }
                });
            }
            Expr::Match(node) => {
                self.visit_expr(&node.expr);
                self.branch(|audit| node.arms.iter().for_each(|arm| audit.visit_arm(arm)));
            }
            Expr::ForLoop(node) => {
                self.visit_expr(&node.expr);
                self.branch(|audit| audit.visit_block(&node.body));
            }
            Expr::While(_) | Expr::Loop(_) => self.branch(|audit| visit::visit_expr(audit, node)),
            Expr::Binary(binary) if matches!(binary.op, BinOp::And(_) | BinOp::Or(_)) => {
                self.visit_expr(&binary.left);
                self.branch(|audit| audit.visit_expr(&binary.right));
            }
            // A closure may be a test's component, so its body is checked as its own function.
            Expr::Closure(_) => {
                let name = format!("{} closure", self.function);
                self.function(name, |audit| visit::visit_expr(audit, node));
            }
            Expr::Async(_) => self.deferred(|audit| visit::visit_expr(audit, node)),
            Expr::Return(_) | Expr::Try(_) => {
                visit::visit_expr(self, node);
                if self.deferred == 0 {
                    self.returned = true;
                }
            }
            _ => visit::visit_expr(self, node),
        }
    }

    fn visit_local(&mut self, node: &'ast Local) {
        visit::visit_local(self, node);
        if node
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
            && self.deferred == 0
        {
            self.returned = true;
        }
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        let hook = match &*node.func {
            Expr::Path(path) => path
                .path
                .segments
                .last()
                .map(|last| last.ident.to_string())
                .filter(|name| name.starts_with("use_")),
            _ => None,
        };
        self.visit_expr(&node.func);
        // A hook's own closure runs inside that hook, never as a render's next slot.
        if hook.is_some() {
            node.args.iter().for_each(|arg| self.argument(arg));
        } else {
            node.args.iter().for_each(|arg| self.visit_expr(arg));
        }
        if let Some(name) = hook {
            self.hook(&name);
        }
    }

    /// `option.map(|x| ...)`, `flag.then(|| ...)`: the closure runs some of the time.
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        self.visit_expr(&node.receiver);
        node.args.iter().for_each(|arg| self.argument(arg));
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        self.tokens(node.tokens.clone());
    }
}

fn sources(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn audit(name: &str, source: &str) -> Vec<String> {
    let mut audit = Audit {
        file: name.to_owned(),
        ..Audit::default()
    };
    audit.visit_file(&syn::parse_file(source).unwrap());
    audit.hits
}

fn crate_hits() -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    sources(&root.join("src"), &mut files);
    files.sort();
    files
        .iter()
        .flat_map(|path| {
            let name = path.strip_prefix(root).unwrap().display().to_string();
            audit(&name, &fs::read_to_string(path).unwrap())
        })
        .collect()
}

#[test]
fn no_hook_runs_behind_a_branch_or_an_early_return() {
    let hits = crate_hits();
    let open: Vec<&String> = hits.iter().filter(|hit| !fixed(hit)).collect();
    assert!(
        open.is_empty(),
        "call every hook first, unconditionally (or list a per-build branch in `FIXED`):\n{}",
        open.iter()
            .map(|hit| hit.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
    let stale: Vec<_> = FIXED
        .iter()
        .filter(|(file, function, _)| {
            !hits
                .iter()
                .any(|hit| hit.starts_with(&format!("{file}: {function}: ")))
        })
        .collect();
    assert!(stale.is_empty(), "drop from `FIXED`: {stale:?}");
}

/// The 2134 bug, as it was before 981766fa2: the audit has to catch it.
#[test]
fn the_audit_finds_the_glass_gradient_bug() {
    let hits = audit(
        "theme.rs",
        "fn use_glass(glass: bool) -> Option<String> {
            let style = use_gradient_style();
            if !glass { return style; }
            let context = use_context::<LiberoContext>();
            None
        }",
    );
    assert_eq!(
        hits,
        ["theme.rs: use_glass: use_context after an early return"]
    );
}

#[test]
fn the_audit_finds_branches_closures_and_macros() {
    let hits = audit(
        "x.rs",
        "fn a(on: bool) -> Element {
            let ok = use_signal(|| 0);
            let bad = on.then(|| use_memo(|| 1));
            if on { use_effect(|| {}); }
            let Some(x) = y else { return None };
            let late = use_hook(|| use_context::<C>());
            rsx! { if on { {use_context::<C>()} } }
        }
        fn b() {
            let dom = VirtualDom::new(|| { let ok = use_signal(|| 0); rsx! {} });
        }",
    );
    assert_eq!(
        hits,
        [
            "x.rs: a: use_memo inside a closure",
            "x.rs: a: use_effect inside a branch",
            "x.rs: a: use_context inside a closure",
            "x.rs: a: use_hook after an early return",
            "x.rs: a: use_context inside a branch",
        ]
    );
}
