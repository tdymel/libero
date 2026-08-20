//! What a component costs to re-render, in nanoseconds per instance.
//!
//! Ignored by default - it is a measurement, not an assertion, and the numbers
//! only mean anything in a release build:
//!
//! ```text
//! cargo test --release -p libero -- --ignored --nocapture
//! ```
//!
//! Baseline on the development machine, 2026-09-01:
//!
//! ```text
//! span      146    0.15x
//! Leaf      956    1.00x
//! Box     1,530    1.62x
//! Text    1,593    1.69x
//! Button  5,281    5.61x
//! ```
//!
//! `span` is a bare element with no scope; `Leaf` the cheapest possible
//! component. The ratios travel between machines, the absolutes do not - so
//! compare a change against a run of `main` on the same machine, not against
//! the table above.

use dioxus::dioxus_core::{NoOpMutations, ScopeId, VirtualDom};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Box, Button, Text},
};

/// A shape to measure: what to call it, and the app that renders
/// [`CHILDREN`] of it.
type Shape = (&'static str, fn() -> Element);

/// Children per app. Big enough that per-render fixed costs disappear into the
/// per-item average.
const CHILDREN: usize = 200;
const ROUNDS: usize = 80;

/// The cheapest component that can exist: one scope, one element, no styling.
#[component]
fn Leaf(children: Element) -> Element {
    rsx! { span { {children} } }
}

fn app_span() -> Element {
    rsx! { LiberoProvider { div { for _ in 0..CHILDREN { span { "x" } } } } }
}

fn app_leaf() -> Element {
    rsx! { LiberoProvider { div { for _ in 0..CHILDREN { Leaf { "x" } } } } }
}

fn app_box() -> Element {
    rsx! { LiberoProvider { div { for _ in 0..CHILDREN { Box { "x" } } } } }
}

fn app_text() -> Element {
    rsx! { LiberoProvider { div { for _ in 0..CHILDREN { Text { "x" } } } } }
}

fn app_button() -> Element {
    rsx! { LiberoProvider { div { for _ in 0..CHILDREN { Button { "x" } } } } }
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
    let measured = measure(&[
        ("span", app_span),
        ("Leaf", app_leaf),
        ("Box", app_box),
        ("Text", app_text),
        ("Button", app_button),
    ]);

    let leaf = measured
        .iter()
        .find(|(name, _)| *name == "Leaf")
        .expect("the Leaf baseline")
        .1;

    println!();
    for (name, ns) in &measured {
        println!("{name:<8} {ns:>8.0} {:>6.2}x", ns / leaf);
    }
    println!();

    if cfg!(debug_assertions) {
        println!("debug build - numbers are not comparable to the baseline");
        return;
    }

    // Tripwires, not targets. They catch a component growing a scope or an
    // uncached per-render build, and stay quiet for ordinary drift.
    let cost = |wanted: &str| {
        measured
            .iter()
            .find(|(name, _)| *name == wanted)
            .expect("a measured shape")
            .1
    };

    assert!(
        cost("Box") < leaf * 3.0,
        "Box regressed: {:.0} ns",
        cost("Box")
    );
    assert!(
        cost("Text") < leaf * 5.0,
        "Text regressed: {:.0} ns",
        cost("Text")
    );
}
