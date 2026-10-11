//! The WebView apps' viewport rewrite (`main.rs`): a probe padded by the top safe-area inset,
//! and a field under 16px, which iOS zooms into on focus unless the viewport stops it.

use dioxus::prelude::*;
use libero::components::TextField;
use libero::theme::Size;

use crate::Routes;

pub const ROUTES: Routes = &[("/viewport-meta", || rsx! { Probe {} })];

#[component]
fn Probe() -> Element {
    rsx! {
        div { id: "inset-top", style: "padding-top: env(safe-area-inset-top, 0px)", "inset" }
        TextField { id: "small-field", label: "Small", size: Size::Sm }
    }
}
