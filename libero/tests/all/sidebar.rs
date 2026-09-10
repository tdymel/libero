use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::Sidebar,
    theme::{SidebarDefaults, Size, Theme},
};

#[test]
fn a_sidebar_renders_in_place() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { "sidebar content" }
            }
        }
    }

    let html = render(app);

    assert!(body(&html).contains("sidebar content"));
}

#[test]
fn a_sidebar_names_its_side_in_data_state() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { side: "right", "sidebar content" }
            }
        }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];
    assert!(state.contains("side-right"), "got {state}");
}

static WIDE_SIDEBARS: Theme = Theme {
    sidebar: SidebarDefaults {
        size: Size::Lg,
        ..Theme::DEFAULT.sidebar
    },
    ..Theme::DEFAULT
};

#[test]
fn an_unsized_sidebar_takes_the_themes_default_size() {
    fn app() -> Element {
        rsx! { LiberoProvider { themes: &WIDE_SIDEBARS, Sidebar { "sidebar content" } } }
    }

    let html = render(app);

    let state = &attributes_of(&html, "div")["data-state"];
    assert!(state.contains("size-lg"), "got {state}");
}
