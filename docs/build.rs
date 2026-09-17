//! Lists the components `libero::components` exports, so the docs count them
//! instead of writing a number down. `src/exports.rs` includes the output.

use std::{env, fs, path::Path};

fn main() {
    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("../libero/src/components");
    // A directory reruns on a change to any file below it.
    println!("cargo::rerun-if-changed={}", root.display());
    let mut names = Vec::new();
    collect(&root, &mut names);
    names.sort();

    let list: String = names
        .iter()
        .map(|name| format!("    \"{name}\",\n"))
        .collect();
    let uses: String = names
        .iter()
        .map(|name| format!("    {name} as _,\n"))
        .collect();
    let out = format!(
        "pub const COMPONENTS: &[&str] = &[\n{list}];\n\n\
         // Fails to build if a listed name is not exported.\n\
         #[allow(unused_imports)]\nuse libero::components::{{\n{uses}}};\n"
    );
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("components.rs"),
        out,
    )
    .unwrap();
}

/// Every `pub fn` at the top of a file with a capitalised name that returns
/// `Element`: the shape of a component. `pub(crate)` ones stay out.
fn collect(dir: &Path, names: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, names);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        for (at, _) in source.match_indices("\npub fn ") {
            let rest = &source[at + "\npub fn ".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            let signature = &rest[..rest.find('{').unwrap_or(rest.len())];
            if name.starts_with(|c: char| c.is_ascii_uppercase())
                && signature.contains("-> Element")
            {
                names.push(name);
            }
        }
    }
}
