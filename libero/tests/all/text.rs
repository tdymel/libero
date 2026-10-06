use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Text,
    theme::{Size, TextDefaults, Theme},
};

#[test]
fn text_carries_its_size_as_a_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Text { size: Size::Lg, "body copy" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "p")["data-state"], "size-lg");
    assert!(body(&html).contains("body copy"));
}

static SMALL_TEXT: Theme = Theme {
    text: TextDefaults {
        size: Size::Sm,
        ..Theme::DEFAULT.text
    },
    ..Theme::DEFAULT
};

#[test]
fn an_unsized_text_takes_the_themes_default_size() {
    fn app() -> Element {
        rsx! { LiberoProvider { themes: &SMALL_TEXT, Text { "body copy" } } }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "p")["data-state"], "size-sm");
}

/// A palette colour publishes its text shade, and the `colored` state has a rule reading it.
#[test]
fn a_colour_sets_the_colored_state_and_its_text_shade() {
    fn app() -> Element {
        rsx! { LiberoProvider { Text { color: "error", "body copy" } } }
    }

    let html = render(app);
    let p = attributes_of(&html, "p");

    assert_eq!(p["data-state"], "size-md colored");
    assert!(p["style"].contains("--lsx-text-color:"), "{}", p["style"]);
    assert!(
        html.contains("[data-state~=\"colored\"]{color:var(--lsx-text-color)"),
        "{html}"
    );
}

/// Under a gradient the colour is the first stop, not a plain colour beside it.
#[test]
fn a_gradient_replaces_the_plain_colour() {
    fn app() -> Element {
        rsx! { LiberoProvider { Text { color: "error", gradient: ("secondary", 45), "body copy" } } }
    }

    let html = render(app);
    let p = attributes_of(&html, "p");

    assert_eq!(p["data-state"], "size-md gradient");
    assert!(
        p["style"].contains("--lsx-gradient-from:"),
        "{}",
        p["style"]
    );
    assert!(!p["style"].contains("--lsx-text-color"), "{}", p["style"]);
}

/// A caller's `style` joins the gradient vars instead of replacing them.
#[test]
fn a_callers_style_is_merged_with_the_gradient() {
    fn app() -> Element {
        rsx! { LiberoProvider { Text { gradient: ("secondary", 45), style: "margin-top: 4px", "body copy" } } }
    }

    let style = attributes_of(&render(app), "p")["style"].clone();

    assert!(style.contains("--lsx-gradient-from:"), "{style}");
    assert!(style.contains("margin-top: 4px"), "{style}");
}

/// `component` swaps the element and keeps the look.
#[test]
fn component_swaps_the_tag() {
    fn app() -> Element {
        rsx! { LiberoProvider { Text { component: "span", size: Size::Lg, "body copy" } } }
    }

    let html = body(&render(app));

    assert!(!html.contains("<p"), "{html}");
    assert_eq!(attributes_of(&html, "span")["data-state"], "size-lg");
}
