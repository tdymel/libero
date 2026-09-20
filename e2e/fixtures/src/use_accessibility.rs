//! What `use_accessibility` reads back from the browser, and a forced reduced
//! motion reaching libero's CSS.

use dioxus::prelude::*;
use libero::components::{Box, Text};
use libero::hooks::use_accessibility;
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/use-accessibility", || rsx! { UseAccessibilityPage {} })];

#[component]
fn UseAccessibilityPage() -> Element {
    let accessibility = use_accessibility();
    let motion = if accessibility.reduced_motion() {
        "reduce"
    } else {
        "no-preference"
    };
    let force = move |reduced: Option<bool>| {
        let accessibility = accessibility.clone();
        move |_| accessibility.set_reduced_motion(reduced)
    };
    rsx! {
        Text { id: "motion", "{motion}" }
        button { id: "still", onclick: force(Some(true)), "Still" }
        button { id: "moving", onclick: force(Some(false)), "Moving" }
        button { id: "system", onclick: force(None), "System" }
        Box {
            id: "motion-box",
            sx: sx()
                .width("10px")
                .media("(prefers-reduced-motion: reduce)", sx().width("20px")),
        }
    }
}
