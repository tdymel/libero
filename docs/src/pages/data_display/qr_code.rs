use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
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
            properties: vec![
                props("QrCode", vec![
                    prop("data", "String").doc("The payload encoded into the code."),
                    prop("robustness", "QrRobustness")
                        .default("medium")
                        .doc("How much damage or occlusion the code tolerates, at the cost of density."),
                    prop("aria_label", "String")
                        .doc("Required: a QR code says nothing to a screen reader without one."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "Encodes "
                    Code { source: "data" }
                    " as a scalable QR code, rendered as an inline SVG. Background/"
                    "foreground colors come from the theme ("
                    Code { source: "Theme::qr_code" }
                    "), not per-instance props - "
                    Code { source: "robustness" }
                    " is the only thing you tune per code: higher levels tolerate more "
                    "damage or occlusion at the cost of a denser code, and unset takes "
                    Code { source: "Theme::qr_code.robustness" }
                    " (Medium)."
                }
                Text {
                    "The SVG has no fixed width or height, just a square "
                    Code { source: "viewBox" }
                    ", so it fills its container - which is why the demo carries an "
                    Code { source: "sx" }
                    " width. "
                    Code { source: "aria_label" }
                    " is required, not optional: a QR code carries real information to a "
                    "sighted or scanning user and none at all to a screen reader without one."
                }
            },
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
    }
}
