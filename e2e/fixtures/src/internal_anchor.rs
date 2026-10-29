//! An internal `_self` link under the app's router, its history served under `/base`.

use dioxus::prelude::*;
use libero::components::Anchor;

use crate::Routes;

pub const ROUTES: Routes = &[("/internal-anchor", || rsx! { BasePage {} })];

/// The router asks the nearest history for its prefix, as a `dx` `base_path` build would.
#[component]
fn BasePage() -> Element {
    use_hook(|| {
        dioxus::history::provide_history_context(std::rc::Rc::new(
            dioxus::history::MemoryHistory::default().with_prefix("/base"),
        ))
    });
    rsx! {
        Anchor { id: "self", to: "/elsewhere", target: "_self", "Elsewhere" }
        Anchor { id: "external", to: "https://example.com", target: "_self", "Example" }
    }
}
