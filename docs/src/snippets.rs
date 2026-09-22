//! Every piece of Rust the docs pages print (`const` snippets and rendered `Demo` code), written
//! to the gitignored `libero/tests/page_snippets.md`: run this before libero's doc-tests.
//!
//! Each blank-line piece is placed by its first line: items on top, `let` in the body, rsx in
//! `rsx!`. Directives in comment lines right above the `const` or the page's `Demo {`:
//!
//! - `// snippet: ignore - <why>` - not compiled; required for non-Rust `CodeBlock`s.
//! - `// snippet: after A, B` - the page's snippets `A` and `B` go first.
//! - `// snippet: item <item>` - one more item, e.g. a stand-in icon component.
//! - `// snippet: let <statement>` - one more statement in the body.
//! - `// snippet: in <rsx>` - the rsx the snippet goes into, at `..`.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use dioxus::history::{History, MemoryHistory, provide_history_context};
use dioxus::prelude::*;

use crate::Route;
use crate::components::{DemoCode, DemoValues, PropGroup};

const PRELUDE: &str = "use dioxus::prelude::*;
use libero::components::*;
use libero::hooks::*;
use libero::localization::*;
use libero::platform::*;
use libero::sx::*;
use libero::theme::*;
use libero::{LiberoProvider, use_theme};
// Both globs name a `Title`; the component is the one a snippet means.
use libero::components::Title;
use std::time::Duration;
";

thread_local! {
    static DEMOS: RefCell<Vec<DemoCode>> = const { RefCell::new(Vec::new()) };
}

thread_local! {
    static PAGES: RefCell<Vec<(Option<String>, Vec<PropGroup>)>> = const { RefCell::new(Vec::new()) };
}

/// Called by every `DocPage` as it mounts.
pub fn record_page(markdown: Option<String>, properties: Vec<PropGroup>) {
    PAGES.with(|pages| pages.borrow_mut().push((markdown, properties)));
}

/// Called by every `Demo` as it mounts.
pub fn record(code: &DemoCode) {
    DEMOS.with(|demos| demos.borrow_mut().push(code.clone()));
}

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
    /// Not Rust as far as the first line tells: TOML, shell, prose.
    Unknown,
}

/// The first line of code in `body`, comments skipped.
fn first_line(body: &str) -> &str {
    body.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("//"))
        .unwrap_or("")
}

/// The leading identifier or path of `line` (`r#type`, `libero::sx`), and
/// what follows it.
fn split_path(line: &str) -> (&str, &str) {
    let end = line
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '#'))
        .unwrap_or(line.len());
    line.split_at(end)
}

fn shape(body: &str) -> Shape {
    let first = first_line(body);
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
    let (path, rest) = split_path(first);
    let starts_lower = first.starts_with(|c: char| c.is_ascii_lowercase());
    if ITEMS.iter().any(|start| first.starts_with(start)) {
        Shape::Items
    } else if first.starts_with("let ") {
        Shape::Statements
    } else if is_prop(first) {
        Shape::Prop
    } else if ["if ", "for ", "match "]
        .iter()
        .any(|start| first.starts_with(start))
        || first.starts_with(['"', '{'])
        // An element or a component: `div {`, `Button {`, `Button {}`.
        || (!path.is_empty() && rest.trim_start().starts_with('{'))
    {
        Shape::Rsx
    } else if starts_lower && !path.is_empty() && rest.starts_with(['(', '.', '!']) {
        Shape::Expression
    } else {
        Shape::Unknown
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

/// The `// snippet: ` lines in the comment block that ends right above line
/// `index`.
fn markers(lines: &[&str], index: usize) -> Vec<String> {
    lines[..index]
        .iter()
        .rev()
        .take_while(|l| l.trim_start().starts_with("//"))
        .filter_map(|l| l.trim().strip_prefix("// snippet: "))
        .map(str::to_string)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
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

        found.push(Snippet {
            name: name.to_string(),
            body,
            markers: markers(&lines, index),
        });
    }
    found
}

/// The language a `CodeBlock { source: NAME, language: ".." }` in `source`
/// prints the const `name` as, when one names it.
fn printed_language(source: &str, name: &str) -> Option<String> {
    let needle = format!("source: {name}");
    source.match_indices(&needle).find_map(|(at, _)| {
        // The rest of the `CodeBlock`'s props, on one line or several: up to
        // its end.
        let after = &source[at + needle.len()..];
        if after.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_') {
            return None;
        }
        let props = &after[..after.find('}').unwrap_or(after.len())];
        let language = props.split_once("language: \"")?.1;
        Some(language[..language.find('"')?].to_string())
    })
}

/// Why the snippet is not compiled, `None` when it is.
fn ignored(snippet: &Snippet) -> Result<Option<String>, String> {
    let Some(marker) = snippet.markers.iter().find(|m| m.starts_with("ignore")) else {
        return Ok(None);
    };
    match marker.strip_prefix("ignore - ") {
        Some(why) if !why.trim().is_empty() => Ok(Some(why.trim().to_string())),
        _ => Err(format!(
            "{}: `// snippet: ignore` needs a reason: `// snippet: ignore - <why>`",
            snippet.name
        )),
    }
}

/// The snippet as a block that compiles on its own after `PRELUDE`, or what
/// its markers lack. `ignore` is handled by the caller.
fn compilable(snippet: &Snippet, page: &[Snippet]) -> Result<String, String> {
    let (mut items, mut statements, mut rsx) = (String::new(), String::new(), String::new());
    let mut host = String::from("..");
    let mut unknown = None;
    let mut place = |body: &str| {
        let body = body.trim_end();
        for chunk in chunks(body) {
            let _ = match shape(chunk) {
                Shape::Items => writeln!(items, "{chunk}"),
                Shape::Statements => writeln!(statements, "{chunk}"),
                // On a line of its own: the chunk may end in a `//` comment.
                Shape::Expression => writeln!(statements, "let _ = {chunk}\n;"),
                Shape::Prop | Shape::Rsx => writeln!(rsx, "{chunk}"),
                Shape::Unknown => {
                    unknown.get_or_insert(first_line(chunk).to_string());
                    Ok(())
                }
            };
        }
    };
    for marker in &snippet.markers {
        if let Some(names) = marker.strip_prefix("after ") {
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
        } else if !marker.starts_with("ignore") {
            return Err(format!(
                "{}: unknown marker `// snippet: {marker}`",
                snippet.name
            ));
        }
    }
    let whole = shape(&snippet.body);
    if whole == Shape::Prop && host == ".." {
        return Err(format!(
            "{}: a prop needs `// snippet: in <Component {{ .. }}>` or `// snippet: ignore - <why>`",
            snippet.name
        ));
    }
    if matches!(whole, Shape::Rsx | Shape::Prop) {
        // One piece of rsx, blank lines and all - apart from an item, a `let`
        // or a call after a blank line, which is another fragment of the page's.
        let (hoisted, kept): (Vec<&str>, Vec<&str>) =
            chunks(&snippet.body).into_iter().partition(|chunk| {
                matches!(
                    shape(chunk),
                    Shape::Items | Shape::Statements | Shape::Expression
                )
            });
        for chunk in hoisted {
            place(chunk);
        }
        let _ = writeln!(rsx, "{}", host.replacen("..", &kept.join("\n\n"), 1));
    } else {
        place(&snippet.body);
    }
    if let Some(line) = unknown {
        return Err(format!(
            "{}: `{line}` starts no Rust item, statement, expression or rsx. If the snippet \
             is not Rust, mark it `// snippet: ignore - <language>`",
            snippet.name
        ));
    }
    Ok(format!(
        "{items}\n#[component]\nfn Snippet() -> Element {{\n{statements}\nrsx! {{\n{rsx}}}\n}}\n"
    ))
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

/// The site's own `App` at `route`, so the shell gets every context it reads:
/// a copy without the `Rtl` provider made the shell panic and hid every `Demo` (todo 827).
#[component]
fn Page(route: Route) -> Element {
    use_hook(|| {
        provide_history_context(
            Rc::new(MemoryHistory::with_initial_path(route.to_string())) as Rc<dyn History>
        )
    });
    rsx! {
        crate::App {}
    }
}

/// Every `Demo` the route renders, in render order.
fn demos_of(route: Route) -> Vec<DemoCode> {
    DEMOS.with(|demos| demos.borrow_mut().clear());
    let mut dom = VirtualDom::new_with_props(Page, PageProps { route });
    dom.rebuild_in_place();
    DEMOS.with(|demos| std::mem::take(&mut *demos.borrow_mut()))
}

/// The markdown mirror and the property groups of every `DocPage` the route renders.
fn pages_of(route: Route) -> Vec<(Option<String>, Vec<PropGroup>)> {
    PAGES.with(|pages| pages.borrow_mut().clear());
    let mut dom = VirtualDom::new_with_props(Page, PageProps { route });
    dom.rebuild_in_place();
    PAGES.with(|pages| std::mem::take(&mut *pages.borrow_mut()))
}

/// A method's name without its parameters: `show(args)` is `show`.
fn bare(name: &str) -> &str {
    name.split('(').next().unwrap_or(name)
}

/// The names in a mirror's `## Props` and `## API` tables, per `###` group; `""` names the
/// `## Props` ones before any. `## API` files options structs and handles by heading; a handle may also have a `## ` section
/// or a table headed by its type.
fn md_props(md: &str) -> Vec<(String, BTreeSet<String>)> {
    let mut groups: Vec<(String, BTreeSet<String>)> = Vec::new();
    let mut section = "";
    // The group the next row joins: none until a heading (or, in `## Props`, a row) opens one.
    let mut current: Option<usize> = None;
    let open = |groups: &mut Vec<(String, BTreeSet<String>)>, name: &str| {
        groups.push((name.to_string(), BTreeSet::new()));
        Some(groups.len() - 1)
    };
    for line in md.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            section = heading.trim();
            current = None;
            // A handle may have a `## ` section of its own, named by its type.
            if section.trim_matches('`').ends_with("Handle") {
                current = open(&mut groups, section.trim_matches('`'));
            }
        } else if current.is_none() && !matches!(section, "Props" | "API") {
            continue;
        } else if let Some(heading) = line.strip_prefix("### ") {
            if matches!(section, "Props" | "API") {
                current = open(&mut groups, heading.trim().trim_matches('`'));
            }
        } else if let Some(row) = line.strip_prefix("| `") {
            let Some((name, _)) = row.split_once('`') else {
                continue;
            };
            // A table may head its first column with its type: `| `FooHandle` | Description |`.
            if name.starts_with(|c: char| c.is_uppercase()) {
                current = open(&mut groups, name);
                continue;
            }
            if current.is_none() && section == "Props" {
                current = open(&mut groups, "");
            }
            if let Some(group) = current {
                groups[group].1.insert(bare(name).to_string());
            }
        }
    }
    groups
}

/// Todo 1038: the page's property tables and its `public/md` mirror name the same props.
#[test]
fn md_mirrors_list_the_props_of_their_page() {
    let public = Path::new(env!("CARGO_MANIFEST_DIR")).join("public");
    let mut problems = Vec::new();
    for route in Route::static_routes() {
        for (markdown, groups) in pages_of(route.clone()) {
            let Some(markdown) = markdown.filter(|_| !groups.is_empty()) else {
                continue;
            };
            let Ok(md) = std::fs::read_to_string(public.join(markdown.trim_start_matches('/')))
            else {
                problems.push(format!("{route}: no mirror at {markdown}"));
                continue;
            };
            let mirrored = md_props(&md);
            for group in &groups {
                // A group the mirror files under another heading, such as an options
                // struct, is not compared; a page's only group may sit under none.
                let named = |heading: &str| {
                    heading.split('<').next() == group.component().split('<').next()
                };
                let listed = mirrored
                    .iter()
                    .find(|(heading, _)| named(heading))
                    .or_else(|| {
                        mirrored
                            .iter()
                            .find(|(heading, _)| groups.len() == 1 && heading.is_empty())
                    })
                    .map(|(_, names)| names);
                let Some(listed) = listed else {
                    if group.component().ends_with("Handle") {
                        problems.push(format!("{markdown}: no table for `{}`", group.component()));
                    }
                    continue;
                };
                let page: BTreeSet<&str> = group.names().map(bare).collect();
                for name in page.iter().filter(|name| !listed.contains(**name)) {
                    problems.push(format!(
                        "{markdown}: `{}` lacks `{name}`",
                        group.component()
                    ));
                }
                for name in listed.iter().filter(|name| !page.contains(name.as_str())) {
                    problems.push(format!("{markdown}: `{name}` is not on the {route} page"));
                }
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The code block in the defaults, then in as few states as show every control option once.
/// A control the defaults hide is revealed by also moving the control that shows it.
fn demo_sources(code: &DemoCode) -> Vec<String> {
    let base = DemoValues::defaults(&code.controls);
    let hidden = |state: &DemoValues, name: &str| {
        code.controls
            .iter()
            .find(|c| c.name == name)
            .is_some_and(|c| c.is_hidden(state))
    };
    let mut todo: Vec<(&str, &str)> = code
        .controls
        .iter()
        .flat_map(|c| c.options.iter().map(|o| (c.name, o.as_str())))
        .collect();
    let mut states = vec![base.clone()];
    loop {
        let shown = states.last().unwrap();
        todo.retain(|(name, option)| hidden(shown, name) || shown.str(name) != *option);
        let mut state = base.clone();
        let mut moved: Vec<&str> = Vec::new();
        for &(name, option) in &todo {
            if moved.contains(&name) {
                continue;
            }
            let mut next = state.with(name, option);
            let mut also = None;
            if hidden(&next, name) {
                also = code
                    .controls
                    .iter()
                    .filter(|c| c.name != name && !moved.contains(&c.name))
                    .flat_map(|c| c.options.iter().map(|o| (c.name, o.as_str())))
                    .find(|(by, to)| !hidden(&next.with(by, to), name));
                let Some((by, to)) = also else { continue };
                next = next.with(by, to);
            }
            // Only if nothing moved before goes out of sight.
            if moved.iter().any(|m| hidden(&next, m)) {
                continue;
            }
            state = next;
            moved.push(name);
            moved.extend(also.map(|(by, _)| by));
        }
        if moved.is_empty() {
            // What is left no single move reveals; a reader cannot see it
            // without two, and the test does not look for those.
            break;
        }
        states.push(state);
    }
    let mut sources: Vec<String> = Vec::new();
    for state in &states {
        let source = code.source(state);
        if !sources.contains(&source) {
            sources.push(source);
        }
    }
    sources
}

/// Stand-ins for what a `Demo`'s code names but the reader's app would own:
/// the docs' icons as empty components, and its `Asset` statics as paths.
fn stand_ins(sources: &[&str]) -> String {
    let mut items = String::new();
    for source in sources {
        for line in source.lines().map(str::trim) {
            let line = line
                .trim_start_matches("pub(crate) ")
                .trim_start_matches("pub ");
            if let Some(name) = line
                .strip_prefix("fn ")
                .and_then(|rest| rest.strip_suffix("() -> Element {"))
                .filter(|name| name.ends_with("Icon"))
            {
                let _ = writeln!(items, "#[component] fn {name}() -> Element {{ rsx! {{}} }}");
            } else if let Some((name, _)) = line
                .strip_prefix("static ")
                .and_then(|rest| rest.split_once(": Asset ="))
            {
                let _ = writeln!(
                    items,
                    "const {name}: &str = \"/{}.png\";",
                    name.to_lowercase()
                );
            }
        }
    }
    items
}

/// The name a `let` or `item` marker defines, when it defines one.
fn defines(marker: &str) -> Option<&str> {
    let rest = if let Some(rest) = marker.strip_prefix("let ") {
        rest.trim_start_matches("mut ")
    } else {
        let item = marker.strip_prefix("item ")?;
        // Past the attributes and the keyword: `#[component] fn Name(`.
        let item = item.rsplit_once("] ").map_or(item, |(_, rest)| rest);
        let item = ["fn ", "struct ", "enum ", "const ", "static ", "impl "]
            .iter()
            .find_map(|keyword| item.strip_prefix(keyword))?;
        // `impl Trait for Type` is about `Type`.
        item.split_once(" for ").map_or(item, |(_, rest)| rest)
    };
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    (end > 0).then_some(&rest[..end])
}

/// Whether `body` mentions the identifier `name`.
fn names(body: &str, name: &str) -> bool {
    body.match_indices(name).any(|(at, _)| {
        let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
        !body[..at].ends_with(ident) && !body[at + name.len()..].starts_with(ident)
    })
}

/// The line of each `Demo {` in a page, and its markers.
fn demo_markers(source: &str) -> Vec<(usize, Vec<String>)> {
    let lines: Vec<&str> = source.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == "Demo {")
        .map(|(index, _)| (index + 1, markers(&lines, index)))
        .collect()
}

/// Todo 1034: a screen reader lists the code blocks by name, so two demos of a page need two.
#[test]
fn demo_code_labels_are_distinct_per_page() {
    let mut problems = Vec::new();
    for route in Route::static_routes() {
        let labels: Vec<String> = demos_of(route.clone())
            .into_iter()
            .map(|code| code.label)
            .collect();
        for label in &labels {
            if labels.iter().filter(|other| *other == label).count() > 1 {
                problems.push(format!(
                    "{route}: `{label}` names several demos: give each a `title`"
                ));
            }
        }
    }
    problems.sort();
    problems.dedup();
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn page_snippets_are_current() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let pages = root.join("src/pages");
    let mut files = Vec::new();
    rust_files(&pages, &mut files);
    files.sort();
    let sources: Vec<(String, String)> = files
        .iter()
        .map(|file| {
            let path = file.strip_prefix(&pages).unwrap().display().to_string();
            (path, std::fs::read_to_string(file).unwrap())
        })
        .collect();

    let icons = std::fs::read_to_string(root.join("src/icons.rs")).unwrap();
    let main = std::fs::read_to_string(root.join("src/main.rs")).unwrap();

    let mut out = String::from(
        "# Page snippets\n\nGenerated by `docs/src/snippets.rs` from the `const` snippets and \
         the `Demo` code blocks in `docs/src/pages`, and compiled as doc-tests by \
         `libero/src/md_examples.rs`. Gitignored; do not edit: change the page, then run \
         `cargo test -p docs page_snippets`.\n",
    );
    let mut problems = Vec::new();
    let ignore = |out: &mut String, heading: &str, why: &str, body: &str| {
        let _ = write!(
            out,
            "\n## {heading}\n\nNot compiled: {why}.\n\n```rust,ignore\n{}\n```\n",
            body.trim_end()
        );
    };

    for (path, source) in &sources {
        let page = snippets(source);
        for snippet in &page {
            let heading = format!("{path} `{}`", snippet.name);
            let language = printed_language(source, &snippet.name);
            match (ignored(snippet), &language) {
                (Err(problem), _) => problems.push(format!("{path} {problem}")),
                (Ok(Some(why)), _) => ignore(&mut out, &heading, &why, &snippet.body),
                (Ok(None), Some(language)) if language != "rust" => problems.push(format!(
                    "{path} {}: printed as `{language}`, not Rust: mark it \
                     `// snippet: ignore - {language}`",
                    snippet.name
                )),
                (Ok(None), _) => match compilable(snippet, &page) {
                    Ok(code) => {
                        let _ = write!(
                            out,
                            "\n## {heading}\n\n```rust,no_run\n{PRELUDE}\n{code}```\n"
                        );
                    }
                    Err(problem) => problems.push(format!("{path} {problem}")),
                },
            }
        }
    }

    // Each page's `Demo`s, matched to its `Demo {` lines by order.
    let mut demo_count = 0;
    for route in Route::static_routes() {
        let demos = demos_of(route.clone());
        demo_count += demos.len();
        let name = format!("{route:?}");
        let name = name.trim_end_matches(" {}").trim_end_matches(" { }");
        let defined = format!("fn {name}(");
        // A definition, not a mention: a `// snippet: item .. fn Home()` comment is not a page.
        let is_page = |source: &str| {
            source.lines().any(|line| {
                let line = line.trim_start().trim_start_matches("pub ");
                line.starts_with(&defined)
            })
        };
        let Some((path, source)) = sources.iter().find(|(_, s)| is_page(s)) else {
            if !demos.is_empty() {
                problems.push(format!("{route}: no page defines `{defined}`"));
            }
            continue;
        };
        let lines = demo_markers(source);
        if demos.is_empty() && lines.is_empty() {
            continue;
        }
        if lines.len() != demos.len() {
            problems.push(format!(
                "{path}: renders {} `Demo`s but has {} `Demo {{` lines to match them to",
                demos.len(),
                lines.len()
            ));
            continue;
        }
        let page = snippets(source);
        for (code, (line, markers)) in demos.iter().zip(lines) {
            let heading = format!("{path} `Demo` at line {line}");
            let sources = demo_sources(code);
            let whole = Snippet {
                name: format!("Demo at line {line}"),
                body: String::new(),
                markers: markers.clone(),
            };
            match ignored(&whole) {
                Err(problem) => {
                    problems.push(format!("{path} {problem}"));
                    continue;
                }
                Ok(Some(why)) => {
                    ignore(&mut out, &heading, &why, &sources.join("\n\n"));
                    continue;
                }
                Ok(None) => {}
            }
            // A state that prints an ignored `const` is not compiled either,
            // for the same reason. It is listed, not dropped.
            let ignored_consts: Vec<(&Snippet, String)> = page
                .iter()
                .filter_map(|s| Some((s, ignored(s).ok()??)))
                .filter(|(s, _)| !s.body.trim().is_empty())
                .collect();
            let mut block = String::new();
            let mut skipped = String::new();
            for (state, body) in sources.into_iter().enumerate() {
                if let Some((by, why)) = ignored_consts
                    .iter()
                    .find(|(s, _)| body.contains(s.body.trim()))
                {
                    let _ = write!(
                        skipped,
                        "\nState {state} not compiled: it prints `{}`, which is not compiled: {why}.\n\n```rust,ignore\n{}\n```\n",
                        by.name,
                        body.trim_end()
                    );
                    continue;
                }
                // One `Demo` prints different code per state, so a stand-in
                // goes only into the states that name it.
                let mut kept: Vec<&String> = Vec::new();
                loop {
                    let named = |name: &str| {
                        names(&body, name) || kept.iter().any(|marker| names(marker, name))
                    };
                    let next: Vec<&String> = markers
                        .iter()
                        .filter(|marker| defines(marker).is_none_or(named))
                        .collect();
                    if next.len() == kept.len() {
                        break;
                    }
                    kept = next;
                }
                let markers = kept.into_iter().cloned().collect();
                let snippet = Snippet {
                    name: format!("Demo at line {line}, state {state}"),
                    body,
                    markers,
                };
                match compilable(&snippet, &page) {
                    // A module each, so every state has its own `Snippet`.
                    Ok(code) => {
                        let _ = write!(block, "mod state_{state} {{\nuse super::*;\n{code}}}\n");
                    }
                    Err(problem) => problems.push(format!("{path} {problem}")),
                }
            }
            let _ = write!(
                out,
                // Its own `main`, or rustdoc wraps the modules in one and
                // `super` stops reaching the prelude.
                "\n## {heading}\n\n```rust,no_run\n{PRELUDE}\n{}\n{block}\nfn main() {{}}\n```\n",
                stand_ins(&[&icons, &main, source])
            );
            out.push_str(&skipped);
        }
    }

    assert!(problems.is_empty(), "{}", problems.join("\n"));
    // Zero anywhere is the shell failing, not a page: say that, not 90 mismatches.
    assert!(
        demo_count > 0,
        "no route rendered a `Demo`: the shell likely failed"
    );
    // Gitignored, so nothing to commit. Rewritten only when it changed.
    let generated = root.join("../libero/tests/page_snippets.md");
    if std::fs::read_to_string(&generated).ok().as_deref() != Some(&out) {
        std::fs::write(&generated, &out).unwrap();
    }
}
