//! The sequential focus starting point: prose between buttons to click on.

use dioxus::prelude::*;
use libero::components::Button;

use crate::Routes;

pub const ROUTES: Routes = &[("/focus-start", || rsx! { FocusStartPage {} })];

#[component]
fn FocusStartPage() -> Element {
    rsx! {
        Button { id: "first", "First" }
        p { id: "early", "Before the middle" }
        Button { id: "middle", "Middle" }
        p { id: "late", "After the middle" }
        Button { id: "last", "Last" }
    }
}
