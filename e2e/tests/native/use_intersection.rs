//! `use_intersection` and `on_viewport_resize` on Blitz, which reports neither:
//! both are polled (todo 2758). Scrolling and roots are shared scenarios (`use_intersection::`).

use dioxus::prelude::*;
use e2e::native::{Page, VIEWPORT, mount};
use libero::{
    components::{Box, Button},
    hooks::{Align, IntersectionOptions, PopoverOptions, PopoverWidth, Side, use_element},
    hooks::{use_intersection, use_popover},
};

/// A target just below the window's foot.
fn below_the_fold() -> Element {
    let seen = use_intersection(IntersectionOptions {
        thresholds: vec![0.0, 1.0],
        ..Default::default()
    });
    let state = match *seen.entry.read() {
        None => "none".to_string(),
        Some(entry) if !entry.is_intersecting => "out".to_string(),
        Some(entry) => format!("{:.0}", entry.ratio * 100.0),
    };
    rsx! {
        p { id: "state", "{state}" }
        div { height: "{VIEWPORT.1 + 100}px" }
        div {
            id: "target",
            position: "absolute",
            top: "{VIEWPORT.1 + 20}px",
            width: "100px",
            height: "40px",
            onmounted: move |event| seen.on_mounted.call(event),
            ..seen.attributes,
        }
    }
}

#[test]
fn a_taller_window_brings_a_target_into_view() {
    let mut page = mount(below_the_fold);
    let state = |page: &Page| page.text("#state");
    assert!(
        page.wait_for(|page| state(page) == "out"),
        "first measure: {}",
        state(&page)
    );
    // Half of the target's 40px past its top.
    let top = page.rect("#target").1;
    page.resize(VIEWPORT.0, (top + 20.0) as u32);
    assert!(
        page.wait_for(|page| state(page) == "50"),
        "half in: {}, target {:?}",
        state(&page),
        page.rect("#target")
    );
    page.resize(VIEWPORT.0, (top + 60.0) as u32);
    assert!(
        page.wait_for(|page| state(page) == "100"),
        "all in: {}",
        state(&page)
    );
}

/// A scroller root, scrolled by focus as in the shared scenario.
fn rooted() -> Element {
    let scroller = use_element();
    let seen = use_intersection(IntersectionOptions {
        root: Some(scroller),
        thresholds: vec![0.0, 0.5, 1.0],
        ..Default::default()
    });
    let percent = seen
        .entry
        .read()
        .map_or(0, |entry| (entry.ratio * 100.0).round() as u32);
    rsx! {
        p { id: "state", "{percent}" }
        div {
            id: "scroller",
            onmounted: scroller.mount(),
            height: "100px",
            overflow: "auto",
            div { height: "300px" }
            div {
                id: "target",
                tabindex: "-1",
                height: "60px",
                onmounted: move |event| seen.on_mounted.call(event),
                ..seen.attributes,
            }
            div { height: "300px" }
        }
    }
}

#[test]
fn a_scrolled_root_brings_its_target_in() {
    let mut page = mount(rooted);
    page.advance(1.0);
    assert_eq!(page.text("#state"), "0");
    page.hover("#scroller");
    page.wheel("#scroller", 280.0);
    assert!(
        page.wait_for(|page| page.text("#state") == "100"),
        "{} at scroll {}, target {:?}, scroller {:?}",
        page.text("#state"),
        page.scroll_top("#scroller"),
        page.rect("#target"),
        page.rect("#scroller")
    );
}

#[test]
fn a_wheel_brings_a_target_into_view() {
    let mut page = mount(below_the_fold);
    let state = |page: &Page| page.text("#state");
    assert!(
        page.wait_for(|page| state(page) == "out"),
        "first measure: {}",
        state(&page)
    );
    page.wheel("#state", 200.0);
    assert!(
        page.wait_for(|page| state(page) == "100"),
        "scrolled in: {}",
        state(&page)
    );
}

/// A target in the window under a `display: none` box, shown by a click.
fn hidden() -> Element {
    let seen = use_intersection(IntersectionOptions {
        thresholds: vec![0.0, 1.0],
        ..Default::default()
    });
    let mut shown = use_signal(|| false);
    let state = match *seen.entry.read() {
        None => "none".to_string(),
        Some(entry) if !entry.is_intersecting => "out".to_string(),
        Some(entry) => format!("{:.0}", entry.ratio * 100.0),
    };
    rsx! {
        p { id: "state", "{state}" }
        button { id: "show", onclick: move |_| shown.set(true), "Show" }
        div { display: if shown() { "block" } else { "none" },
            div {
                id: "target",
                width: "100px",
                height: "40px",
                onmounted: move |event| seen.on_mounted.call(event),
                ..seen.attributes,
            }
        }
    }
}

/// Unstyled under `display: none`, the target reports no intersection (todo 2803).
#[test]
fn a_target_under_display_none_is_not_intersecting() {
    let mut page = mount(hidden);
    let state = |page: &Page| page.text("#state");
    assert!(
        page.wait_for(|page| state(page) == "out"),
        "hidden: {}",
        state(&page)
    );
    page.click("#show");
    assert!(
        page.wait_for(|page| state(page) == "100"),
        "shown: {}",
        state(&page)
    );
}

/// An anchor whose popover fits below it in the test window only.
fn low_anchor() -> Element {
    let anchor = use_element();
    let mut opened = use_signal(|| false);
    let options = PopoverOptions::new(8.0, 0.0)
        .side(Side::Bottom)
        .align(Align::Start)
        .width(PopoverWidth::Auto);
    let popover = use_popover(anchor, opened(), options);
    let floating = *popover.floating();
    popover.show(opened().then(|| {
        rsx! {
            Box {
                id: "floating",
                style: popover.style(),
                onmounted: floating.mount(),
                div { width: "150px", height: "100px" }
            }
        }
    }));
    rsx! {
        div { display: "flow-root",
            div { margin_top: "500px", width: "max-content",
                Button {
                    id: "anchor",
                    onmounted: anchor.mount(),
                    onclick: move |_| opened.toggle(),
                    "Anchor"
                }
            }
        }
    }
}

/// `use_popover` re-places on `on_viewport_resize`, which Blitz now polls.
#[test]
fn an_open_popover_flips_when_the_window_shrinks() {
    let mut page = mount(low_anchor);
    page.click("#anchor");
    let below =
        |page: &Page| page.exists("#floating") && page.rect("#floating").1 > page.rect("#anchor").1;
    assert!(
        page.wait_for(below),
        "not below its anchor: {}",
        page.tree()
    );
    page.resize(VIEWPORT.0, 600);
    let above = |page: &Page| page.rect("#floating").1 < page.rect("#anchor").1;
    assert!(
        page.wait_for(above),
        "still below after a shrink: floating {:?}, anchor {:?}",
        page.rect("#floating"),
        page.rect("#anchor")
    );
}
