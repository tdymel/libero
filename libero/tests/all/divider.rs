use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Divider,
    theme::{Color, DividerDefaults, Size, Theme},
};

#[test]
fn divider_renders_as_a_separator() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { color: Color::Muted }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "div");

    assert_eq!(attributes["role"], "separator", "{html}");
    assert!(attributes["style"].contains("--lsx-divider-color"));
}

#[test]
fn a_dividers_size_reaches_its_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { size: Size::Lg }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "div")["data-state"].contains("size-lg"));
}

/// `aria-orientation` is not allowed on `none` (axe `aria-allowed-attr`).
#[test]
fn a_decorative_vertical_divider_has_no_orientation() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { orientation: "vertical", role: "none" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "div");

    assert_eq!(attributes["role"], "none", "{html}");
    assert!(!attributes.contains_key("aria-orientation"), "{html}");
}

/// The caller's `role: "separator"` is the default role, so the label still names it.
#[test]
fn a_callers_separator_role_keeps_the_label_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { orientation: "vertical", role: "separator", "Advanced" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "div");

    assert!(attributes.contains_key("aria-labelledby"), "{html}");
    assert_eq!(attributes["aria-orientation"], "vertical", "{html}");
}

#[test]
fn a_callers_aria_label_replaces_the_label_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Divider { aria_label: "Billing", "Or" }
            }
        }
    }

    let html = render(app);
    let attributes = attributes_of(&html, "div");

    assert_eq!(attributes["aria-label"], "Billing", "{html}");
    assert!(!attributes.contains_key("aria-labelledby"), "{html}");
}

static THICK_DIVIDERS: Theme = Theme {
    divider: DividerDefaults {
        size: Size::Lg,
        ..Theme::DEFAULT.divider
    },
    ..Theme::DEFAULT
};

#[test]
fn an_unsized_divider_takes_the_themes_default_size() {
    fn app() -> Element {
        rsx! { LiberoProvider { themes: &THICK_DIVIDERS, Divider {} } }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];
    assert!(state.contains("size-lg"), "{state}");
}
