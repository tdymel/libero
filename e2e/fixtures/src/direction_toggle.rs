//! `DirectionToggle` beside text, so a turn has something to reorder, and a
//! button that clears the choice.

use dioxus::prelude::*;
use libero::components::{Button, DirectionToggle, Flex, Text};
use libero::hooks::use_direction;

use crate::Routes;

pub const ROUTES: Routes = &[("/direction-toggle", || rsx! { DirectionTogglePage {} })];

#[component]
fn DirectionTogglePage() -> Element {
    let direction = use_direction();
    let kept = direction.kept().map_or("none", |kept| kept.as_str());
    rsx! {
        Flex { direction: "row", gap: "md", align: "center",
            DirectionToggle { id: "direction" }
            Text { id: "page-text", "Page text" }
            Button { id: "clear", onclick: move |_| direction.clear(), "Clear" }
            Text { id: "kept", "{kept}" }
        }
    }
}
