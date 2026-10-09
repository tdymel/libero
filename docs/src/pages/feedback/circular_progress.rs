use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{CircularProgress, CircularProgressPart, Code, Input, Text},
    use_theme,
};

fn indeterminate(values: &DemoValues) -> bool {
    values.str("indeterminate") == "true"
}

/// The centred text: the percentage while there is one.
fn label_text(values: &DemoValues) -> String {
    match values.str("label") == "true" && !indeterminate(values) {
        true => format!("{}%", values.str("value")),
        false => String::new(),
    }
}

#[component]
pub fn CircularProgressPage() -> Element {
    let theme = use_theme();
    let defaults = theme.circular_progress;
    rsx! {
        DocPage {
            title: "CircularProgress",
            source: "libero/src/components/feedback/circular_progress.rs",
            markdown: "/md/circular_progress.md",
            properties: vec![props("CircularProgress", vec![
                prop("value", "Option<f64>")
                    .default("required")
                    .doc("Current progress, clamped into `min..=max`. `None` makes it spin."),
                prop("min", "f64").default("0.0").doc("Range start."),
                prop("max", "f64").default("100.0").doc("Range end. At or below `min` the ring draws empty."),
                prop("color", "ThemeAwareValue")
                    .default(defaults.color.as_str())
                    .doc("The arc. A theme color name paints its text shade, as `ProgressBar`'s fill. Any other CSS color paints as given."),
                prop("size", "Size")
                    .default(defaults.size.as_str())
                    .doc("The outer edge, 18px at `xs` to 72px at `xxl` at the default text size; it grows with text zoom."),
                prop("thickness", "Size")
                    .default(defaults.thickness.as_str())
                    .doc("The ring's width as a share of the edge, 6% at `xs` to 16% at `xxl`, so it grows with `size`."),
                prop("aria_valuetext", "String")
                    .doc("Read instead of the rounded percentage, such as \"3 of 8 files\"."),
                prop("children", "Element")
                    .doc("Drawn in the middle of the ring, such as the percentage or an icon, at a quarter of the edge, never under 12px. Hidden from screen readers: say the same in `aria_valuetext` when it differs from the percentage."),
                prop("parts", "Parts<CircularProgressPart>")
                    .doc("Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(CircularProgressPart::Label, sx().font_weight(\"bold\"))`."),
            ])
            .parts("CircularProgressPart", vec![
                (CircularProgressPart::Arc, "The `<svg>` holding the value arc. The root is the track."),
                (CircularProgressPart::Label, "The centred children."),
            ])],
            accessibility: a11y()
                .handles([
                    "A screen reader reads the rounded percentage, or `aria_valuetext` when you set it. A spinning ring reports no value.",
                    "The ring takes no focus. The centred children are hidden from screen readers, so the value is not read twice.",
                    "A theme color draws the arc in its text shade, at 3:1 or more against the track and the page. Yellow stays short of that on a light page, so `warning` draws its arc on a wider ink ring.",
                    "With reduced motion a spinning ring stops and shows as a full dashed ring, so it does not read as a quarter done.",
                ])
                .must([
                    "Name it with `aria_label`, or `aria_labelledby` pointing at a visible caption.",
                    "The ring is not a live region. To announce progress, update a separate status line at milestones, not on every tick.",
                    "Pick a `size` whose middle fits the children: the text is never under 12px, so it spills over an `xs` or `sm` ring, and `md` holds about three characters (\"42%\", not \"100%\").",
                ])
                .example("An upload ring, `CircularProgress { value: Some(42.0), aria_label: \"Upload\", \"42%\" }`: a screen reader reads the name \"Upload\" and 42% when it reaches the ring."),
            lead: rsx! {
                Text {
                    "A ring that fills clockwise from the top, for a percentage in a small space: on a "
                    "button, around an avatar, beside a step count. It shows output and takes no focus. With "
                    Code { source: "value: None" }
                    " it spins instead, for the time before the total is known."
                }
            },
            Demo {
                component: "CircularProgress",
                children_text: "",
                child: Child(label_text),
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
                    Control::color("color"),
                    // Large enough for its label; the prop's own default is `md`.
                    Control::sizes("size").default("xl").code(|_, values| {
                        match values.str("size").as_str() {
                            "md" => vec![],
                            size => vec![format!("size: {size:?}")],
                        }
                    }),
                    Control::sizes("thickness").default("md"),
                    // Not a prop: the centred children.
                    Control::switch("label").default("true").code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| {
                    let label = label_text(&values);
                    rsx! {
                        CircularProgress {
                            aria_label: "Upload",
                            value: if indeterminate(&values) {
                                None
                            } else {
                                values.str("value").parse::<f64>().ok()
                            },
                            color: Input::from(values.str("color")),
                            size: values.str("size"),
                            thickness: values.str("thickness"),
                            if !label.is_empty() { "{label}" }
                        }
                    }
                },
            }
        }
    }
}
