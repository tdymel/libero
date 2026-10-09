//! The layering, checked on the source files with archunit (todos 178, 359, 820).
//! Read `.agents/brain/codebase/architecture.md` for the why; this file is the what.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use archunit::{
    Graph, SourceOptions, assert_passes, extract_graph, locate_project, pattern, project_layers,
    slice_by_regex,
};

/// Top to bottom: a layer may use any layer below it, never one above.
/// `backend` is declared before `platform`, so its files are not `platform`'s.
const LAYERS: [(&str, &str); 10] = [
    ("components", "libero/src/components/**"),
    // A hook may render a component's context, never the reverse.
    ("hooks", "libero/src/hooks/**"),
    ("context", "libero/src/context/**"),
    ("localization", "libero/src/localization/**"),
    ("backend", "libero/src/platform/backend/**"),
    ("platform", "libero/src/platform/**"),
    ("theme", "libero/src/theme/**"),
    ("sx", "libero/src/sx/**"),
    ("tokens", "libero/src/tokens/**"),
    ("css", "libero/src/css/**"),
];

/// The renderer crates: each backend's own, reached only inside `platform`.
const RENDERER_CRATES: [&str; 11] = [
    "web_sys",
    "js_sys",
    "wasm_bindgen",
    "wasm_bindgen_futures",
    "dioxus_native_dom",
    "blitz_dom",
    "blitz_traits",
    "style",
    "arboard",
    "rfd",
    "futures_core",
];

/// Every crate `docs` may name, each for one reason.
const DOCS_CRATES: [(&str, &str); 11] = [
    ("dioxus", "the app itself"),
    (
        "pictogram_icons_lucide",
        "the glyphs a reader's app passes as `SvgData`",
    ),
    (
        "pictogram_icons_simple",
        "the GitHub and Markdown marks on every page",
    ),
    (
        "pictogram_core",
        "`Icon` and `Svg`, the Pictogram page's icon catalogue",
    ),
    (
        "pictogram",
        "`build.rs` writes the icon catalogue's files from every set's index",
    ),
    ("serde_json", "reads and writes the icon catalogue's files"),
    ("std", "the standard library"),
    (
        "dioxus_native",
        "the `native-cpu` launch, with its own renderer features",
    ),
    (
        "web_sys",
        "reads the stored direction before the first render",
    ),
    (
        "web_time",
        "the use_timers stopwatch's `Instant`, which std panics on in wasm",
    ),
    (
        "log",
        "the `native` build prints the wgpu adapter, which the dioxus logger drops",
    ),
];

const CATEGORY: &str = r"^libero/src/components/([a-z_]+)/";

/// Extracted once: archunit caches it too, but tests starting together each missed that cache
/// and extracted their own.
fn graph() -> &'static Graph {
    static GRAPH: OnceLock<Graph> = OnceLock::new();
    GRAPH.get_or_init(|| {
        let project = locate_project().unwrap();
        extract_graph(&project, SourceOptions::new())
            .unwrap()
            .into_graph()
    })
}

/// A test module split into its own `tests.rs` may render anything: it is not a layer's code.
/// Inline `#[cfg(test)]` modules cannot be told apart, which is why they get split out.
fn is_test_file(path: &str) -> bool {
    path.ends_with("/tests.rs")
}

#[test]
fn no_layer_uses_one_above_it() {
    // Fills archunit's cache with the same extraction the rule below asks for.
    graph();
    let mut rule = project_layers();
    for (name, path) in LAYERS {
        rule = rule
            .layer(name)
            .defined_by(pattern(path).except("**/tests.rs"));
    }
    for (index, (name, _)) in LAYERS.iter().enumerate() {
        let mut forbidden: Vec<&str> = LAYERS[..index].iter().map(|(name, _)| *name).collect();
        // Only `platform` opens `backend`: everything else goes through its facade.
        match *name {
            "platform" => forbidden.retain(|layer| *layer != "backend"),
            "backend" => {}
            _ => forbidden.push("backend"),
        }
        if !forbidden.is_empty() {
            rule = rule.where_layer(*name).may_not_depend_on_layers(&forbidden);
        }
    }
    assert_passes!(rule);
}

#[test]
fn renderer_crates_are_only_named_inside_platform() {
    let outside: BTreeSet<String> = graph()
        .edges()
        .iter()
        .filter(|edge| edge.external && RENDERER_CRATES.contains(&edge.target.as_str()))
        .filter(|edge| edge.source.starts_with("libero/"))
        .filter(|edge| !edge.source.starts_with("libero/src/platform/"))
        .map(|edge| format!("{} -> {}", edge.source, edge.target))
        .collect();
    assert!(outside.is_empty(), "go through `platform`:\n{outside:#?}");
}

#[test]
fn docs_uses_libero_through_its_root_and_known_crates() {
    let graph = graph();
    let from_docs = || {
        graph
            .edges()
            .iter()
            .filter(|edge| edge.source.starts_with("docs/"))
    };
    let unknown: BTreeSet<&str> = from_docs()
        .filter(|edge| edge.external)
        .map(|edge| edge.target.as_str())
        .filter(|name| DOCS_CRATES.iter().all(|(known, _)| known != name))
        .collect();
    assert!(
        unknown.is_empty(),
        "add each to `DOCS_CRATES` with its reason: {unknown:?}"
    );
    // The public API, as a user writes it: `libero::X`, never a module path into the tree.
    let inside: BTreeSet<String> = from_docs()
        .filter(|edge| !edge.external && !edge.target.starts_with("docs/"))
        .filter(|edge| edge.target != "libero/src/lib.rs")
        .map(|edge| format!("{} -> {}", edge.source, edge.target))
        .collect();
    assert!(
        inside.is_empty(),
        "import from the crate root:\n{inside:#?}"
    );
}

/// Inside the crate a component names its category: `components::<category>::X`.
#[test]
fn components_import_from_the_category_not_the_flat_re_export() {
    let categories = slice_by_regex(CATEGORY).unwrap();
    let flat: BTreeSet<String> = graph()
        .edges()
        .iter()
        .filter(|edge| !edge.external && edge.target == "libero/src/components/mod.rs")
        .filter(|edge| categories.label_for(&edge.source).is_some())
        .filter(|edge| !is_test_file(&edge.source))
        .map(|edge| edge.source.clone())
        .collect();
    assert!(
        flat.is_empty(),
        "import `components::<category>::X`, not `components::X`:\n{flat:#?}"
    );
}
