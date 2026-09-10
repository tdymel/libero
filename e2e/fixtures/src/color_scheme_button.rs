//! `ColorSchemeButton`, the toggle alone.

use dioxus::prelude::*;
use libero::components::{ColorSchemeButton, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/color-scheme-button", || rsx! { ColorSchemeButtonPage {} })];

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
