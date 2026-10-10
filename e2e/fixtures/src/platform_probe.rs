//! What each engine offers (`e2e/tests/all/platform_probe.rs` reads it): an element padded by
//! the top safe-area inset and one the user-select rule lands on.

use dioxus::prelude::*;

use crate::Routes;

pub const ROUTES: Routes = &[("/platform-probe", || rsx! { Probe {} })];

#[component]
fn Probe() -> Element {
    rsx! {
        div { id: "inset-top", style: "padding-top: env(safe-area-inset-top, 0px)", "probe" }
        div { id: "no-select", style: "-webkit-user-select: none; user-select: none", "none" }
    }
}
