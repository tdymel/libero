//! What a component costs to re-render, in nanoseconds per instance.
//!
//! Ignored by default - it is a measurement, not an assertion, and the numbers
//! only mean anything in a release build:
//!
//! ```text
//! cargo test --release -p libero --test render_cost -- --ignored --nocapture
//! ```
//!
//! `span` is a bare element with no scope; `Leaf` the cheapest possible
//! component. The ratios travel between machines, the absolutes do not - so
//! compare a change against a run of `main` on the same machine, not against a
//! number written down anywhere. The current baseline table lives in
//! `memory/performance.md`.
//!
//! Every shape renders its component in the plainest configuration that
//! compiles: what is measured is the framework overhead a caller pays for
//! reaching for the component at all, not any particular feature of it.
//!
//! A row marked `memoized` came in below `Leaf`. That component takes no
//! `children`, so its props compare equal and dioxus skips the re-render
//! entirely - the number is the parent's diff, not the component's render.
//! Price those with a first render instead.

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;
use libero::components::{Option, Title};
use libero::{LiberoProvider, components::*};

/// A shape to measure: what to call it, and the app that renders
/// [`CHILDREN`] of it.
type Shape = (&'static str, fn() -> Element);

/// Children per app. Big enough that per-render fixed costs disappear into the
/// per-item average.
const CHILDREN: usize = 200;
const ROUNDS: usize = 80;

/// One [`Shape`] per entry: a label, and the `rsx!` body to repeat.
macro_rules! shapes {
    ($($label:literal { $($item:tt)* })*) => {
        &[$((
            $label,
            {
                fn app() -> Element {
                    rsx! {
                        LiberoProvider {
                            div { for _ in 0..CHILDREN { $($item)* } }
                        }
                    }
                }
                app as fn() -> Element
            },
        )),*]
    };
}

/// The cheapest component that can exist: one scope, one element, no styling.
#[component]
fn Leaf(children: Element) -> Element {
    rsx! { span { {children} } }
}

/// Nanoseconds to re-render one instance of each shape.
///
/// The variants are interleaved and the *minimum* per variant is kept: a
/// sequential best-of-N drifts enough between variants to invert small
/// differences.
fn measure(shapes: &[Shape]) -> Vec<(&'static str, f64)> {
    let mut doms: Vec<_> = shapes
        .iter()
        .map(|(name, app)| {
            let mut dom = VirtualDom::new(*app);
            dom.rebuild(&mut NoOpMutations);
            (*name, dom, u64::MAX)
        })
        .collect();

    for _ in 0..ROUNDS {
        for (_, dom, best) in doms.iter_mut() {
            dom.mark_dirty(ScopeId::APP);
            let started = std::time::Instant::now();
            dom.render_immediate(&mut NoOpMutations);
            *best = (*best).min(started.elapsed().as_nanos() as u64);
        }
    }

    doms.into_iter()
        .map(|(name, _, best)| (name, best as f64 / CHILDREN as f64))
        .collect()
}

#[test]
#[ignore = "a measurement; needs --release to mean anything"]
fn render_cost_per_component() {
    let measured = measure(shapes! {
        // Controls. Everything below is read as a multiple of `Leaf`.
        "span" { span { "x" } }
        "Leaf" { Leaf { "x" } }

        "Box" { Box { "x" } }
        "Flex" { Flex { "x" } }
        "GridZone" { GridZone { GridItem { "x" } } }
        "Center" { Center { "x" } }
        "Container" { Container { "x" } }
        "AspectRatio" { AspectRatio { "x" } }
        "Divider" { Divider {} }
        "Float" { Float { "x" } }
        "Header" { Header { "x" } }
        "ScrollArea" { ScrollArea { "x" } }
        "Sidebar" { Sidebar { "x" } }
        "Splitter" { Splitter { initial_size: 50.0, panel_a: rsx! { div { "l" } }, panel_b: rsx! { div { "r" } } } }

        "Text" { Text { "x" } }
        "Title" { Title { "x" } }
        "Kbd" { Kbd { "x" } }
        "Mark" { Mark { "x" } }
        "Code" { Code { source: "let x = 1;" } }
        "CodeBlock" { CodeBlock { source: "let x = 1;" } }

        "Button" { Button { "x" } }
        "ActionIcon" { ActionIcon { aria_label: "a", "x" } }
        "Select" { Select { value: "a", Option { value: "a", "x" } } }
        "Chip" { Chip { checked: true, onchange: move |_| {}, "x" } }
        "Switch" { Switch { checked: true, onchange: move |_| {}, "x" } }
        "Slider" { Slider { value: 50.0, on_change: move |_| {} } }
        "ToggleButtonGroup" { ToggleButtonGroup { ToggleButton { value: "a", "x" } } }

        "Icon" { Icon { "x" } }
        "Image" { Image { src: "/x.png" } }
        "QrCode" { QrCode { data: "x", aria_label: "a" } }
        "List" { List { ListItem { "x" } } }
        "DataList" { DataList { DataListItem { label: rsx! { "l" }, "x" } } }
        "Table" { Table { data: vec![1u32], columns: vec![column("N").value(|n: &u32| *n)] } }

        "Anchor" { Anchor { to: "https://example.com", "x" } }
        "NavLink" { NavLink { to: "https://example.com", "x" } }
        "Tree" { Tree { aria_label: "a", data: vec![TreeNode::new("a", "Alpha".to_string())] } }
        "TreeItem" { TreeItem { "x" } }

        "Overlay" { Overlay {} }
        "Modal" { Modal { "x" } }
        "Drawer" { Drawer { "x" } }
        "Dialog" { Dialog { "x" } }
        "Tooltip" { Tooltip { label: rsx! { "t" }, "x" } }

        "FocusTrap" { FocusTrap { "x" } }
        "VisuallyHidden" { VisuallyHidden { "x" } }
    });

    let cost = |wanted: &str| {
        measured
            .iter()
            .find(|(name, _)| *name == wanted)
            .unwrap_or_else(|| panic!("{wanted} was not measured"))
            .1
    };
    let leaf = cost("Leaf");

    println!();
    for (name, ns) in &measured {
        let memoized = if *ns < leaf && *name != "span" {
            "  memoized"
        } else {
            ""
        };
        println!("{name:<16} {ns:>8.0} {:>6.2}x{memoized}", ns / leaf);
    }
    println!();

    if cfg!(debug_assertions) {
        println!("debug build - numbers are not comparable to the baseline");
        return;
    }

    // Tripwires, not targets. They catch a component growing a scope or an
    // uncached per-render build, and stay quiet for ordinary drift.
    for (name, ceiling) in [("Box", 3.0), ("Text", 5.0), ("Button", 6.0)] {
        assert!(
            cost(name) < leaf * ceiling,
            "{name} regressed: {:.0} ns, {:.1}x Leaf",
            cost(name),
            cost(name) / leaf
        );
    }
}
