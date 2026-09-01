use crate::components::{Child, Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use crate::icons::CheckmarkIcon;
use dioxus::prelude::*;
use libero::components::{Badge, Code, CodeBlock, Flex, Icon, Input, Text};

/// The label is a child, not a prop, so the control that varies it prints
/// nothing of its own - `Demo` renders it as the child.
fn label(values: &DemoValues) -> String {
    values.str("label")
}

const SECTIONS: &str = r#"Badge {
    color: "success",
    Flex { direction: "row", align: "center", gap: "xs",
        Icon { size: "xs", CheckmarkIcon {} }
        "Verified"
    }
}"#;

const OR_CHIP: &str = r#"Badge { color: "success", "Shipped" }                      // a label
Chip { checked: on(), onchange: move |v| set(v), "Rust" }  // a choice"#;

#[component]
pub fn BadgePage() -> Element {
    rsx! {
        DocPage {
            title: "Badge",
            source: "libero/src/components/data_display/badge.rs",
            markdown: "/md/badge.md",
            properties: vec![props("Badge", vec![
                prop("variant", "ButtonVariant")
                    .default("filled")
                    .doc("Chrome, shared with `Button` and `Chip`: `filled`, `tonal`, `elevated`, `outlined`, `standard`. A badge is not interactive, so it takes no hover response."),
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("The accent; a theme color name or a literal CSS color. A theme color also brings the `-contrast` twin the label reads with."),
                prop("size", "Size")
                    .default("md")
                    .doc("Height, horizontal padding and font size, on the badge's own scale - smaller than a chip's."),
                prop("radius", "ThemeAwareValue")
                    .default("9999px")
                    .doc("A size step or any CSS length. The theme's own default is off the scale, because a badge is a pill at every height."),
                prop("circle", "bool")
                    .default("false")
                    .doc("Drops the horizontal padding and floors the width at the height, for a one- or two-character count."),
                prop("children", "Element").doc("The label."),
            ])],
            lead: rsx! {
                Text {
                    "A short status label. Renders one "
                    Code { source: "<span>" }
                    " with no role and no ARIA - a badge is visible text, read in document "
                    "order, so its content already is its accessible name. The theme owns the "
                    "uppercase, the letter spacing and the weight that tell a badge from a chip "
                    "at a glance; a project flips them once in "
                    Code { source: "BadgeDefaults" }
                    " rather than per call site."
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
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info", "neutral"],
                    ),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    // "pill" is the theme's own off-scale default, so it is
                    // "leave the prop unset", not a value to print.
                    Control::slider("radius", ["pill", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .code(|_, values| match values.str("radius").as_str() {
                            "pill" => vec![],
                            radius => vec![format!("radius: {radius:?}")],
                        }),
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
                        radius: match values.str("radius").as_str() {
                            "pill" => Input::None,
                            radius => Input::from(radius),
                        },
                        circle: values.str("circle") == "true",
                        {values.str("label")}
                    }
                },
                child: Child(label),
            }
            DocSection {
                title: "Sections",
                Text {
                    "There is no "
                    Code { source: "left_section" }
                    "/"
                    Code { source: "right_section" }
                    " - the root is a flex container, so a "
                    Code { source: "Flex" }
                    " child does the same job with nothing new to learn."
                }
                Badge {
                    color: "success",
                    Flex { direction: "row", align: "center", gap: "xs",
                        Icon { size: "xs", CheckmarkIcon {} }
                        "Verified"
                    }
                }
                CodeBlock { source: SECTIONS, language: "rust" }
            }
            DocSection {
                title: "Badge or Chip?",
                Text {
                    "A badge labels; a chip is picked. If the thing can be selected, clicked or "
                    "followed, it is a "
                    Code { source: "Chip" }
                    " - which carries "
                    Code { source: "checked" }
                    ", "
                    Code { source: "onclick" }
                    " and "
                    Code { source: "to" }
                    ", and responds to the pointer. A badge has none of that and never changes "
                    "under the cursor."
                }
                CodeBlock { source: OR_CHIP, language: "rust" }
            }
        }
    }
}
