use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Input, ProgressBar, ProgressBarPart, Text};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

fn indeterminate(values: &DemoValues) -> bool {
    values.str("indeterminate") == "true"
}

#[component]
pub fn ProgressBarPage() -> Element {
    rsx! {
        DocPage {
            title: "ProgressBar",
            source: "libero/src/components/feedback/progress_bar.rs",
            markdown: "/md/progress_bar.md",
            properties: vec![props("ProgressBar", vec![
                prop("value", "Option<f64>")
                    .default("required")
                    .doc("Current progress, clamped into `min..=max`. `None` makes it indeterminate."),
                prop("min", "f64").default("0.0").doc("Range start."),
                prop("max", "f64").default("100.0").doc("Range end. At or below `min` the bar draws empty."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("The fill. A theme color name or any CSS color."),
                prop("size", "Size").default("md").doc("Track height, 3px at `xs` to 20px at `xxl`."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Corner of the track and the fill. On a thin track most steps draw the same pill."),
                prop("aria_valuetext", "String")
                    .doc("Read instead of the rounded percentage, such as \"4.2 MB of 12 MB\"."),
                prop("parts", "Parts<ProgressBarPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(ProgressBarPart::Fill, sx().background(\"success.6\"))`."),
            ])
            .parts("ProgressBarPart", vec![
                (ProgressBarPart::Fill, "The drawn share, or the indeterminate sweep. The root is the track."),
            ])],
            accessibility: a11y()
                .handles([
                    "A screen reader reads the rounded percentage, or `aria_valuetext` when you set it.",
                    "The bar takes no focus.",
                    "With reduced motion an indeterminate bar stops sweeping and shows as a dimmed full bar, so it does not read as part done.",
                ])
                .must([
                    "Name it with `aria_label`, or `aria_labelledby` pointing at a visible caption.",
                    "The bar is not a live region. To announce progress, update a separate status line at milestones, not on every tick.",
                ]),
            lead: rsx! {
                Text {
                    "A bar that fills from "
                    Code { source: "min" }
                    " to "
                    Code { source: "max" }
                    ". It shows output and takes no focus. With "
                    Code { source: "value: None" }
                    " it sweeps instead, for the time before the total is known."
                }
            },
            Demo {
                component: "ProgressBar",
                children_text: "",
                fixed: vec![r#"aria_label: "Upload""#.to_string()],
                controls: vec![
                    Control::switch("indeterminate").code(|_, values| {
                        if indeterminate(values) {
                            vec!["value: None".to_string()]
                        } else {
                            vec![format!("value: {}.0", values.str("value"))]
                        }
                    }),
                    Control::slider(
                        "value",
                        ["0", "10", "20", "30", "40", "50", "60", "70", "80", "90", "100"],
                    )
                    .default("40")
                    // Printed by `indeterminate`, which owns the prop.
                    .code(|_, _| vec![])
                    .hidden_when(indeterminate),
                    // An unset `color` is `base_color`'s primary shade 6, which
                    // is exactly what a bare `primary` resolves to.
                    Control::color("color"),
                    Control::slider("size", SIZES).default("md"),
                    Control::slider("radius", SIZES).default("xl"),
                ],
                render: move |values: DemoValues| rsx! {
                    ProgressBar {
                        aria_label: "Upload",
                        value: if indeterminate(&values) {
                            None
                        } else {
                            values.str("value").parse::<f64>().ok()
                        },
                        color: Input::from(values.str("color")),
                        size: values.str("size"),
                        radius: values.str("radius"),
                    }
                },
            }
        }
    }
}
