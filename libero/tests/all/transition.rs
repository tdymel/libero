//! `Transition`'s rendered contract: the from-state markup, the per-kind transform,
//! and the reduced-motion guard at a specificity that can win.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Transition, TransitionKind},
};

fn open_app() -> Element {
    rsx! {
        LiberoProvider {
            Transition { open: true, "panel body" }
        }
    }
}

fn closed_app() -> Element {
    rsx! {
        LiberoProvider {
            Transition { open: false, "panel body" }
        }
    }
}

fn omitted_app() -> Element {
    rsx! {
        LiberoProvider {
            Transition { kind: TransitionKind::FadeUp, duration: 350, "panel body" }
        }
    }
}

fn root_class(html: &str) -> String {
    attributes_of(html, "div")["class"]
        .split_whitespace()
        .next()
        .expect("the root carries a framework class")
        .to_string()
}

#[test]
fn an_initially_open_transition_renders_open_children() {
    let html = render(open_app);

    assert!(body(&html).contains("panel body"), "{}", body(&html));
    assert_eq!(attributes_of(&html, "div")["data-state"], "open");
}

/// Closed from the start: nothing mounted, and no exit to wait out.
#[test]
fn an_initially_closed_transition_renders_no_children() {
    let html = render(closed_app);

    assert!(!body(&html).contains("panel body"), "{}", body(&html));
    assert_eq!(attributes_of(&html, "div")["data-state"], "closed");
}

/// The server sends the from-state, so the client's first paint is the one to animate from.
#[test]
fn an_omitted_open_renders_the_children_in_the_closed_state() {
    let html = render(omitted_app);

    assert!(body(&html).contains("panel body"), "{}", body(&html));
    assert_eq!(attributes_of(&html, "div")["data-state"], "closed");
}

#[test]
fn the_kind_and_duration_set_instance_variables() {
    let html = render(omitted_app);
    let style = &attributes_of(&html, "div")["style"];

    assert!(
        style.contains("--lsx-transition-duration-override:350ms;"),
        "{style}"
    );
    assert!(
        style.contains("--lsx-transition-from:translateY(var(--lsx-transition-distance));"),
        "{style}"
    );
}

/// `visibility` leaves closed content out of the tab order and the tree; the delay
/// keeps it announced for the whole exit.
#[test]
fn closed_content_hides_from_the_accessibility_tree_as_the_exit_ends() {
    let html = render(open_app);
    let class = root_class(&html);

    assert!(
        html.contains(&format!(
            ".{class}[data-state~=\"closed\"]{{opacity:0;transform:var(--lsx-transition-from);visibility:hidden;"
        )),
        "{html}"
    );
    assert!(
        html.contains("visibility 0s linear var(--lsx-transition-duration-override,"),
        "{html}"
    );
    assert!(html.contains("visibility 0s linear 0s;"), "{html}");
}

/// A `@media` block adds no specificity, so the guard carries the state attribute too.
#[test]
fn the_reduced_motion_guard_is_nested_inside_each_state() {
    let html = render(open_app);
    let class = root_class(&html);

    for state in ["open", "closed"] {
        assert!(
            html.contains(&format!(
                "@media (prefers-reduced-motion: reduce){{.{class}[data-state~=\"{state}\"]{{transition:none;}}}}"
            )),
            "{html}"
        );
    }
    assert_eq!(
        html.matches("@media (prefers-reduced-motion: reduce)")
            .count(),
        2,
        "{html}"
    );
}
