//! `ThemeToggle`, the toggle alone.

use dioxus::prelude::*;
use libero::components::{Flex, Text, ThemeToggle};
use libero::theme::ThemeSet;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/theme-toggle", || rsx! { ThemeTogglePage {} }),
    ("/theme-toggle/themes", || rsx! { ThemesPage {} }),
    ("/theme-toggle/system", || rsx! { SystemPage {} }),
];

/// The toggle with the system entry in its cycle.
#[component]
fn SystemPage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            ThemeToggle { id: "scheme", with_system: true }
            Text { id: "page-text", "Page text" }
        }
    }
}

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
