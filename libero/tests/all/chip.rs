use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Chip};

/// The hidden checkbox is `position: absolute`, so the chip has to be its
/// containing block - the same defect `SegmentedControl` had, where a focused
/// input laid out against the viewport scrolls the whole document.
#[test]
fn a_selectable_chip_contains_its_hidden_checkbox() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { checked: true, onchange: move |_| {}, "tag" }
            }
        }
    }

    let html = render(app);
    let input = classes_of(&html, "input");
    assert!(
        input
            .iter()
            .any(|class| html.contains(&format!(".{class}{{position:absolute"))),
        "{input:?}"
    );

    let positioned = classes_of(&html, "span").iter().any(|class| {
        let rule = format!(".{class}{{");
        html.find(&rule).is_some_and(|at| {
            let base = &html[at..];
            base[..base.find('}').unwrap()].contains("position:relative")
        })
    });
    assert!(positioned, "no positioned class on the chip root");
}

#[test]
fn a_selectable_chip_renders_a_checkbox_its_label_points_at() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { checked: true, onchange: move |_| {}, "tag" }
            }
        }
    }

    let html = render(app);
    let input = attributes_of(&html, "input");

    assert_eq!(input["type"], "checkbox");
    assert!(html.contains("checked=true"));
    // The label points at the input, so clicking the text toggles it.
    assert_eq!(attributes_of(&html, "label")["for"], input["id"]);

    let span = attributes_of(&html, "span");
    assert_eq!(
        span["data-state"],
        "filled size-md radius-xl checked selectable"
    );
    // M3's selected filter chip is a tonal container, and one step past
    // `Tonal`'s own resting tint - so selecting an already-tonal chip still
    // reads as a change.
    assert!(span["style"].contains("--lsx-chip-container:var(--lsx-primary-fill-2);"));
    assert!(span["style"].contains("--lsx-chip-on-container:var(--lsx-primary-contrast-2);"));
    assert!(body(&html).contains(">tag<"));
}

#[test]
fn a_clickable_chip_is_a_button_and_a_linked_one_an_anchor() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { onclick: move |_| {}, "act" }
                Chip { to: "https://example.com", target: "_blank", "go" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "button")["type"], "button");
    assert_eq!(attributes_of(&html, "a")["href"], "https://example.com");
    assert_eq!(attributes_of(&html, "a")["target"], "_blank");
    // The pointer state both roots need, and no `<span>` root gets.
    assert!(attributes_of(&html, "button")["data-state"].contains("clickable"));
}

/// A row of filter chips shares one `name`, so each has to post its own
/// `value` (todo 20). Without one the attribute is absent and the browser
/// posts its default `on`, exactly as `Checkbox` does.
#[test]
fn a_chip_posts_its_value_under_the_shared_name_or_falls_back_to_on() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { name: "tags", value: "rust", "Rust" }
            }
        }
    }

    let html = render(app);
    let input = attributes_of(&html, "input");
    assert_eq!(input["name"], "tags");
    assert_eq!(input["value"], "rust");

    fn valueless() -> Element {
        rsx! {
            LiberoProvider {
                Chip { name: "agreed", "Agreed" }
            }
        }
    }

    let html = render(valueless);
    let input = attributes_of(&html, "input");
    assert_eq!(input["name"], "agreed");
    assert!(!input.contains_key("value"), "{input:?}");
}
