//! `HoverCard`'s markup: nothing but the trigger while closed, a named dialog
//! on a `Paper` surface once open, and the trigger bare when disabled. Hover,
//! focus, the delays and the Tab bridge need a real renderer - `ElementApi` is
//! `Unsupported` here - and are verified in the browser instead.

use std::cell::Cell;

use crate::common::{attributes_of, body, render};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, HoverCard},
};

thread_local! {
    static OPENED: Cell<Option<bool>> = const { Cell::new(None) };
    static DISABLED: Cell<bool> = const { Cell::new(false) };
}

fn app() -> Element {
    rsx! {
        LiberoProvider {
            HoverCard {
                opened: OPENED.get(),
                disabled: DISABLED.get(),
                aria_label: "Author",
                content: rsx! { a { href: "/authors/ada", "Profile" } },
                Button { "Ada" }
            }
        }
    }
}

fn rendered(opened: Option<bool>, disabled: bool) -> String {
    OPENED.set(opened);
    DISABLED.set(disabled);
    render(app)
}

#[test]
fn closed_renders_only_the_trigger() {
    let html = body(&rendered(None, false));
    assert!(html.contains(">Ada</button>"), "{html}");
    assert!(!html.contains("role=\"dialog\""), "{html}");
    assert!(!html.contains("Profile"), "{html}");
}

#[test]
fn the_trigger_sits_in_a_wrapper_with_a_contents_bridge() {
    let html = body(&rendered(None, false));
    let wrapper = html.find("<span").expect("a wrapper span");
    let bridge = html[wrapper + 1..].find("<span").expect("a bridge span") + wrapper + 1;
    let button = html.find("<button").expect("the trigger");
    assert!(wrapper < bridge && bridge < button, "{html}");
    assert_eq!(
        attributes_of(&html[bridge..], "span")
            .get("style")
            .map(String::as_str),
        Some("display: contents"),
    );
}

#[test]
fn open_renders_a_named_dialog_on_a_paper_surface() {
    let html = rendered(Some(true), false);
    let markup = body(&html);
    let at = markup.find("role=\"dialog\"").expect("an open card");
    let card = &markup[markup[..at].rfind('<').unwrap()..];
    let attributes = attributes_of(card, "div");

    assert_eq!(
        attributes.get("aria-label").map(String::as_str),
        Some("Author")
    );
    let states = attributes.get("data-state").expect("the card's tokens");
    assert!(states.contains("radius-sm"), "{states}");
    assert!(states.contains("shadow-md"), "{states}");
    assert!(markup.contains("Profile"), "{markup}");

    // The surface is Paper's: its background token and the focus-ring
    // contrast that has to travel with it.
    assert!(
        html.contains("var(--lsx-paper-background)"),
        "no paper background"
    );
    assert!(
        html.contains("--lsx-focus-contrast:var(--lsx-paper-contrast)")
            || html.contains("--lsx-focus-contrast: var(--lsx-paper-contrast)")
    );
}

#[test]
fn forced_closed_renders_no_card() {
    let html = body(&rendered(Some(false), false));
    assert!(!html.contains("role=\"dialog\""), "{html}");
}

#[test]
fn disabled_renders_the_trigger_bare() {
    let html = body(&rendered(Some(true), true));
    assert!(!html.contains("<span"), "{html}");
    assert!(!html.contains("role=\"dialog\""), "{html}");
    assert!(html.contains(">Ada</button>"), "{html}");
}
