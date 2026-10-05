//! Printed snippets against the live code they copy.

use super::*;

/// The code block in the defaults, then in as few states as show every control option once.
/// A control the defaults hide is revealed by also moving the control that shows it.
pub(super) fn demo_sources(code: &DemoCode) -> Vec<String> {
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
/// a page's own `...Icon` components as empty ones, and its `Asset` statics as paths.
pub(super) fn stand_ins(sources: &[&str]) -> String {
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
pub(super) fn defines(marker: &str) -> Option<&str> {
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
pub(super) fn names(body: &str, name: &str) -> bool {
    body.match_indices(name).any(|(at, _)| {
        let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
        !body[..at].ends_with(ident) && !body[at + name.len()..].starts_with(ident)
    })
}

/// The line of each `Demo {` in a page, and its markers.
pub(super) fn demo_markers(source: &str) -> Vec<(usize, Vec<String>)> {
    let lines: Vec<&str> = source.lines().collect();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim() == "Demo {")
        .map(|(index, _)| (index + 1, markers(&lines, index)))
        .collect()
}

/// `line` without its `//` comment; a `//` right after `:` is a URL, not a comment.
fn code_part(line: &str) -> &str {
    line.match_indices("//")
        .find(|(at, _)| !line[..*at].ends_with(':'))
        .map_or(line, |(at, _)| &line[..at])
}

/// Every item named `name` in `source` (`fn`, `const`, `enum`, `impl`, ..), its attributes
/// included, joined.
fn live_items(source: &str, name: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let starts = |line: &str| {
        let line = line
            .trim_start()
            .trim_start_matches("pub(crate) ")
            .trim_start_matches("pub(super) ")
            .trim_start_matches("pub ");
        let rest = [
            "fn ", "const ", "static ", "struct ", "enum ", "type ", "impl ",
        ]
        .iter()
        .find_map(|keyword| line.strip_prefix(keyword));
        // `impl Trait for Name` is about `Name`.
        let rest = rest.map(|rest| rest.split_once(" for ").map_or(rest, |(_, r)| r));
        rest.is_some_and(|rest| {
            rest.strip_prefix(name)
                .is_some_and(|after| !after.starts_with(|c: char| c.is_alphanumeric() || c == '_'))
        })
    };
    let printed = in_raw_strings(&lines);
    lines
        .iter()
        .enumerate()
        .filter(|&(index, l)| !printed[index] && starts(l))
        .map(|(index, _)| item_text(&lines, index))
        .collect()
}

/// Which lines start inside a `r#".."#` literal: a printed `fn Name` is not the live one.
fn in_raw_strings(lines: &[&str]) -> Vec<bool> {
    let mut close: Option<String> = None;
    lines
        .iter()
        .map(|line| {
            let inside = close.is_some();
            let mut rest = *line;
            loop {
                match &close {
                    Some(end) => match rest.find(end.as_str()) {
                        Some(at) => {
                            rest = &rest[at + end.len()..];
                            close = None;
                        }
                        None => break,
                    },
                    None => match rest.find("r#") {
                        Some(at) => {
                            let after = &rest[at + 1..];
                            let hashes = after.len() - after.trim_start_matches('#').len();
                            if !after[hashes..].starts_with('"') {
                                rest = &rest[at + 2..];
                                continue;
                            }
                            close = Some(format!("\"{}", "#".repeat(hashes)));
                            rest = &after[hashes + 1..];
                        }
                        None => break,
                    },
                }
            }
            inside
        })
        .collect()
}

/// The item that starts at line `index`, up to its closing bracket or `;`, attributes included.
fn item_text(lines: &[&str], index: usize) -> String {
    let first = lines[..index]
        .iter()
        .rev()
        .take_while(|l| l.trim_start().starts_with("#["))
        .count();
    let (mut found, mut depth) = (String::new(), 0i32);
    for line in &lines[index - first..] {
        found.push_str(line);
        found.push('\n');
        let code = code_part(line);
        for c in code.chars() {
            match c {
                '{' | '(' | '[' => depth += 1,
                '}' | ')' | ']' => depth -= 1,
                _ => {}
            }
        }
        let end = code.trim_end();
        if depth == 0 && (end.ends_with('}') || end.ends_with(';')) {
            break;
        }
    }
    found
}

/// The identifiers and string literals of `body` that `live` lacks: what drifted.
fn drifted(body: &str, live: &str) -> Vec<String> {
    const KEYWORDS: [&str; 13] = [
        "move", "let", "mut", "self", "Self", "impl", "for", "match", "else", "return", "true",
        "false", "use",
    ];
    // A printed prop (`option: move |o| ..`) is a live `fn`: its name is the page's, not drift.
    let body = match shape(body) {
        Shape::Prop => body.split_once(": ").map_or(body, |(_, rest)| rest),
        _ => body,
    };
    let mut missing = Vec::new();
    // An import is the reader's setup; the snippet compile checks it.
    for line in body
        .lines()
        .map(code_part)
        .filter(|line| !line.trim_start().starts_with("use "))
    {
        // The line without its plain literals, which are checked whole.
        let (mut code, mut rest) = (String::new(), line);
        while let Some(at) = rest.find('"') {
            let Some(len) = rest[at + 1..].find('"') else {
                break;
            };
            let literal = &rest[at..at + len + 2];
            code.push_str(&rest[..at]);
            // `"{x}"` is interpolation, its names are checked as identifiers.
            if literal.contains(['{', '\\']) {
                code.push_str(literal);
            } else if len >= 3 && !live.contains(literal) {
                missing.push(literal.to_string());
            }
            rest = &rest[at + len + 2..];
        }
        code.push_str(rest);
        let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
        for word in code.split(|c: char| !ident(c)) {
            if word.len() >= 3
                && !word.starts_with(|c: char| c.is_ascii_digit())
                && !KEYWORDS.contains(&word)
                && !names(live, word)
            {
                missing.push(word.to_string());
            }
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

/// Every tagged item other than a `const NAME: &str`, one `Snippet` per string literal in it:
/// an array of props (Carousel's `FIXED`) or a `fn code` that returns the printed text.
fn printed_items(source: &str) -> Vec<Snippet> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let line = line
            .trim_start_matches("pub(super) ")
            .trim_start_matches("pub ");
        let Some(rest) = ["const ", "static ", "fn "]
            .iter()
            .find_map(|keyword| line.strip_prefix(keyword))
        else {
            continue;
        };
        let markers = markers(&lines, index);
        // `snippets` reads the `&str` consts, and only tagged items matter here.
        if rest.contains(": &str =") || !markers.iter().any(|m| m.starts_with("mirrors ")) {
            continue;
        }
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let text = item_text(&lines, index);
        let mut rest_text = text.as_str();
        while let Some(at) = rest_text.find('"') {
            let hashes = at - rest_text[..at].trim_end_matches('#').len();
            let raw = rest_text[..at - hashes].ends_with('r');
            let close = format!("\"{}", "#".repeat(hashes));
            let body = &rest_text[at + 1..];
            // A plain literal ends at the first `"` not escaped.
            let len = if raw {
                body.find(&close).unwrap()
            } else {
                body.char_indices()
                    .find(|&(i, c)| c == '"' && !body[..i].ends_with('\\'))
                    .unwrap()
                    .0
            };
            let literal = &body[..len];
            found.push(Snippet {
                name: rest[..end].to_string(),
                body: if raw {
                    literal.to_string()
                } else {
                    literal.replace("\\\"", "\"")
                },
                markers: markers.clone(),
            });
            rest_text = &body[len + close.len()..];
        }
    }
    found
}

/// Todo 1858: a printed `const` tagged `// snippet: mirrors <live item>, ..` names nothing
/// the live demo does not, so the two cannot drift apart. The live items are looked up in
/// the page and in its `<page>/demo.rs`.
#[test]
fn printed_snippets_mirror_their_live_code() {
    let pages = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pages");
    let mut files = Vec::new();
    rust_files(&pages, &mut files);
    files.sort();
    let (mut problems, mut tagged) = (Vec::new(), 0);
    for file in &files {
        let source = std::fs::read_to_string(file).unwrap();
        let demo =
            std::fs::read_to_string(file.with_extension("").join("demo.rs")).unwrap_or_default();
        let path = file.strip_prefix(&pages).unwrap().display();
        for snippet in snippets(&source).into_iter().chain(printed_items(&source)) {
            for marker in snippet
                .markers
                .iter()
                .filter_map(|m| m.strip_prefix("mirrors "))
            {
                tagged += 1;
                // `except` lists what only the print has: an asset path the reader owns.
                let (names, except) = marker.split_once(" except ").unwrap_or((marker, ""));
                let mut live = String::new();
                for name in names.split(',').map(str::trim) {
                    let item = live_items(&source, name) + &live_items(&demo, name);
                    if item.is_empty() {
                        problems.push(format!("{path} {}: no live item `{name}`", snippet.name));
                    }
                    live += &item;
                }
                let mut missing = drifted(&snippet.body, &live);
                missing.retain(|token| match token.starts_with('"') {
                    true => !except.contains(token.as_str()),
                    false => !except.split_whitespace().any(|e| e == token),
                });
                if !missing.is_empty() {
                    problems.push(format!(
                        "{path} {}: printed but not in `{names}`: {}",
                        snippet.name,
                        missing.join(" ")
                    ));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert!(tagged > 0, "no `// snippet: mirrors` marker found");
}

#[test]
fn live_items_skip_the_printed_copy() {
    let source = "const CODE: &str = r#\"use x::y;\nfn demo() {\n    printed(r#type);\n}\"#;\n\nfn demo() {\n    live();\n}\n";
    let item = live_items(source, "demo");
    assert!(
        item.contains("live()") && !item.contains("printed"),
        "{item}"
    );
}

#[test]
fn a_drifted_snippet_names_what_the_live_item_lacks() {
    let source = "#[derive(Clone)]\nenum Fruit {\n    Apple,\n}\n\nfn row(f: Fruit) -> Element {\n    rsx! { \"Fresh\" }\n}\n";
    assert!(live_items(source, "Fruit").starts_with("#[derive(Clone)]"));
    assert!(!live_items(source, "Fruit").contains("row"));
    let printed =
        "fn row(f: Fruit) -> Element {\n    // Banana is a comment\n    rsx! { \"Ripe\" Pear }\n}";
    assert_eq!(
        drifted(printed, &live_items(source, "row")),
        ["\"Ripe\"", "Pear"]
    );
}

/// Todo 1034: a screen reader lists the code blocks by name, so two demos of a page need two.
#[test]
fn demo_code_labels_are_distinct_per_page() {
    let mut problems = Vec::new();
    for Rendered { route, demos, .. } in rendered() {
        let labels: Vec<&String> = demos.iter().map(|code| &code.label).collect();
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
