use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Input, ProgressBar, ProgressBarPart, ProgressBarSegment, Text};

fn indeterminate(values: &DemoValues) -> bool {
    values.str("indeterminate") == "true"
}

fn segmented(values: &DemoValues) -> bool {
    values.str("segments") == "true"
}

/// An upload's stages: preparing up to 25, sending to 75, verifying after.
fn segments(values: &DemoValues) -> Vec<ProgressBarSegment> {
    match segmented(values) {
        true => vec![
            ProgressBarSegment::labeled(25.0, "Send"),
            ProgressBarSegment::labeled(75.0, "Verify"),
        ],
        false => Vec::new(),
    }
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
                    .doc("The fill. A theme color name paints its text shade, darker on a light page so the bar stands out from its track. Any other CSS color paints as given."),
                prop("size", "Size").default("md").doc("Track height, 3px at `xs` to 20px at `xxl`."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Corner of the track and the fill. On a thin track most steps draw the same pill."),
                prop("aria_valuetext", "String")
                    .doc("Read instead of the rounded percentage, such as \"4.2 MB of 12 MB\"."),
                prop("segments", "Vec<ProgressBarSegment>")
                    .default("[]")
                    .doc("Splits the track into stretches with 2px gaps, each from its `start` to the next one's, as a `Slider`'s `segments`, and fills them up to the value. A label does not change what the bar reports: name the stage in `aria_valuetext`. Sorted for you; a start outside the range or a repeat is dropped, and an unlabeled stretch fills from `min` to the first start. An indeterminate bar sweeps as without them."),
                prop("parts", "Parts<ProgressBarPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(ProgressBarPart::Fill, sx().background(\"success.6\"))`."),
            ])
            .parts("ProgressBarPart", vec![
                (ProgressBarPart::Fill, "The drawn share, or the indeterminate sweep. The root is the track."),
                (ProgressBarPart::Segments, "The row of segments a `segments` bar draws instead of the fill."),
                (ProgressBarPart::Segment, "One stretch of a segmented bar."),
                (ProgressBarPart::SegmentFill, "The filled part of a segment."),
            ]),
            props("ProgressBarSegment", vec![
                prop("start", "f64").doc("Where the stretch starts; it ends at the next one's start, or `max`."),
                prop("label", "Option<String>").doc("Names the stretch. `ProgressBarSegment::labeled(start, label)`."),
            ])
            .without_base_props()],
            accessibility: a11y()
                .handles([
                    "A screen reader reads the rounded percentage, or `aria_valuetext` when you set it.",
                    "The bar takes no focus.",
                    "A theme color fills in its text shade, at 3:1 or more against the track and the page. Yellow stays short of that on a light page, so `warning` draws a 1px ink edge inside its fill.",
                    "With reduced motion an indeterminate bar stops sweeping and shows as a dimmed full bar, so it does not read as part done.",
                ])
                .must([
                    "Name it with `aria_label`, or `aria_labelledby` pointing at a visible caption.",
                    "The bar is not a live region. To announce progress, update a separate status line at milestones, not on every tick.",
                ])
                .example("An upload bar, `ProgressBar { value: Some(42.0), aria_label: \"Upload\" }`: a screen reader reads the name \"Upload\" and 42% when it reaches the bar. A status line beside it says \"Half done\" once, at the milestone."),
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
                    Control::sizes("size").default("md"),
                    Control::sizes("radius").default("xl"),
                    Control::switch("segments").default("true").code(|_, values| {
                        match segmented(values) {
                            true => vec![
                                r#"segments: vec![ProgressBarSegment::labeled(25.0, "Send"), ProgressBarSegment::labeled(75.0, "Verify")]"#
                                    .to_string(),
                            ],
                            false => vec![],
                        }
                    }),
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
                        segments: segments(&values),
                    }
                },
            }
        }
    }
}
