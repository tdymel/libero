//! The docs pages print most of their code from `const NAME: &str = r#"..."#`
//! snippets in `docs/src/pages`, and nothing compiled them. This test writes
//! every such snippet into `tests/page_snippets.md`, wrapped into a component
//! so it can compile on its own, and `src/md_examples.rs` compiles that file
//! as doc-tests. When a page changes a snippet, the file changes with it and
//! this test fails once, so the new file gets committed.
//!
//! Blocks are compiled, not run (`no_run`). A snippet is split at its blank
//! lines, and each piece is placed by its first line of code: items (`fn`, `enum`,
//! `#[derive]`, ...) at the top, `let` statements in the component's body,
//! and anything else in its `rsx!`. Comment lines right above the `const`
//! tell it more:
//!
//! - `// snippet: ignore - <why>` - not compiled: pseudocode, not Rust, or
//!   built on another page's code.
//! - `// snippet: after A, B` - the page's snippets `A` and `B` go first, each
//!   placed by its own shape. Usually the enum the snippet matches on.
//! - `// snippet: item <item>` - one more item, such as a stand-in for the
//!   docs' own icons: `#[component] fn FileIcon() -> Element { rsx! {} }`.
//! - `// snippet: let <statement>` - one more statement in the body.
//! - `// snippet: in <rsx>` - the rsx the snippet goes into, at `..`. A prop
//!   line like `label: |s: Section| ...` needs the component that takes it.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

const PRELUDE: &str = "use dioxus::prelude::*;
use libero::components::*;
use libero::hooks::*;
use libero::platform::*;
use libero::sx::*;
use libero::theme::*;
use libero::{LiberoProvider, use_theme};
// Both globs name a `Title`; the component is the one a snippet means.
use libero::components::Title;
use std::time::Duration;
";

struct Snippet {
    name: String,
    body: String,
    markers: Vec<String>,
}

#[derive(PartialEq)]
enum Shape {
    Items,
    Statements,
    /// A value like `sx().padding("md")`, compiled as `let _ = ..;`.
    Expression,
    /// A component's prop, like `label: |s: Section| ..`: needs `in`.
    Prop,
    Rsx,
}

fn shape(body: &str) -> Shape {
    let first = body
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("//"))
        .unwrap_or("");
    const ITEMS: [&str; 11] = [
        "#[",
        "use ",
        "fn ",
        "pub ",
        "impl ",
        "static ",
        "const ",
        "enum ",
        "struct ",
        "mod ",
        "async fn ",
    ];
    if ITEMS.iter().any(|start| first.starts_with(start)) {
        Shape::Items
    } else if first.starts_with("let ") {
        Shape::Statements
    } else if is_prop(first) {
        Shape::Prop
    } else if first.starts_with(|c: char| c.is_ascii_lowercase())
        && first.contains(['(', '.', '!'])
        && !first.starts_with("if ")
        && !first.starts_with("match ")
        && !first.starts_with("for ")
    {
        Shape::Expression
    } else {
        Shape::Rsx
    }
}

/// A snippet split at its blank lines outside any bracket, so an enum followed
/// by a `let` lands in two places.
fn chunks(body: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let (mut depth, mut start, mut offset) = (0i32, 0, 0);
    for line in body.split_inclusive('\n') {
        if line.trim().is_empty() && depth == 0 {
            found.push(body[start..offset].trim());
            start = offset;
        }
        for c in line.chars() {
            match c {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => depth -= 1,
                _ => {}
            }
        }
        offset += line.len();
    }
    found.push(body[start..].trim());
    found.retain(|chunk| !chunk.is_empty());
    found
}

/// `name: value`, the way a prop is written inside a component's braces.
fn is_prop(line: &str) -> bool {
    let name = line.trim_start_matches("r#");
    let end = name
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(name.len());
    end > 0 && name[end..].starts_with(": ") && !name[end..].starts_with("::")
}

/// Every `const NAME: &str = r#"..."#;` in one file, with the `// snippet:`
/// lines in the comment block right above it.
fn snippets(source: &str) -> Vec<Snippet> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let declaration = line
            .trim_start_matches("pub(super) ")
            .trim_start_matches("pub ");
        let Some(rest) = declaration.strip_prefix("const ") else {
            continue;
        };
        let Some((name, _)) = rest.split_once(": &str =") else {
            continue;
        };
        // The raw string starts on this line or the next.
        let offset: usize = lines[..index].iter().map(|l| l.len() + 1).sum::<usize>()
            + line.find(": &str =").unwrap()
            + ": &str =".len();
        let value = source[offset..].trim_start();
        let Some(value) = value.strip_prefix('r') else {
            continue;
        };
        let hashes = value.len() - value.trim_start_matches('#').len();
        let open = &value[hashes..];
        let Some(open) = open.strip_prefix('"') else {
            continue;
        };
        let close = format!("\"{}", "#".repeat(hashes));
        // A `__NAME__` placeholder is where the page splices in more; the
        // snippet compiles without it.
        let mut body = open[..open.find(&close).unwrap()].to_string();
        while let Some(start) = body.find("__") {
            let Some(len) = body[start + 2..].find("__") else {
                break;
            };
            let name = &body[start + 2..start + 2 + len];
            if name.is_empty() || !name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                break;
            }
            body.replace_range(start..start + 4 + len, "");
        }

        let markers = lines[..index]
            .iter()
            .rev()
            .take_while(|l| l.trim_start().starts_with("//"))
            .filter_map(|l| l.trim().strip_prefix("// snippet: "))
            .map(str::to_string)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        found.push(Snippet {
            name: name.to_string(),
            body,
            markers,
        });
    }
    found
}

/// The snippet as a block that compiles on its own, `Ok(None)` for `ignore`,
/// or what its markers lack.
fn compilable(snippet: &Snippet, page: &[Snippet]) -> Result<Option<String>, String> {
    let (mut items, mut statements, mut rsx) = (String::new(), String::new(), String::new());
    let mut host = String::from("..");
    let mut place = |body: &str| {
        let body = body.trim_end();
        for chunk in chunks(body) {
            let _ = match shape(chunk) {
                Shape::Items => writeln!(items, "{chunk}"),
                Shape::Statements => writeln!(statements, "{chunk}"),
                // On a line of its own: the chunk may end in a `//` comment.
                Shape::Expression => writeln!(statements, "let _ = {chunk}\n;"),
                Shape::Prop | Shape::Rsx => writeln!(rsx, "{chunk}"),
            };
        }
    };
    for marker in &snippet.markers {
        if marker.starts_with("ignore") {
            return Ok(None);
        } else if let Some(names) = marker.strip_prefix("after ") {
            for name in names.split(',').map(str::trim) {
                let before = page
                    .iter()
                    .find(|s| s.name == name)
                    .ok_or(format!("{}: no snippet `{name}`", snippet.name))?;
                place(&before.body);
            }
        } else if let Some(item) = marker.strip_prefix("item ") {
            place(item);
        } else if let Some(statement) = marker.strip_prefix("let ") {
            place(&format!("let {statement}"));
        } else if let Some(template) = marker.strip_prefix("in ") {
            host = template.to_string();
        } else {
            return Err(format!(
                "{}: unknown marker `// snippet: {marker}`",
                snippet.name
            ));
        }
    }
    let shape = shape(&snippet.body);
    if shape == Shape::Prop && host == ".." {
        return Err(format!(
            "{}: a prop needs `// snippet: in <Component {{ .. }}>` or `// snippet: ignore`",
            snippet.name
        ));
    }
    if matches!(shape, Shape::Rsx | Shape::Prop) {
        // One piece of rsx, blank lines and all.
        let _ = writeln!(rsx, "{}", host.replacen("..", snippet.body.trim_end(), 1));
    } else {
        place(&snippet.body);
    }
    Ok(Some(format!(
        "{PRELUDE}\n{items}\n#[component]\nfn Snippet() -> Element {{\n{statements}\nrsx! {{\n{rsx}}}\n}}\n"
    )))
}

fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn page_snippets_are_current() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pages = root.join("../docs/src/pages");
    let mut files = Vec::new();
    rust_files(&pages, &mut files);
    files.sort();

    let mut out = String::from(
        "# Page snippets\n\nGenerated by `libero/tests/all/page_snippets.rs` from the `const` \
         snippets in `docs/src/pages`, and compiled as doc-tests by `libero/src/md_examples.rs`. \
         Do not edit: change the page, then run `cargo test -p libero --test all page_snippets::`.\n",
    );
    let mut problems = Vec::new();
    for file in &files {
        let page = snippets(&std::fs::read_to_string(file).unwrap());
        let path = file.strip_prefix(&pages).unwrap().display().to_string();
        for snippet in &page {
            let _ = write!(out, "\n## {path} `{}`\n\n", snippet.name);
            match compilable(snippet, &page) {
                Ok(Some(code)) => {
                    let _ = write!(out, "```rust,no_run\n{code}```\n");
                }
                Ok(None) => {
                    let _ = write!(out, "```rust,ignore\n{}\n```\n", snippet.body.trim_end());
                }
                Err(problem) => problems.push(format!("{path} {problem}")),
            }
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
    let generated = root.join("tests/page_snippets.md");
    let current = std::fs::read_to_string(&generated).unwrap_or_default();
    if current != out {
        std::fs::write(&generated, &out).unwrap();
        panic!("tests/page_snippets.md was out of date and is now rewritten: commit it");
    }
}
