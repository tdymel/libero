//! Turning one page's snippets into a compilable function.

use super::*;

/// The snippet as a block that compiles on its own after `PRELUDE`, or what
/// its markers lack. `ignore` is handled by the caller.
pub(super) fn compilable(snippet: &Snippet, page: &[Snippet]) -> Result<String, String> {
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
        } else if !marker.starts_with("ignore") && !marker.starts_with("mirrors ") {
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

pub(super) fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            into.push(path);
        }
    }
}
