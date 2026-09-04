//! `Indicator`'s rendered contract: an `aria-hidden` `<span>` with no
//! positioning, a capped count, and a ping whose reduced-motion guard sits
//! where it can win.

use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Indicator};

fn indicator_class(html: &str) -> String {
    classes_of(&body(html), "span")
        .first()
        .expect("a class on the indicator")
        .clone()
}

/// A bare dot is an empty node, and a count is a truncation with no noun -
/// neither is worth announcing. The meaning lives on what it marks.
#[test]
fn a_bare_dot_is_hidden_empty_and_unlabelled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Indicator {} }
        }
    }

    let html = body(&render(app));
    let attributes = attributes_of(&html, "span");

    assert_eq!(attributes["aria-hidden"], "true", "{attributes:?}");
    assert!(!attributes.contains_key("role"), "{attributes:?}");
    assert!(html.contains(r#"data-state="size-md""#), "{html}");
    assert!(html.contains("></span>"), "no text in a bare dot: {html}");
}

/// `aria-hidden` goes on through `attr_default`, so a caller who wants the
/// indicator itself read out can say so.
#[test]
fn a_caller_can_override_aria_hidden() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Indicator { label: 3, aria_hidden: "false" } }
        }
    }

    let attributes = attributes_of(&body(&render(app)), "span");
    assert_eq!(attributes["aria-hidden"], "false", "{attributes:?}");
}

/// Over the theme's cap of 99 the label reads `99+`; a caller's `max` moves
/// the cap, and the cap itself is still printed as a number.
#[test]
fn a_count_over_max_is_capped() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Indicator { label: 128 }
                Indicator { label: 10, max: 9 }
                Indicator { label: 9, max: 9 }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains(">99+</span>"), "{html}");
    assert!(html.contains(">9+</span>"), "{html}");
    assert!(html.contains(">9</span>"), "{html}");
    assert!(
        html.contains(r#"data-state="size-md labelled""#),
        "a label turns on the padding: {html}"
    );
}

/// The ping is a `::before` at `inset: 0`, so the root must be its containing
/// block - otherwise it resolves against the `Float` around it and only looks
/// right while the dot is the `Float`'s sole unpadded child. And the root
/// positions nothing else: no corner, no offset, no z-index of its own.
#[test]
fn the_root_contains_the_ping_and_positions_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Indicator { processing: true } }
        }
    }

    let html = render(app);
    let class = indicator_class(&html);
    let base = html
        .split(&format!(".{class}{{"))
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("a base rule");

    assert!(base.contains("position:relative;"), "{base}");
    for property in [
        "top:",
        "right:",
        "bottom:",
        "left:",
        "transform:",
        "z-index:",
    ] {
        assert!(!base.contains(property), "{property} in {base}");
    }
}

/// A bare `@media` rule keyed on the `data-state` token would lose to the
/// per-instance `.cls[data-state~="processing"]::before` rule on specificity
/// whatever its source order. The guard has to be that same selector, inside
/// the media block.
#[test]
fn the_reduced_motion_guard_uses_the_ping_selector() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Indicator { processing: true } }
        }
    }

    let html = render(app);
    let class = indicator_class(&html);
    let selector = format!(r#".{class}[data-state~="processing"]::before"#);

    assert!(
        html.contains(&format!(
            "@media (prefers-reduced-motion: reduce){{{selector}{{animation:none;}}"
        )),
        "{html}"
    );
    assert!(
        html.contains("@keyframes lsx-indicator-processing{"),
        "the keyframes ride on the theme stylesheet: {html}"
    );
}

/// The ring is the surface colour, `Paper`'s token, and the fill resolves a
/// shade with its own contrast twin for the label.
#[test]
fn the_ring_reads_the_paper_token_and_the_fill_its_contrast_twin() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Indicator { with_border: true, color: "success", label: 4 } }
        }
    }

    let html = render(app);
    let class = indicator_class(&html);
    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="with-border"]{{box-shadow:0 0 0 var(--lsx-indicator-border-width) var(--lsx-paper-background);"#
        )),
        "{html}"
    );

    let style = attributes_of(&body(&html), "span")["style"].clone();
    assert!(
        style.contains("--lsx-indicator-color:var(--lsx-success-6);"),
        "{style}"
    );
    assert!(
        style.contains("--lsx-indicator-contrast:var(--lsx-success-contrast-6);"),
        "{style}"
    );
}

/// With no `color` the theme's error role fills the dot.
#[test]
fn the_default_fill_is_the_theme_colour() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Indicator {} }
        }
    }

    let style = attributes_of(&body(&render(app)), "span")["style"].clone();
    assert!(
        style.contains("--lsx-indicator-color:var(--lsx-error-6);"),
        "{style}"
    );
    assert!(
        !style.contains("--lsx-indicator-radius-override"),
        "{style}"
    );
}
