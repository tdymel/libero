//! A `Paper` with a `color` (so an inline `style`) and style attributes of its own.

use dioxus::prelude::*;
use libero::components::Paper;

use crate::Routes;

pub const ROUTES: Routes = &[("/polymorphic", || rsx! { PolymorphicPage {} })];

#[component]
fn PolymorphicPage() -> Element {
    rsx! {
        Paper {
            id: "sized",
            color: "#123456",
            position: "relative",
            left: "5px",
            height: "40px",
        }
    }
}
