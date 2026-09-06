//! No `rsx! { {children} }` handed straight to a call. `render` takes an
//! `Element` or a `Vec<Element>` as it is, and the wrapper costs a template and
//! a dynamic node every render (`IntoChildren`, todo 42). A source scan rather
//! than a render, so a component no test renders is covered too.
//!
//! Only the last-argument position is checked. An `rsx!` that turns an iterator or a
//! `String` into an `Element` - a `match` arm, a return value, a prop - is the
//! conversion doing real work.

use std::{fs, path::Path};

/// The index just past the brace that closes the one at `open`.
fn close(text: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (index, byte) in text.iter().enumerate().skip(open) {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Each `rsx! { {..} }` whose next character is a call's closing `)`.
fn wrapped_arguments(source: &str) -> Vec<usize> {
    let text = source.as_bytes();
    let mut hits = Vec::new();
    for (start, _) in source.match_indices("rsx!") {
        let Some(open) = source[start..].find('{').map(|at| start + at) else {
            continue;
        };
        let Some(end) = close(text, open) else {
            continue;
        };
        let inner = source[open + 1..end - 1].trim();
        let single = inner.starts_with('{') && close(inner.as_bytes(), 0) == Some(inner.len());
        // A trailing comma still makes it the last argument.
        let rest = source[end..].trim_start();
        let argument = rest
            .strip_prefix(',')
            .unwrap_or(rest)
            .trim_start()
            .starts_with(')');
        if single && argument {
            hits.push(source[..start].lines().count());
        }
    }
    hits
}

fn scan(dir: &Path, root: &Path, hits: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            scan(&path, root, hits);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        for line in wrapped_arguments(&fs::read_to_string(&path).unwrap()) {
            hits.push(format!("{relative}:{line}"));
        }
    }
}

#[test]
fn the_scan_finds_the_shape_and_leaves_conversions_alone() {
    assert_eq!(
        wrapped_arguments("x.render(Div, v, rsx! { {body} })").len(),
        1
    );
    assert_eq!(
        wrapped_arguments("x.render(Div, v, rsx! {\n {rows.into_iter()}\n})").len(),
        1
    );
    assert!(wrapped_arguments("None => rsx! { {items.into_iter()} },").is_empty());
    assert!(wrapped_arguments("x.render(Div, v, rsx! { {a} {b} })").is_empty());
}

#[test]
fn no_call_in_libero_takes_a_wrapped_element() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    scan(&root.join("src"), root, &mut hits);
    assert!(
        hits.is_empty(),
        "pass the `Element` or `Vec<Element>` itself, not `rsx! {{ {{..}} }}` around it:\n{}",
        hits.join("\n")
    );
}
