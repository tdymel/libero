use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, a11y, gradient_controls, gradient_value,
    not_gradient_variant, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Badge, Code, Input, Text},
    use_theme,
};

/// The label is a child, not a prop, so the control that varies it prints
/// nothing of its own. `Demo` renders it as the child.
fn label(values: &DemoValues) -> String {
    values.str("label")
}

#[component]
pub fn BadgePage() -> Element {
    let theme = use_theme();
    let [gradient_to, gradient_deg] = gradient_controls(not_gradient_variant);
    rsx! {
        DocPage {
            title: "Badge",
            source: "libero/src/components/data_display/badge.rs",
            markdown: "/md/badge.md",
            properties: vec![props("Badge", vec![
                prop("variant", "Variant")
                    .default(theme.badge.variant.as_str())
                    .doc("The look, shared with `Button` and `Chip`. A badge is not interactive, so it has no hover state."),
                prop("gradient", "Gradient")
                    .doc("With `variant: \"gradient\"`: the second stop and the angle, as `(\"info\", 90)` or `Gradient::default().to(\"info\").deg(90)`. The first stop is `color`. Ignored by the other variants."),
                prop("color", "ThemeAwareValue")
                    .default(theme.badge.color.as_str())
                    .doc("A theme color name or a CSS color. A theme color also sets a label color that reads on it. Under a gradient, its first stop."),
                prop("size", "Size")
                    .default(theme.badge.size.as_str())
                    .doc("Height, horizontal padding and font size, on a scale smaller than a chip's."),
                prop("radius", "ThemeAwareValue")
                    .default(theme.badge.radius.as_str())
                    .doc("A step on the badge's own radius scale, `2px` to `12px`, or any CSS, e.g. `radius: \"0\"`. The default `xxl` is a pill at every height."),
                prop("circle", "bool")
                    .default("false")
                    .doc("Drops the horizontal padding and makes the width at least the height, for a count of one or two characters."),
                prop("children", "Element").default("required").doc("The label."),
            ])],
            accessibility: a11y()
                .handles([
                    "A badge has no role, so screen readers read its text in place and announce no change.",
                    "`filled` and `tonal` labels reach 4.5:1 in every color.",
                    "`gradient` picks its own label, the black, white, ink or surface that reads best on both stops and the midpoint. Stops on which no label reaches 4.5:1 warn in debug builds.",
                ])
                .must([
                    "For a badge that reports a change, wrap it in your own `role=\"status\"` region.",
                    "Pick `filled` or `tonal` for `warning` and `success`.",
                    "Say what a `circle` count counts: a bare \"9\" reads as a number. Add the unit as `VisuallyHidden` text, `Badge { circle: true, \"9\", VisuallyHidden { \" unread\" } }`, or name it in the row (\"9 unread\").",
                ])
                .example("A \"Paid\" badge in an invoice row, `Badge { color: \"success\", variant: \"tonal\", \"Paid\" }`: read in place with the row, at 4.5:1 or more.")
                .limits([
                    "`elevated`, `outlined` and `standard` print the label in the color itself, which stays under 4.5:1 on white for `warning` (3.27:1) and `success` (4.05:1).",
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
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    ).default(theme.badge.variant.as_str())
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"]),
                    // A bare `primary` is what an unset `color` resolves to,
                    // so that swatch prints nothing.
                    Control::color("color").default(theme.badge.color.as_str()),
                    gradient_to,
                    gradient_deg,
                    Control::sizes("size").default(theme.badge.size.as_str()),
                    // `standard` draws no box, so there is no corner to round.
                    Control::sizes("radius")
                        .default(theme.badge.radius.as_str())
                        .hidden_when(|values| values.str("variant") == "standard"),
                    Control::switch("circle"),
                    // The child, not a prop: `circle` needs a one- or two-character label.
                    Control::toggle("label", ["New", "Beta", "9"]).code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    Badge {
                        variant: values.str("variant"),
                        gradient: gradient_value(&values),
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
