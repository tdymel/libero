//! CSS transitions run natively, but Blitz sends no `transitionend`: an exit
//! ends on `use_presence`'s own timer (todo 468 N6).

use std::time::Duration;

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Collapse, Transition, TransitionKind};

fn collapse_app() -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        button { id: "toggle", onclick: move |_| open.toggle(), "Toggle" }
        Collapse { open: open(), keep_mounted: false, div { id: "content", "Details" } }
    }
}

#[test]
fn a_closed_collapse_unmounts_its_content_on_the_exit_timer() {
    let mut page = mount(collapse_app);
    assert!(page.exists("#content"));

    page.click("#toggle");
    page.advance(2.0);
    page.wait(Duration::from_secs(1));
    assert!(
        !page.exists("#content"),
        "the closed content is still mounted:\n{}",
        page.tree()
    );
}

fn transition_app() -> Element {
    let mut open = use_signal(|| true);
    rsx! {
        button { id: "toggle", onclick: move |_| open.toggle(), "Toggle" }
        Transition { open: open(), div { id: "content", "Details" } }
    }
}

#[test]
fn a_closed_transition_unmounts_its_content_on_the_exit_timer() {
    let mut page = mount(transition_app);
    assert!(page.exists("#content"));

    page.click("#toggle");
    page.advance(2.0);
    page.wait(Duration::from_secs(1));
    assert!(
        !page.exists("#content"),
        "the closed content is still mounted:\n{}",
        page.tree()
    );
}

fn corner_app() -> Element {
    rsx! {
        Transition { id: "corner", kind: TransitionKind::PopTopLeft, div { "Details" } }
        Transition { id: "skew", kind: TransitionKind::SkewUp, open: false, div { "Details" } }
    }
}

/// Blitz folds `transform-origin` and `skew` into the painted matrix, so these kinds run natively.
#[test]
fn the_origin_and_skew_kinds_resolve_natively() {
    let mut page = mount(corner_app);
    page.advance(1.0);
    assert_eq!(page.computed("#corner", "transform-origin"), "0% 0% 0px");
    let skew = page.computed("#skew", "transform");
    assert!(skew.contains("skew("), "{skew}");
}

fn appear_app() -> Element {
    rsx! {
        Transition { id: "appear", kind: TransitionKind::FadeUp, duration: 1000, div { "Details" } }
    }
}

/// An omitted `open` enters by the `appear` keyframe; Blitz runs it and resolves its `var()`.
#[test]
fn an_omitted_open_appears_by_its_keyframe_natively() {
    let mut page = mount(appear_app);
    page.advance(0.3);
    let opacity: f32 = page.computed("#appear", "opacity").parse().unwrap();
    assert!(opacity < 0.9, "opacity {opacity} 300ms into a 1s entrance");
    let transform = page.computed("#appear", "transform");
    assert_ne!(transform, "none", "transform 300ms into a 1s entrance");
    // Frame-sized steps: one long `advance` left the keyframe at its first sample.
    for _ in 0..20 {
        page.advance(0.1);
    }
    assert_eq!(page.computed("#appear", "opacity"), "1");
    assert_eq!(page.computed("#appear", "transform"), "none");
}

fn from_app() -> Element {
    let mut open = use_signal(|| false);
    rsx! {
        button { id: "toggle", onclick: move |_| open.toggle(), "Toggle" }
        Transition { id: "from", open: open(), from: libero::sx::sx().filter("blur(4px)"),
            div { "Details" }
        }
    }
}

/// A caller's from-state applies closed and clears once open.
#[test]
fn a_from_state_applies_natively() {
    let mut page = mount(from_app);
    page.advance(0.1);
    assert_eq!(page.computed("#from", "filter"), "blur(4px)");
    page.click("#toggle");
    page.advance(2.0);
    assert_eq!(page.computed("#from", "filter"), "none");
}

static ENDED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn fading_app() -> Element {
    let mut faded = use_signal(|| false);
    rsx! {
        button { id: "toggle", onclick: move |_| faded.toggle(), "Toggle" }
        div {
            id: "box",
            style: if faded() { "opacity: 0; transition: opacity 0.1s" } else { "opacity: 1; transition: opacity 0.1s" },
            ontransitionend: |_| {
                ENDED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
            "Box"
        }
    }
}

#[test]
#[ignore = "needs Blitz: it runs the transition but dispatches no transitionend"]
fn a_finished_transition_fires_transitionend() {
    let mut page = mount(fading_app);
    page.click("#toggle");
    for _ in 0..5 {
        page.advance(0.05);
    }
    assert_eq!(page.computed("#box", "opacity"), "0");
    assert_eq!(ENDED.load(std::sync::atomic::Ordering::SeqCst), 1);
}
