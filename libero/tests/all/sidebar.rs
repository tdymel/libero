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
                Sidebar { side: "end", "sidebar content" }
            }
        }
    }

    let html = render(app);

    let state = &attributes_of(&html, "aside")["data-state"];
    assert!(state.contains("side-end"), "got {state}");
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

    let state = &attributes_of(&html, "aside")["data-state"];
    assert!(state.contains("size-lg"), "got {state}");
}

#[test]
fn a_sidebar_is_an_aside_unless_told_otherwise() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sidebar { id: "rail", "rail" }
                Sidebar { id: "menu", component: "nav", "menu" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "aside")["id"], "rail");
    assert_eq!(attributes_of(&html, "nav")["id"], "menu");
}

#[test]
fn a_blank_name_is_dropped_from_the_panel_and_its_scroll_tab_stop() {
    fn blank() -> Element {
        rsx! { LiberoProvider { Sidebar { aria_label: " ", "content" } } }
    }
    fn named() -> Element {
        rsx! { LiberoProvider { Sidebar { aria_label: "Filters", "content" } } }
    }

    assert!(!render(blank).contains("aria-label"));
    assert_eq!(
        attributes_of(&render(named), "aside")["aria-label"],
        "Filters"
    );
}
