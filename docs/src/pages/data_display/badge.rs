use crate::components::{Child, Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Badge, Code, Input, Text};

/// The label is a child, not a prop, so the control that varies it prints
/// nothing of its own. `Demo` renders it as the child.
fn label(values: &DemoValues) -> String {
    values.str("label")
}

#[component]
pub fn BadgePage() -> Element {
    rsx! {
        DocPage {
            title: "Badge",
            source: "libero/src/components/data_display/badge.rs",
            markdown: "/md/badge.md",
            properties: vec![props("Badge", vec![
                prop("variant", "Variant")
                    .default("filled")
                    .doc("The look, shared with `Button` and `Chip`. A badge is not interactive, so it has no hover state."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: this badge's own stops and angle. Ignored by the other variants."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("A theme color name or a CSS color. A theme color also sets a label color that reads on it."),
                prop("size", "Size")
                    .default("md")
                    .doc("Height, horizontal padding and font size, on a scale smaller than a chip's."),
                prop("radius", "Size")
                    .default("xxl")
                    .doc("A step on the badge's own radius scale, `2px` to `12px`. The default `xxl` is a pill at every height."),
                prop("circle", "bool")
                    .default("false")
                    .doc("Drops the horizontal padding and makes the width at least the height, for a count of one or two characters."),
                prop("children", "Element").default("required").doc("The label."),
            ])],
            accessibility: a11y()
                .handles([
                    "A badge has no role, so screen readers read its text in place and announce no change.",
                    "`filled` and `tonal` labels reach 4.5:1 in every color.",
                ])
                .must([
                    "For a badge that reports a change, wrap it in your own `role=\"status\"` region.",
                    "Pick `filled` or `tonal` for `warning` and `success`.",
                ])
                .limits([
                    "The other variants print the label in the color itself, which stays under 4.5:1 on white for `warning` (3.27:1) and `success` (4.05:1).",
                ]),
            lead: rsx! {
                Text {
                    "A short status label, rendered as one "
                    Code { source: "<span>" }
                    ". An icon and text as children sit one spacing step apart. The theme "
                    "sets the uppercase, letter spacing and weight in "
                    Code { source: "BadgeDefaults" }
                    ". For something a user can select, click or follow, use "
                    Code { source: "Chip" }
                    "."
                }
            },
            Demo {
                component: "Badge",
                children_text: "New",
                controls: vec![
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard"]),
                    // A bare `primary` is what an unset `color` resolves to,
                    // so that swatch prints nothing.
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    // `standard` draws no box, so there is no corner to round.
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("xxl")
                        .hidden_when(|values| values.str("variant") == "standard"),
                    Control::switch("circle"),
                    // The child, not a prop: `circle` needs a one- or two-character label.
                    Control::toggle("label", ["New", "Beta", "9"]).code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    Badge {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        circle: values.str("circle") == "true",
                        {values.str("label")}
                    }
                },
                child: Child(label),
            }
        }
    }
}
