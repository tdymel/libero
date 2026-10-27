//! `Overlay` and `LoadingOverlay`.

use dioxus::prelude::*;
use libero::components::{Box, Button, Flex, LoadingOverlay, Overlay, Paper, Text};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[("/overlay", || rsx! { OverlayPage {} })];

/// A contained `Overlay` in `#box`, and a `LoadingOverlay` around `#inside` that
/// `#toggle` switches (todo 3985) and `#inside` starts. `#before` and `#after` frame it for Tab.
#[component]
fn OverlayPage() -> Element {
    let mut loading = use_signal(|| true);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            Box {
                id: "box",
                sx: sx().position("relative").z_index("0").height("120px"),
                Text { "Behind" }
                Overlay { id: "contained", contained: true }
            }
            Button { id: "toggle", onclick: move |_| loading.toggle(), "Toggle loading" }
            Button { id: "before", "Before" }
            LoadingOverlay { id: "loading", loading: loading(),
                Paper { sx: sx().padding("16px"),
                    Text { "Ada Lovelace" }
                    Button { id: "inside", onclick: move |_| loading.set(true), "Edit profile" }
                }
            }
            Button { id: "after", "After" }
        }
    }
}
