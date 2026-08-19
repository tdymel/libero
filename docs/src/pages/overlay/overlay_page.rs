use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Overlay, Text},
    sx::sx,
};

#[component]
pub fn OverlayPage() -> Element {
    rsx! {
        DocPage {
            title: "Overlay",
            lead: rsx! {
                Text {
                    "Dims/blurs whatever is behind it - Modal renders one behind its content. "
                    "Defaults to position: fixed, spanning the whole viewport; overridden to "
                    "position: absolute below to stay contained in this demo."
                }
            },
            DocSection {
                title: "Example",
                Box {
                    // `z-index` (any value, not just a high one) is what
                    // actually contains the overlay here - `position` alone
                    // doesn't start a new stacking context, so without it
                    // the overlay's own `z-index: 300` (sized for its real
                    // job: sit below a `Modal`, above ordinary page content)
                    // would escape this box and compete globally, e.g.
                    // outranking the docs site's own mobile nav drawer.
                    sx: sx()
                        .position("relative")
                        .z_index("0")
                        .height("160px")
                        .background("grey.2"),
                    Text { sx: sx().padding("16px"), "Content behind the overlay" }
                    Overlay { sx: sx().position("absolute") }
                }
            }
        }
    }
}
