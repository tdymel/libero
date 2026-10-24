//! Sheets first registered under a suspense with no `SuspenseBoundary` above it, so the
//! whole app, the style outlet included, is suspended while they arrive (todo 4124).

use dioxus::prelude::*;
use libero::components::{Avatar, Box};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/suspense-style", || rsx! { SuspenseStylePage {} })];

/// "Load" mounts a page that suspends on its first render, as `use_server_future(..)?` does
/// on a client-side navigation; nothing on the page before uses its sheets.
#[component]
fn SuspenseStylePage() -> Element {
    let mut loaded = use_signal(|| false);
    rsx! {
        button { id: "load", onclick: move |_| loaded.set(true), "Load" }
        if loaded() {
            Loaded {}
        }
    }
}

#[component]
fn Loaded() -> Element {
    let name = use_resource(|| async { "Ada Lovelace".to_string() }).suspend()?;
    rsx! {
        div { id: "loaded",
            Avatar { id: "avatar", name: name(), initials: "AL", size: "xl" }
            Box { id: "box", sx: sx().background("rgb(41, 42, 43)"), "Loaded" }
        }
    }
}
