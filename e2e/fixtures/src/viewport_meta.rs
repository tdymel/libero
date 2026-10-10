//! The WebView apps' viewport rewrite (`main.rs`): a probe padded by the top safe-area inset.

use dioxus::prelude::*;

use crate::Routes;

pub const ROUTES: Routes = &[("/viewport-meta", || rsx! { Probe {} })];

#[component]
fn Probe() -> Element {
    rsx! {
        div { id: "inset-top", style: "padding-top: env(safe-area-inset-top, 0px)", "inset" }
    }
}
