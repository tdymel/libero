//! `ThemeToggle`, the toggle alone.

use dioxus::prelude::*;
use libero::components::{Flex, Text, ThemeToggle};
use libero::theme::ThemeSet;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/theme-toggle", || rsx! { ThemeTogglePage {} }),
    ("/theme-toggle/themes", || rsx! { ThemesPage {} }),
];

/// The split button: the toggle and the theme picker's chevron.
#[component]
fn ThemesPage() -> Element {
    rsx! {
        ThemeToggle { id: "split", themes: ThemeSet::CATALOGUE }
    }
}

/// Text beside the toggle, so a scheme change has something to recolour.
#[component]
fn ThemeTogglePage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ThemeToggle { id: "scheme" }
            Text { id: "page-text", "Page text" }
        }
    }
}
