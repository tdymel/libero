use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Input, ProgressBar, Text};

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
                    .doc("Required. Current progress, clamped into `min..=max`. `None` is indeterminate: the bar sweeps and `aria-valuenow` is dropped."),
                prop("min", "f64").default("0.0").doc("Range start."),
                prop("max", "f64").default("100.0").doc("Range end. At or below `min` it warns and draws empty."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("The fill; a theme color name or a literal CSS color."),
                prop("size", "Size").default("md").doc("Track height, 3px at `xs` to 20px at `xxl`."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Corner of the track and the fill. A track is a full pill once the radius reaches half its height, so on the default `md` track only `xs` looks different."),
                prop("aria_valuetext", "String")
                    .doc("What a screen reader announces instead of the rounded percentage."),
            ])],
            lead: rsx! {
                Text {
                    "A bar that fills from "
                    Code { source: "min" }
                    " to "
                    Code { source: "max" }
                    ". It is output, not a control: it takes no focus and no keys. The root "
                    "carries "
                    Code { source: "role=\"progressbar\"" }
                    " and the raw "
                    Code { source: "aria-valuenow" }
                    "/"
                    Code { source: "min" }
                    "/"
                    Code { source: "max" }
                    ", so it needs a name - pass "
                    Code { source: "aria_label" }
                    ", which lands on the root. The fill eases to each new value; with "
                    Code { source: "value: None" }
                    " it sweeps instead, for the time before the total is known. The bar is "
                    "not a live region: one that ticks inside "
                    Code { source: "role=\"status\"" }
                    " floods the screen reader's queue. If progress has to be announced, "
                    "update a separate status line at milestones, not on every tick."
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
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    ),
                    // Opens on the tallest track, not the default `md`: on an
                    // 8px track every radius from `sm` up is the same pill, so
                    // the radius slider would look dead. The theme default
                    // still prints nothing.
                    Control::slider("size", SIZES)
                        .default("xxl")
                        .code(|_, values| match values.str("size").as_str() {
                            "md" => vec![],
                            size => vec![format!("size: {size:?}")],
                        }),
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
