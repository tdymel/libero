use crate::components::{Control, Demo, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, QrCode, Text},
    sx::sx,
};

const DATA: &str = "https://github.com/tdymel/libero";
const LABEL: &str = "QR code linking to the libero GitHub repository";

#[component]
pub fn QrCodePage() -> Element {
    rsx! {
        DocPage {
            title: "QrCode",
            lead: rsx! {
                Text {
                    "Encodes "
                    Code { source: "data" }
                    " as a scalable QR code, rendered as an inline SVG. Background/"
                    "foreground colors come from the theme ("
                    Code { source: "Theme::qr_code" }
                    "), not per-instance props - "
                    Code { source: "robustness" }
                    " is the only thing you tune per code."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "QrCode",
                    children_text: "",
                    controls: vec![
                        // Both are required, so they always print alongside
                        // the one prop there is to tune.
                        Control::slider("robustness", ["low", "medium", "quartile", "high"])
                            .default("medium")
                            .code(|control, values| {
                                // Required, plus the width the SVG has no
                                // intrinsic size to supply.
                                let mut set = vec![
                                    format!("data: {DATA:?}"),
                                    format!("aria_label: {LABEL:?}"),
                                    "sx: sx().width(\"160px\")".to_string(),
                                ];
                                let value = values.str("robustness");
                                if value != control.default {
                                    set.push(format!("robustness: {value:?}"));
                                }
                                set
                            }),
                    ],
                    render: move |values: DemoValues| rsx! {
                        QrCode {
                            data: DATA,
                            aria_label: LABEL,
                            robustness: values.str("robustness"),
                            sx: sx().width("160px"),
                        }
                    },
                }
            }
            DocSection {
                title: "Robustness",
                Text {
                    sx: sx().color("grey.6"),
                    "Error-correction level - higher levels tolerate more damage or "
                    "occlusion at the cost of a denser code for the same data. Falls "
                    "back to the theme's "
                    Code { source: "Theme::qr_code.robustness" }
                    " (Medium) when unset."
                }
            }
            DocSection {
                title: "Scalable",
                Text {
                    sx: sx().color("grey.6"),
                    "The generated SVG has no fixed width or height, just a square "
                    Code { source: "viewBox" }
                    " - it fills its container, so "
                    Code { source: "sx" }
                    " width/height (or the container's own size) decides how big it "
                    "renders. That is why the demo above carries one."
                }
            }
            DocSection {
                title: "Accessible name",
                Text {
                    sx: sx().color("grey.6"),
                    "aria_label is required, not optional - a QR code conveys real "
                    "information to a sighted or scanning user, but nothing to a screen "
                    "reader without one.",
                }
            }
        }
    }
}
