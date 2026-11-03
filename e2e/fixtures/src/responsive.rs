//! Responsive axis props and `hidden_from` / `visible_from` (todo 5643).

use dioxus::prelude::*;
use libero::components::{Badge, Button, Flex};
use libero::theme::{Size, Variant, responsive};

use crate::Routes;

pub const ROUTES: Routes = &[("/responsive", || rsx! { ResponsivePage {} })];

/// Small and outlined under 62rem, large and filled from it; a late button registers its
/// copies after mount.
#[component]
fn ResponsivePage() -> Element {
    let mut late = use_signal(|| false);
    rsx! {
        Flex { gap: "md", align: "center", wrap: "wrap",
            Button {
                id: "rsp-button",
                size: responsive(Size::Sm).md(Size::Xl),
                variant: responsive(Variant::Outlined).md(Variant::Filled),
                "Responsive"
            }
            Button { id: "rsp-hidden", hidden_from: "md", "Narrow only" }
            Badge { id: "rsp-badge", visible_from: "md", "Wide only" }
            Button { id: "rsp-add", onclick: move |_| late.set(true), "Add" }
            if late() {
                Button { id: "rsp-late", size: responsive(Size::Xs).md(Size::Lg), "Late" }
            }
        }
    }
}
