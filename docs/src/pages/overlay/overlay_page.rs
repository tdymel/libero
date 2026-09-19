use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Code, Input, Overlay, Paper, Text},
    sx::sx,
};

/// On its own surface, so it reads at every dim.
const LOADING: &str = r#"Paper { sx: sx().padding("8px 16px"), "Loading..." }"#;

#[component]
pub fn OverlayPage() -> Element {
    rsx! {
        DocPage {
            title: "Overlay",
            source: "libero/src/components/overlay/overlay.rs",
            markdown: "/md/overlay.md",
            properties: vec![
                props("Overlay", vec![
                    prop("z_index", "ThemeAwareValue").default("300").doc("Stacking order of the layer."),
                    prop("opacity", "ThemeAwareValue").default("0.6").doc("How dark the dim is."),
                    prop("blur", "ThemeAwareValue").default("none").doc("How much the content behind is blurred. A bare number becomes `px`."),
                    prop("onclick", "EventHandler<MouseEvent>").doc("Called on a click anywhere on the overlay, such as a backdrop click."),
                    prop("children", "Element").doc("Content centred on the overlay, such as a loading spinner."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Dims and blurs whatever is behind it. A modal renders one behind its "
                    "content. There is no "
                    Code { source: "open" }
                    " prop, so render it conditionally. Its children are centred, which makes "
                    "it a loading screen as well as a backdrop."
                }
                Text {
                    "It covers the viewport. To keep it inside a box of your own, give that box a "
                    Code { source: "position" }
                    " and a "
                    Code { source: "z-index" }
                    ", and the overlay "
                    Code { source: "position: absolute" }
                    ". Without the "
                    Code { source: "z-index" }
                    ", the overlay still stacks against the whole page."
                }
            },
            Demo {
                component: "Overlay",
                children_text: "",
                children_code: LOADING.to_string(),
                controls: vec![
                    Control::slider("opacity", ["0.2", "0.4", "0.6", "0.8"]).default("0.6"),
                    Control::slider("blur", ["auto", "2px", "4px", "8px"]),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx()
                            .position("relative")
                            .z_index("0")
                            .height("160px")
                            .width("100%")
                            .background("muted.2"),
                        Text { sx: sx().padding("16px"), "Content behind the overlay" }
                        Overlay {
                            opacity: values.str("opacity"),
                            blur: match values.str("blur").as_str() {
                                "auto" => Input::None,
                                blur => Input::from(blur),
                            },
                            sx: sx().position("absolute"),
                            Paper { sx: sx().padding("8px 16px"), "Loading..." }
                        }
                    }
                },
            }
            DocSection { title: "Accessibility",
                Text {
                    "An overlay does not trap focus or hide the page from a screen reader. "
                    "For a modal backdrop, use a "
                    Code { source: "Dialog" }
                    " in "
                    Code { source: "use_modal" }
                    ", which brings its own overlay."
                }
            }
        }
    }
}
