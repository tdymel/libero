//! `GridZone` on its own: two half-width items and a named gap.

use dioxus::prelude::*;
use libero::components::{GridItem, GridSpan, GridZone};

use crate::Routes;

pub const ROUTES: Routes = &[("/grid-zone", || rsx! { GridZonePage {} })];

#[component]
fn GridZonePage() -> Element {
    rsx! {
        div { style: "width: 400px",
            GridZone { id: "zone", gap: "lg",
                GridItem { id: "a", span: GridSpan::Half, "A" }
                GridItem { id: "b", span: GridSpan::Half, "B" }
            }
        }
    }
}
