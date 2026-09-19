use crate::components::{Child, Control, Demo, DemoValues, DocPage, DocSection, prop, props};
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
                    // The child, not a prop - `circle` only reads with a one-
                    // or two-character label, so the demo has to be able to
                    // get there.
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
            DocSection { title: "Accessibility",
                Text {
                    "A badge has no role, so screen readers read its text in place and "
                    "announce no change. For a badge that reports a change, wrap it in your "
                    "own "
                    Code { source: "role=\"status\"" }
                    " region."
                }
                Text {
                    Code { source: "filled" }
                    " and "
                    Code { source: "tonal" }
                    " labels reach 4.5:1 in every color. The other variants print the label "
                    "in the color itself, which stays under 4.5:1 on white for "
                    Code { source: "warning" }
                    " and "
                    Code { source: "success" }
                    ". Pick "
                    Code { source: "filled" }
                    " or "
                    Code { source: "tonal" }
                    " for those."
                }
            }
        }
    }
}
