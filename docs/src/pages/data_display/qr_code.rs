use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, QrCode, Text},
    sx::sx,
};

const DATA: &str = "https://github.com/tdymel/libero";
const LABEL: &str = "The libero repository on GitHub";

#[component]
pub fn QrCodePage() -> Element {
    rsx! {
        DocPage {
            title: "QrCode",
            source: "libero/src/components/data_display/qr_code.rs",
            markdown: "/md/qr_code.md",
            properties: vec![
                props("QrCode", vec![
                    prop("data", "String").default("required").doc("The payload encoded into the code."),
                    prop("robustness", "QrRobustness")
                        .default("medium")
                        .doc("How much damage the code survives. Higher levels make a denser code."),
                    prop("aria_label", "String")
                        .default("required")
                        .doc("The code's accessible name. Say where it leads."),
                ]),
            ],
            accessibility: a11y()
                .handles(["The code is one `role=\"img\"` named by `aria_label`. The svg inside is hidden."])
                .must([
                    "Say in `aria_label` where the code leads or what it holds, not that it is a QR code. A screen reader user cannot scan it, so the label is the only way to the payload.",
                    "Next to a real link, the link serves better.",
                ]),
            lead: rsx! {
                Text {
                    "Encodes "
                    Code { source: "data" }
                    " as a QR code in an inline SVG. The colors come from the theme. The SVG "
                    "has no size of its own and fills its container, so give it a width. If "
                    Code { source: "data" }
                    " is too long for the "
                    Code { source: "robustness" }
                    ", it renders nothing."
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
