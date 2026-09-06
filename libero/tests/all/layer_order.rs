//! `theme` and `hooks` sit below `components` in the layer order, so neither
//! may import from it (todo 178). A source scan, so a module no test compiles
//! into a render is covered too. Comments and the `#[cfg(test)]` module at a
//! file's end are skipped: a test may render a component.

use std::{fs, path::Path};

/// The one statement allowed to name `components`: the overlay hooks, which
/// live beside their components, stay public under `hooks`.
const REEXPORT: (&str, &str) = ("hooks/mod.rs", "pub use crate::components::overlay::");

/// Known and filed: `use_box_css` takes an `Input<ClassList>`, and `Input`
/// drags `ClassList`, `States` and `Variables` along with it.
const KNOWN: &[&str] = &["hooks/stylesheet.rs"];

/// The 1-based line of each `crate::components` path, or `use crate::{..}`
/// naming `components`, in `source`.
fn upward_imports(source: &str, allowed: Option<&str>) -> Vec<usize> {
    // Only a test module ends the scan, not a `#[cfg(test)]` helper mid-file.
    let test_module = source
        .match_indices("#[cfg(test)]")
        .map(|(at, attribute)| at + attribute.len())
        .find(|&after| source[after..].trim_start().starts_with("mod "));
    let code = &source[..test_module.unwrap_or(source.len())];
    let code: String = code
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("//") {
                ""
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut hits = Vec::new();
    for (start, _) in code.match_indices("crate::") {
        let statement_start = code[..start].rfind([';', '}', '\n']).map_or(0, |at| at + 1);
        let end = code[start..].find(';').map_or(code.len(), |at| start + at);
        let statement = &code[statement_start..end];
        if allowed.is_some_and(|prefix| statement.trim_start().starts_with(prefix)) {
            continue;
        }
        let path = &code[start + "crate::".len()..end];
        let names_components = path.starts_with("components")
            || (path.starts_with('{')
                && path
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|word| word == "components"));
        if names_components {
            hits.push(code[..start].lines().count().max(1));
        }
    }
    hits.dedup();
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
        if KNOWN.contains(&relative.as_str()) {
            continue;
        }
        let allowed = (relative == REEXPORT.0).then_some(REEXPORT.1);
        for line in upward_imports(&fs::read_to_string(&path).unwrap(), allowed) {
            hits.push(format!("{relative}:{line}"));
        }
    }
}

#[test]
fn the_scan_finds_an_upward_import() {
    assert_eq!(upward_imports("use crate::components::Variant;", None), [1]);
    assert_eq!(
        upward_imports(
            "use crate::{\n    components::Modal,\n    hooks::X,\n};",
            None
        ),
        [1]
    );
    assert_eq!(
        upward_imports("let x = crate::components::Select::<String> {};", None),
        [1]
    );
    assert!(upward_imports("use crate::{hooks::X, theme::Size};", None).is_empty());
    assert!(upward_imports("/// [`Box`](crate::components::Box)", None).is_empty());
    assert!(
        upward_imports(
            "#[cfg(test)]\nmod tests { use crate::components::Modal; }",
            None
        )
        .is_empty()
    );
    let reexport = "pub use crate::components::overlay::{a::B};";
    assert!(upward_imports(reexport, Some(REEXPORT.1)).is_empty());
    assert_eq!(upward_imports(reexport, None), [1]);
}

#[test]
fn theme_and_hooks_import_nothing_from_components() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    for layer in ["theme", "hooks"] {
        scan(&src.join(layer), &src, &mut hits);
    }
    assert!(
        hits.is_empty(),
        "`theme` and `hooks` sit below `components`; move the type down or the hook up:\n{}",
        hits.join("\n")
    );
}
