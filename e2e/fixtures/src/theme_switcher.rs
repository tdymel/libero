//! `ThemeSwitcher`, the toggle alone.

use dioxus::prelude::*;
use libero::components::{Flex, MenuPart, Parts, Text, ThemeSwitcher};
use libero::sx::sx;
use libero::theme::ThemeSet;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/theme-switcher", || rsx! { ThemeSwitcherPage {} }),
    ("/theme-switcher/themes", || rsx! { ThemesPage {} }),
    ("/theme-switcher/system", || rsx! { SystemPage {} }),
];

/// The toggle with the system entry in its cycle.
#[component]
fn SystemPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ThemeSwitcher { id: "scheme", with_system: true }
            Text { id: "page-text", "Page text" }
        }
    }
}

/// The split button: the toggle and the theme picker's chevron. `menu_parts`
/// reaches the portaled menu's labels.
#[component]
fn ThemesPage() -> Element {
    rsx! {
        ThemeSwitcher {
            id: "split",
            themes: ThemeSet::CATALOGUE,
            menu_parts: Parts::new().part(MenuPart::Label, sx().font_style("italic")),
        }
    }
}

/// Text beside the toggle, so a scheme change has something to recolour.
#[component]
fn ThemeSwitcherPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ThemeSwitcher { id: "scheme" }
            Text { id: "page-text", "Page text" }
        }
    }
}
