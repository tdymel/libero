//! Cutting a snippet into pieces and reading its shape and directives.

use super::*;

/// The first line of code in `body`, comments skipped.
pub(super) fn first_line(body: &str) -> &str {
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

pub(super) fn shape(body: &str) -> Shape {
    let first = first_line(body);
    const ITEMS: [&str; 12] = [
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
        "type ",
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
pub(super) fn chunks(body: &str) -> Vec<&str> {
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
pub(super) fn markers(lines: &[&str], index: usize) -> Vec<String> {
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
pub(super) fn snippets(source: &str) -> Vec<Snippet> {
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
pub(super) fn printed_language(source: &str, name: &str) -> Option<String> {
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
pub(super) fn ignored(snippet: &Snippet) -> Result<Option<String>, String> {
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
