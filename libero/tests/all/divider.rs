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
                Divider { color: Color::Grey }
            }
        }
    }

    let html = render(app);

    assert!(attributes_of(&html, "div")["style"].contains("--lsx-divider-color"));
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
        rsx! { LiberoProvider { theme: &THICK_DIVIDERS, Divider {} } }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];
    assert!(state.contains("size-lg"), "{state}");
}
