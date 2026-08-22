use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Input, Overlay, Text},
    sx::sx,
};

#[component]
pub fn OverlayPage() -> Element {
    rsx! {
        DocPage {
            title: "Overlay",
            lead: rsx! {
                Text {
                    "Dims and blurs whatever is behind it - "
                    Code { source: "Modal" }
                    " renders one behind its content. Render it conditionally; there is no "
                    Code { source: "open" }
                    ". It spans the viewport as "
                    Code { source: "position: fixed" }
                    " by default, so the demo below overrides that to "
                    Code { source: "absolute" }
                    " to stay inside its frame - that frame and its "
                    Code { source: "z-index: 0" }
                    " are the demo's own scaffolding, left out of the code below. Containing "
                    "an overlay always takes both: "
                    Code { source: "position" }
                    " alone starts no stacking context, so the overlay's "
                    Code { source: "z-index: 300" }
                    " would escape and compete with the whole page. Children are centred in "
                    "it, which is what makes it a loading layer as well as a backdrop."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Overlay",
                    children_text: "Loading...",
                    controls: vec![
                        Control::slider("opacity", ["auto", "0.2", "0.4", "0.6", "0.8"]),
                        Control::slider("blur", ["auto", "2px", "4px", "8px"]),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Box {
                            sx: sx()
                                .position("relative")
                                .z_index("0")
                                .height("160px")
                                .width("100%")
                                .background("grey.2"),
                            Text { sx: sx().padding("16px"), "Content behind the overlay" }
                            Overlay {
                                opacity: match values.str("opacity").as_str() {
                                    "auto" => Input::None,
                                    opacity => Input::from(opacity),
                                },
                                blur: match values.str("blur").as_str() {
                                    "auto" => Input::None,
                                    blur => Input::from(blur),
                                },
                                sx: sx().position("absolute"),
                                "Loading..."
                            }
                        }
                    },
                }
            }
        }
    }
}
