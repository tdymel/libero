use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, SvgData},
};

const BAR: SvgData = SvgData::new(r#"<svg viewBox="0 0 16 16"><path d="M2 8h12"/></svg>"#);

/// Todo 1094: `icon` draws a `Pictogram` and wins over `children`.
#[test]
fn an_icon_is_drawn_instead_of_the_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Close", icon: BAR }
                ActionIcon { aria_label: "Both", icon: BAR, "children" }
            }
        }
    }

    let html = render(app);
    assert_eq!(html.matches(r#"<path d="M2 8h12"/>"#).count(), 2, "{html}");
    assert!(!html.contains("children"), "{html}");
}

#[test]
fn action_icon_labels_itself_for_assistive_technology() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Close", "×" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert_eq!(attributes["aria-label"], "Close");
    assert_eq!(attributes["type"], "button");
}

/// Todo 222: `ActionIcon` takes `Button`'s `selected` - `aria-pressed` both
/// ways, the `checked` token while pressed, and nothing on a plain action.
#[test]
fn a_selected_action_icon_says_pressed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Bold", variant: "standard", selected: true, "B" }
                ActionIcon { aria_label: "Italic", variant: "standard", selected: false, "I" }
                ActionIcon { aria_label: "Save", "S" }
            }
        }
    }

    let html = render(app);
    let tags: Vec<&str> = html
        .match_indices("<button")
        .map(|(at, _)| &html[at..at + html[at..].find('>').unwrap()])
        .collect();
    assert_eq!(tags.len(), 3, "{html}");
    // The `checked` token itself, not any `checked` substring: an
    // `aria-checked` or a class fragment would match that too.
    let checked = |tag: &str| {
        tag.split(r#"data-state=""#).nth(1).is_some_and(|rest| {
            rest[..rest.find('"').unwrap()]
                .split(' ')
                .any(|token| token == "checked")
        })
    };
    assert!(tags[0].contains(r#"aria-pressed="true""#), "{}", tags[0]);
    assert!(checked(tags[0]), "{}", tags[0]);
    assert!(tags[1].contains(r#"aria-pressed="false""#), "{}", tags[1]);
    assert!(!checked(tags[1]), "{}", tags[1]);
    assert!(!tags[2].contains("aria-pressed"), "{}", tags[2]);
}

/// Todo 869: `focusable_when_disabled` keeps the tab stop - `aria-disabled`
/// rather than `disabled` - and the disabled look.
#[test]
fn a_focusable_disabled_action_icon_stays_in_the_tab_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ActionIcon { aria_label: "Next", disabled: true, focusable_when_disabled: true, ">" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "button");

    assert!(!attributes.contains_key("disabled"), "{html}");
    assert_eq!(attributes["aria-disabled"], "true");
    assert!(
        attributes["data-state"]
            .split(' ')
            .any(|token| token == "disabled"),
        "{html}"
    );
}
