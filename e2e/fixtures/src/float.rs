//! `Float` against the viewport: a fixed badge at each of two corners, and an absolute
//! one inside a positioned card, which never takes the safe area.

use dioxus::prelude::*;
use libero::components::{Box, Float};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/float", || rsx! { FloatPage {} })];

#[component]
fn FloatPage() -> Element {
    rsx! {
        Float { id: "bottom-end", fixed: true, placement: "bottom-end", "Bottom end" }
        Float { id: "top-start", fixed: true, placement: "top-start", "Top start" }
        Box { id: "card", sx: sx().position("relative").height("120px").margin_top("200px"),
            Float { id: "inside", placement: "bottom-end", "Inside" }
        }
    }
}
