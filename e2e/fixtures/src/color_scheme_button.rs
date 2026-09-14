//! `ColorSchemeButton`, the toggle alone.

use dioxus::prelude::*;
use libero::components::{ColorSchemeButton, Flex, Text};
use libero::theme::ThemeSet;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/color-scheme-button", || rsx! { ColorSchemeButtonPage {} }),
    ("/color-scheme-button/themes", || rsx! { ThemesPage {} }),
];

/// The split button: the toggle and the theme picker's chevron.
#[component]
fn ThemesPage() -> Element {
    rsx! {
        ColorSchemeButton { id: "split", themes: ThemeSet::CATALOGUE }
    }
}

/// Text beside the toggle, so a scheme change has something to recolour.
#[component]
fn ColorSchemeButtonPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ColorSchemeButton { id: "scheme" }
            Text { id: "page-text", "Page text" }
        }
    }
}
