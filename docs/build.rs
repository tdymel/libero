//! Lists the components `libero::components` exports, so the docs count them
//! instead of writing a number down. `src/exports.rs` includes the output.
//! Also writes the icon catalogue's files, see `icon_catalogue`.

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
    icon_catalogue();
}

/// The Pictogram page's catalogue (1358): one `assets/icons/<set>-<variant>.json` per set and
/// variant from pictogram's index, fetched on demand, and the sets' facts as `icon_sets.rs`.
fn icon_catalogue() {
    let dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/icons");
    let mut sets = String::from("pub static ICON_SETS: &[CatalogueSet] = &[\n");
    for library in pictogram::LIBRARIES {
        let variants: Vec<&str> = library
            .variants
            .iter()
            .copied()
            .filter(|variant| shown(variant))
            .collect();
        for variant in &variants {
            let icons: Vec<_> = library
                .variant(variant)
                .map(|icon| {
                    let svg = icon.svg;
                    (icon.name, icon.module, svg.view_box, svg.attrs, svg.body)
                })
                .collect();
            let json = serde_json::to_string(&icons).unwrap();
            write_if_changed(&dir.join(format!("{}-{variant}.json", library.name)), &json);
        }
        sets += &format!(
            "    CatalogueSet {{ name: {:?}, title: {:?}, license: {:?}, repository: {:?}, upstream_version: {:?}, variants: &{variants:?} }},\n",
            library.name,
            library.title,
            library.license,
            library.repository,
            library.upstream_version,
        );
    }
    sets += "];\n";
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("icon_sets.rs"),
        sets,
    )
    .unwrap();
}

/// Lobe's colour variants hard-code fills and share gradient ids across copies.
fn shown(variant: &str) -> bool {
    variant != "color" && !variant.ends_with("_color")
}

/// Leaves an unchanged file alone, so `dx` sees no asset change on a rerun.
fn write_if_changed(path: &Path, contents: &str) {
    if fs::read_to_string(path).is_ok_and(|old| old == contents) {
        return;
    }
    fs::write(path, contents).unwrap();
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
