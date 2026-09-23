//! The layering, checked on the source files with archunit (todos 178, 359, 820).
//! Read `.agents/brain/codebase/architecture.md` for the why; this file is the what.

use std::collections::{BTreeMap, BTreeSet};

use archunit::{
    Edge, Graph, ImportKind, ProjectedEdge, SliceProjection, SourceOptions, assert_passes,
    extract_dependencies, extract_graph, locate_project, pattern, project_cycles, project_layers,
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
const DOCS_CRATES: [(&str, &str); 5] = [
    ("dioxus", "the app itself"),
    (
        "pictogram_icons_lucide",
        "the glyphs a reader's app passes as `SvgData`",
    ),
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

/// The component units, top to bottom: a unit may use its own tier and any below.
/// Within a tier only the cycle check applies (todo 875). A unit is a category
/// directory, or `base` for `BASE_FILES`.
const CATEGORY_TIERS: [(&str, &[&str]); 5] = [
    // Composites of the tiers below: `ThemeToggle` opens a `Menu`.
    ("composite", &["buttons", "navigation", "form"]),
    // `Lightbox` is a `Carousel`; `Dialog` has a `Title`.
    ("overlay", &["overlay"]),
    ("content", &["typography", "feedback", "data_display"]),
    // What every category may render: `Box`, `VisuallyHidden` and `BASE_FILES`.
    ("base", &["layout", "accessibility", "base"]),
    ("common", &["common"]),
];

/// The base tier's files. They stay in their docs group's folder (722 q2), so
/// the tier is declared by file.
const BASE_FILES: [&str; 7] = [
    "libero/src/components/buttons/action_icon.rs",
    // Draws every glyph: `ActionIcon`'s and `Icon`'s.
    "libero/src/components/data_display/pictogram.rs",
    "libero/src/components/buttons/button.rs",
    // `CodeBlock`'s copy control.
    "libero/src/components/buttons/copy_button.rs",
    "libero/src/components/feedback/loader.rs",
    "libero/src/components/overlay/tooltip.rs",
    // `Tooltip`'s pointer delays, shared with `HoverCard`.
    "libero/src/components/overlay/hover_intent.rs",
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

/// The flat `components::X` re-exports hide the category from the tier checks below.
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

/// A component file's unit: `base` for `BASE_FILES`, its category otherwise.
fn unit_of(categories: &SliceProjection, file: &str) -> Option<String> {
    if BASE_FILES.contains(&file) {
        return Some("base".to_owned());
    }
    categories.label_for(file)
}

/// The unit-to-unit edges, production files only. A `components::<category>::X`
/// import lands on the category's `mod.rs`; it is followed to the file whose
/// `pub use` gives `X`, so a base file's item counts as `base`.
fn category_edges() -> Vec<ProjectedEdge> {
    let categories = slice_by_regex(CATEGORY).unwrap();
    let references = extract_dependencies(&locate_project().unwrap(), SourceOptions::new())
        .unwrap()
        .references()
        .to_vec();
    let reexports: BTreeMap<(&str, &str), &str> = references
        .iter()
        .filter(|reference| reference.kind() == ImportKind::PubUse)
        .filter_map(|reference| {
            let item = reference.referenced_path().rsplit("::").next()?;
            Some(((reference.source(), item), reference.internal_target()?))
        })
        .collect();
    let mut grouped: BTreeMap<(String, String), Vec<Edge>> = BTreeMap::new();
    for reference in &references {
        let (source, Some(mut target)) = (reference.source(), reference.internal_target()) else {
            continue;
        };
        if is_test_file(source) {
            continue;
        }
        if let Some(category) = target
            .strip_suffix("/mod.rs")
            .and_then(|dir| dir.strip_prefix("libero/src/components/"))
        {
            let path = reference.referenced_path();
            let item = path
                .split("::")
                .skip_while(|segment| *segment != category)
                .nth(1)
                .or_else(|| path.split("::").next());
            if let Some(file) = item.and_then(|item| reexports.get(&(target, item))) {
                target = file;
            }
        }
        let (Some(from), Some(to)) = (unit_of(&categories, source), unit_of(&categories, target))
        else {
            continue;
        };
        if from != to {
            grouped.entry((from, to)).or_default().push(Edge::new(
                source,
                target,
                false,
                [reference.kind()],
            ));
        }
    }
    grouped
        .into_iter()
        .map(|((from, to), edges)| ProjectedEdge::new(from, to, edges))
        .collect()
}

fn tier_of(category: &str) -> Option<usize> {
    CATEGORY_TIERS
        .iter()
        .position(|(_, members)| members.contains(&category))
}

#[test]
fn every_component_category_has_a_tier() {
    let components = concat!(env!("CARGO_MANIFEST_DIR"), "/src/components");
    let untiered: Vec<String> = std::fs::read_dir(components)
        .unwrap()
        .map(|entry| entry.unwrap())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.file_name().into_string().unwrap())
        .filter(|name| tier_of(name).is_none())
        .collect();
    assert!(untiered.is_empty(), "add to `CATEGORY_TIERS`: {untiered:?}");
}

#[test]
fn no_component_category_uses_a_tier_above_it() {
    // `CATEGORY_TIERS` runs top to bottom, so a lower index is a higher tier.
    let upward: Vec<String> = category_edges()
        .iter()
        .filter(|edge| tier_of(&edge.target_label) < tier_of(&edge.source_label))
        .map(|edge| {
            let files: BTreeSet<&str> = edge
                .cumulated_edges
                .iter()
                .map(|raw| raw.source.as_str())
                .collect();
            format!("{} -> {} {files:?}", edge.source_label, edge.target_label)
        })
        .collect();
    assert!(
        upward.is_empty(),
        "move the shared item down to `BASE_FILES` or `components/common`:\n  {}",
        upward.join("\n  ")
    );
}

#[test]
fn component_categories_have_no_cycles() {
    let edges = category_edges();
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
        "move the shared item down to `BASE_FILES` or `components/common`:\n  {}",
        cycles.join("\n\n  ")
    );
}
