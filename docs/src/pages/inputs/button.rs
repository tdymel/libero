use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Flex, Text};

#[component]
pub fn ButtonPage() -> Element {
    rsx! {
        DocPage {
            title: "Button",
            source: "libero/src/components/inputs/button.rs",
            markdown: "/md/button.md",
            properties: vec![props("Button", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Accent color; a theme color name or a literal CSS color."),
                prop("variant", "ButtonVariant")
                    .default("filled")
                    .doc("Visual style, in Material 3's descending emphasis order: `filled`, `tonal`, `elevated`, `outlined`, `text`."),
                prop("radius", "Size")
                    .default("md")
                    .doc("Corner radius, independent of size."),
                prop("size", "Size")
                    .default("md")
                    .doc("Controls height, padding, and font size."),
                prop("full_width", "bool")
                    .default("false")
                    .doc("Stretches the button to fill its container."),
                prop("selected", "bool")
                    .doc("Turns the button into a toggle, rendering `aria-pressed` and the selected look. Omit to keep it a plain action."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the button."),
                prop("onclick", "EventHandler<MouseEvent>")
                    .doc("Click handler; not called when the button renders as a link."),
                prop("to", "NavigationTarget")
                    .doc("Renders as a router-aware link instead of a `<button>`."),
                prop("target", "String")
                    .doc("The link's `target` attribute, when `to` is set."),
                prop("children", "Element").doc("The button's label."),
            ]).extends("button")],
            lead: rsx! {
                Text {
                    "A clickable control, or a router-aware link when "
                    Code { source: "to" }
                    " is set. A plain "
                    Code { source: "<button>" }
                    " submits an enclosing form; ours defaults to "
                    Code { source: "type=\"button\"" }
                    " instead, so a submit or reset button says so."
                }
            },
            Demo {
                component: "Button",
                children_text: "Save changes",
                controls: vec![
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info", "neutral"],
                    ),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "text"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Text"]),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::switch("full_width"),
                    Control::switch("selected"),
                    Control::switch("disabled"),
                    // A `GlobalAttributes` pass-through rather than a
                    // prop, so it prints as the raw identifier.
                    Control::toggle("type", ["button", "submit", "reset"]).code(
                        |control, values| match values.str("type") {
                            value if value == control.default => vec![],
                            value => vec![format!("r#type: {value:?}")],
                        },
                    ),
                ],
                render: move |values: DemoValues| rsx! {
                    Button {
                        color: values.str("color"),
                        variant: values.str("variant"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        full_width: values.str("full_width") == "true",
                        // `Some(false)` is still a toggle button
                        // (`aria-pressed="false"`); unset is not.
                        selected: match values.str("selected").as_str() {
                            "true" => Some(true),
                            _ => None,
                        },
                        disabled: values.str("disabled") == "true",
                        r#type: values.str("type"),
                        "Save changes"
                    }
                },
            }
            DocSection {
                title: "Emphasis",
                Text {
                    "The five styles are one ladder, highest emphasis first. Reach for "
                    Code { source: "filled" }
                    " for the action that completes a flow, and "
                    Code { source: "text" }
                    " when several options sit side by side. "
                    Code { source: "elevated" }
                    " is a tonal button with a shadow - use it where the button has to separate itself from a patterned or scrolling background."
                }
                Flex {
                    gap: "sm",
                    wrap: "wrap",
                    Button { variant: "filled", "Filled" }
                    Button { variant: "tonal", "Tonal" }
                    Button { variant: "elevated", "Elevated" }
                    Button { variant: "outlined", "Outlined" }
                    Button { variant: "text", "Text" }
                }
            }
            DocSection {
                title: "Neutral",
                Text {
                    "Every style takes a color, and "
                    Code { source: "neutral" }
                    " is the text-dark one - the button that should not compete with the page's accent."
                }
                Flex {
                    gap: "sm",
                    wrap: "wrap",
                    Button { variant: "filled", color: "neutral", "Filled" }
                    Button { variant: "tonal", color: "neutral", "Tonal" }
                    Button { variant: "elevated", color: "neutral", "Elevated" }
                    Button { variant: "outlined", color: "neutral", "Outlined" }
                    Button { variant: "text", color: "neutral", "Text" }
                }
            }
            DocSection {
                title: "As a link",
                Text {
                    "Renders as a real anchor, or a router Link when to matches an internal route.",
                }
                Button {
                    variant: "outlined",
                    to: "https://dioxuslabs.com",
                    target: "_blank",
                    "Open Dioxus docs"
                }
            }
        }
    }
}
