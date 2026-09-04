//! `Scroller`'s rendered contract: a named, focusable region between two
//! step controls, neither of which offers to go anywhere until the strip has
//! been measured.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Scroller};

/// The open tag of every `<button>`, in order.
fn buttons(html: &str) -> Vec<String> {
    html.match_indices("<button")
        .map(|(start, _)| html[start..][..=html[start..].find('>').unwrap()].to_string())
        .collect()
}

/// The region is the tab stop, so a strip of plain text or images can be
/// reached and arrowed through - and a tab stop has to be named.
#[test]
fn the_strip_is_a_named_focusable_region() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", span { "one" } }
            }
        }
    }

    let html = body(&render(app));
    let region = html.find(r#"role="region""#).expect("a region");
    let open = html[..region].rfind("<div").unwrap();
    let region = attributes_of(&html[open..], "div");

    assert_eq!(region["aria-label"], "Tags", "{region:?}");
    assert_eq!(region["tabindex"], "0", "{region:?}");
    assert!(region.contains_key("id"), "{region:?}");
}

/// Before anything is measured nothing is known to overflow, so both controls
/// sit at their edge: `aria-disabled` and out of the tab order, rather than
/// `disabled`, which would drop focus from a control pressed to its end. Each
/// names the region it scrolls.
#[test]
fn both_controls_start_at_their_edge() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", span { "one" } }
            }
        }
    }

    let html = body(&render(app));
    let buttons = buttons(&html);
    assert_eq!(buttons.len(), 2, "{html}");

    let region = html.find(r#"role="region""#).unwrap();
    let open = html[..region].rfind("<div").unwrap();
    let region_id = attributes_of(&html[open..], "div")["id"].clone();

    for (button, label) in buttons.iter().zip(["Scroll backward", "Scroll forward"]) {
        let attributes = attributes_of(button, "button");
        assert_eq!(attributes["type"], "button", "{attributes:?}");
        assert_eq!(attributes["aria-label"], label, "{attributes:?}");
        assert_eq!(attributes["aria-disabled"], "true", "{attributes:?}");
        assert_eq!(attributes["tabindex"], "-1", "{attributes:?}");
        assert_eq!(attributes["aria-controls"], region_id, "{attributes:?}");
        assert!(!attributes.contains_key("disabled"), "{attributes:?}");
    }
}

/// Start control, region, end control: the reading and tab order follows
/// the strip.
#[test]
fn the_controls_flank_the_region() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", span { "one" } }
            }
        }
    }

    let html = body(&render(app));
    let backward = html.find("Scroll backward").unwrap();
    let region = html.find(r#"role="region""#).unwrap();
    let forward = html.find("Scroll forward").unwrap();

    assert!(backward < region && region < forward, "{html}");
}

#[test]
fn never_renders_no_controls() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", controls: "never", span { "one" } }
            }
        }
    }

    let html = body(&render(app));
    assert!(buttons(&html).is_empty(), "{html}");
    assert!(html.contains("controls-never"), "{html}");
}

/// The size and the controls mode go on the root as tokens.
#[test]
fn the_root_carries_size_and_mode() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", control_size: "lg", controls: "always", span { "one" } }
            }
        }
    }

    let html = body(&render(app));
    assert!(
        html.contains(r#"data-state="size-lg controls-always""#),
        "{html}"
    );
}

/// A `fade_color` writes the `-override` twin, which the controls read ahead
/// of the themed surface colour. With none, nothing is written and the paper
/// background applies.
#[test]
fn a_fade_color_overrides_the_surface() {
    fn with_fade() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", fade_color: "grey.1", span { "one" } }
            }
        }
    }
    fn without() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", span { "one" } }
            }
        }
    }

    let html = render(with_fade);
    assert!(
        body(&html).contains("--lsx-scroller-fade-override:var(--lsx-grey-1)"),
        "{html}"
    );
    assert!(
        html.contains("var(--lsx-scroller-fade-override, var(--lsx-scroller-fade))"),
        "the controls read the twin: {html}"
    );
    assert!(!body(&render(without)).contains("--lsx-scroller-fade"));
}

/// The themed defaults reach `:root`, the fade as the paper surface.
#[test]
fn the_theme_declares_the_fade_and_the_control_sizes() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Scroller { aria_label: "Tags", span { "one" } }
            }
        }
    }

    let html = render(app);
    assert!(
        html.contains("--lsx-scroller-fade:var(--lsx-paper-background)"),
        "{html}"
    );
    assert!(
        html.contains("--lsx-scroller-control-size-md:40px"),
        "{html}"
    );
}
