//! What `use_accessibility` reads back from the browser.

use dioxus::prelude::*;
use libero::components::Text;
use libero::hooks::use_accessibility;

use crate::Routes;

pub const ROUTES: Routes = &[("/use-accessibility", || rsx! { UseAccessibilityPage {} })];

#[component]
fn UseAccessibilityPage() -> Element {
    let now = use_accessibility().get();
    let motion = if now.reduced_motion {
        "reduce"
    } else {
        "no-preference"
    };
    rsx! {
        Text { id: "motion", "{motion}" }
    }
}
