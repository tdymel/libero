//! Declares the fixture modules from `src/*.rs` (todo 1618): every one, or with `E2E_FIXTURES`
//! (the e2e runner sets it for a unit filter) only those named, so a one-unit run compiles less.

use std::fmt::Write;

/// Declared by hand in `lib.rs`: the native tests and `main.rs` use them, so they always build.
const ALWAYS: &[&str] = &["lib", "main", "common", "docs_shell", "home", "perf"];

fn main() {
    println!("cargo:rerun-if-env-changed=E2E_FIXTURES");
    println!("cargo:rerun-if-changed=src");
    println!("cargo::rustc-check-cfg=cfg(fixtures_only)");
    let only = std::env::var("E2E_FIXTURES").ok();
    if only.is_some() {
        println!("cargo::rustc-cfg=fixtures_only");
    }
    let mut wanted: Option<Vec<String>> = only
        .as_deref()
        .map(|list| list.split(',').map(str::to_string).collect());

    let src = std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("src");
    let mut modules: Vec<String> = std::fs::read_dir(&src)
        .unwrap()
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            Some(name.strip_suffix(".rs")?.to_string())
        })
        .filter(|module| !ALWAYS.contains(&module.as_str()))
        .collect();
    modules.sort();
    // A wanted module may use another's items (`planted` uses `segmented_control::`).
    if let Some(wanted) = wanted.as_mut() {
        let mut at = 0;
        while at < wanted.len() {
            let source =
                std::fs::read_to_string(src.join(format!("{}.rs", wanted[at]))).unwrap_or_default();
            for module in &modules {
                if !wanted.contains(module) && names(&source, module) {
                    wanted.push(module.clone());
                }
            }
            at += 1;
        }
    }
    let (built, left_out): (Vec<_>, Vec<_>) = modules
        .iter()
        .partition(|module| wanted.as_ref().is_none_or(|w| w.contains(module)));

    let mut out = String::new();
    for module in &built {
        let path = src.join(format!("{module}.rs"));
        writeln!(
            out,
            "#[path = {:?}]\nmod {module};",
            path.display().to_string()
        )
        .unwrap();
    }
    out.push_str("\n/// The routes of the generated modules.\nconst GENERATED: &[Routes] = &[\n");
    for module in &built {
        writeln!(out, "    {module}::ROUTES,").unwrap();
    }
    out.push_str("];\n");
    writeln!(
        out,
        "\n/// The `E2E_FIXTURES` this build was made with.\nconst ONLY: Option<&str> = {only:?};"
    )
    .unwrap();
    out.push_str("\n/// Each route of a module `E2E_FIXTURES` left out, and that module.\nconst LEFT_OUT: &[(&str, &str)] = &[\n");
    for module in &left_out {
        let source = std::fs::read_to_string(src.join(format!("{module}.rs"))).unwrap();
        for path in route_paths(&source) {
            writeln!(out, "    ({path:?}, {module:?}),").unwrap();
        }
    }
    out.push_str("];\n");

    let file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("fixtures.rs");
    if std::fs::read_to_string(&file).ok().as_deref() != Some(out.as_str()) {
        std::fs::write(file, out).unwrap();
    }
}

/// Whether `source` has a crate path through `module`: `crate::module::` or `{module::` in a
/// `use crate::{..}`, not `other_module::` or `libero::components::module::`.
fn names(source: &str, module: &str) -> bool {
    let needle = format!("{module}::");
    source.match_indices(&needle).any(|(at, _)| {
        let before = &source[..at];
        before.ends_with("crate::")
            || !before
                .chars()
                .next_back()
                .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == ':')
    })
}

/// The `("/path", ..)` literals after a module's `ROUTES`.
fn route_paths(source: &str) -> Vec<String> {
    let Some(at) = source.find("ROUTES") else {
        return Vec::new();
    };
    source[at..]
        .split("(\"/")
        .skip(1)
        .filter_map(|rest| Some(format!("/{}", &rest[..rest.find('"')?])))
        .collect()
}
