//! `DirectionToggle` beside text, so a turn has something to reorder.

use dioxus::prelude::*;
use libero::components::{DirectionToggle, Flex, Text};

use crate::Routes;

pub const ROUTES: Routes = &[("/direction-toggle", || rsx! { DirectionTogglePage {} })];

#[component]
fn DirectionTogglePage() -> Element {
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            DirectionToggle { id: "direction" }
            Text { id: "page-text", "Page text" }
        }
    }
}
