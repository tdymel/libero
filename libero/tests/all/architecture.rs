//! The layering, checked on the source files with archunit (todos 178, 359, 820).
//! Read `.agents/brain/codebase/architecture.md` for the why; this file is the what.

use std::collections::BTreeSet;

use archunit::{
    Graph, ProjectedEdge, SourceOptions, assert_passes, extract_graph, locate_project, pattern,
    project_cycles, project_layers, slice_by_regex,
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
const DOCS_CRATES: [(&str, &str); 4] = [
    ("dioxus", "the app itself"),
    ("std", "the standard library"),
    (
        "dioxus_native",
        "the `native-cpu` launch, with its own renderer features",
    ),
    (
        "web_sys",
        "reads the stored direction before the first render",
    ),
];

/// Category edges that close a cycle, each allowed from the named files only.
/// The item they import is public, so moving it is an API change (todo texts).
/// Categories follow the docs groups, not a layering: the layering todo empties this.
const KNOWN_CYCLES: [(&str, &str, &[&str]); 3] = [
    // `ColorSchemeButton` opens a `Menu`.
    (
        "buttons",
        "overlay",
        &["libero/src/components/buttons/color_scheme_button.rs"],
    ),
    // `Alert`'s close is an `ActionIcon`; buttons show feedback's `Loader`.
    (
        "feedback",
        "buttons",
        &["libero/src/components/feedback/alert.rs"],
    ),
    // `AvatarGroup` names its avatars in a `Tooltip`; `Lightbox` is a `Carousel`.
    (
        "data_display",
        "overlay",
        &["libero/src/components/data_display/avatar/group.rs"],
    ),
];

const CATEGORY: &str = r"^libero/src/components/([a-z_]+)/";

fn graph() -> Graph {
    let project = locate_project().unwrap();
    extract_graph(&project, SourceOptions::new())
        .unwrap()
        .into_graph()
}

/// A test module split into its own `tests.rs` may render anything: it is not a layer's code.
/// Inline `#[cfg(test)]` modules cannot be told apart, which is why they get split out.
fn is_test_file(path: &str) -> bool {
    path.ends_with("/tests.rs")
}

#[test]
fn no_layer_uses_one_above_it() {
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

/// The flat `components::X` re-exports hide the category from the cycle check below.
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

#[test]
fn component_categories_have_no_cycles() {
    let graph = graph();
    let categories = slice_by_regex(CATEGORY).unwrap();
    let mut edges: Vec<ProjectedEdge> = categories.project(&graph);
    for edge in &mut edges {
        edge.cumulated_edges.retain(|raw| {
            !raw.external
                && !is_test_file(&raw.source)
                && categories.label_for(&raw.target).is_some()
        });
    }
    let mut stale = Vec::new();
    for (from, to, files) in KNOWN_CYCLES {
        let Some(edge) = edges
            .iter_mut()
            .find(|edge| edge.source_label == from && edge.target_label == to)
        else {
            stale.push(format!("{from} -> {to}"));
            continue;
        };
        for file in files {
            if !edge.cumulated_edges.iter().any(|raw| raw.source == *file) {
                stale.push(format!("{from} -> {to} from {file}"));
            }
        }
        edge.cumulated_edges
            .retain(|raw| !files.contains(&raw.source.as_str()));
    }
    assert!(
        stale.is_empty(),
        "fixed, drop from `KNOWN_CYCLES`: {stale:?}"
    );
    edges.retain(|edge| !edge.cumulated_edges.is_empty());

    let cycles: Vec<String> = project_cycles(&edges)
        .iter()
        .map(|cycle| {
            cycle
                .iter()
                .map(|edge| {
                    let files: BTreeSet<&str> = edge
                        .cumulated_edges
                        .iter()
                        .map(|raw| raw.source.as_str())
                        .collect();
                    format!("{} -> {} {files:?}", edge.source_label, edge.target_label)
                })
                .collect::<Vec<_>>()
                .join("\n  ")
        })
        .collect();
    assert!(
        cycles.is_empty(),
        "move the shared item down to `components/common`:\n  {}",
        cycles.join("\n\n  ")
    );
}
