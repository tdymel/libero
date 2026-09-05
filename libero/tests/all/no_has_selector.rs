//! No `:has()` in libero's CSS: stylo rejects it at parse time, so under
//! Blitz every rule that uses it is dropped ([[codebase/blitz-platform-gaps]]).
//! A source scan rather than a render, so a component no test renders is
//! covered too.

use std::{fs, path::Path};

/// Where `:has(` may be written: the public helper that exists for web-only
/// callers, and the selector splitter's test of a nested comma.
const ALLOWED: &[&str] = &["src/sx/sx.rs", "src/css/selector.rs"];

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
        if ALLOWED.contains(&relative.as_str()) {
            continue;
        }
        for (index, line) in fs::read_to_string(&path).unwrap().lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if code.contains(":has(") {
                hits.push(format!("{relative}:{}: {code}", index + 1));
            }
        }
    }
}

#[test]
fn no_selector_in_libero_uses_has() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut hits = Vec::new();
    scan(&root.join("src"), root, &mut hits);
    assert!(
        hits.is_empty(),
        "`:has()` never matches natively; use a sibling rule (`ring_overlay`):\n{}",
        hits.join("\n")
    );
}
