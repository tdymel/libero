use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::Pictogram;
use libero::components::{ActionIcon, Button, ButtonGroup, Code, Input, Text};
use pictogram_icons_lucide as lucide;

/// The buttons inside - a subtree, so the code block prints it verbatim.
const CHILDREN: &str = r#"Button { "Undo" }
Button { "Redo" }
ActionIcon { aria_label: "Confirm", Pictogram { icon: lucide::check::outlined } }"#;

#[component]
pub fn ButtonGroupPage() -> Element {
    rsx! {
        DocPage {
            title: "ButtonGroup",
            source: "libero/src/components/buttons/button_group.rs",
            markdown: "/md/button_group.md",
            properties: vec![props("ButtonGroup", vec![
                prop("orientation", "Orientation")
                    .default("horizontal")
                    .doc("`\"vertical\"` stacks the buttons, each as wide as the widest."),
                prop("variant", "Variant")
                    .doc("Default `variant` of the buttons inside. Between two buttons without a visible border of their own (every variant but `outlined`), the group draws a thin divider."),
                prop("color", "ThemeAwareValue")
                    .doc("Default `color` of the buttons inside."),
                prop("size", "Size")
                    .doc("Default `size` of the buttons inside."),
                prop("radius", "Size")
                    .doc("The group's outer corners. The corners between two buttons are always square."),
                prop("disabled", "bool")
                    .doc("Disables every button inside that does not set `disabled` itself."),
                prop("children", "Element")
                    .default("required")
                    .doc("`Button`s, `ActionIcon`s, and components built on them, such as `ThemeToggle`."),
            ])],
            accessibility: a11y()
                .handles([
                    "The root is a `role=\"group\"`, so a screen reader announces the buttons as one set.",
                    "Each button stays its own Tab stop, and the focused one's ring is drawn above its neighbours.",
                ])
                .must(["Name the group with `aria-label`, or `aria-labelledby` on a visible heading."])
                .limits([
                    "It is not a `toolbar`: no arrow-key navigation between the buttons.",
                    "For one choice out of several, use `SegmentedControl`, which is a radio group.",
                    "A child hidden with `display: none` still counts as first or last and squares its neighbour's outer corners; render a conditional control only when shown.",
                ]),
            lead: rsx! {
                Text {
                    "Buttons and action icons side by side as one control. Neighbours share "
                    "one seam and only the group's outer corners are round, on logical sides, "
                    "so the ends swap under right-to-left text."
                }
                Text {
                    Code { source: "variant" }
                    ", "
                    Code { source: "color" }
                    ", "
                    Code { source: "size" }
                    ", "
                    Code { source: "radius" }
                    " and "
                    Code { source: "disabled" }
                    " set the default of every button inside; a button's own prop wins. "
                    "This site's header groups its repository link, direction toggle and theme toggle, and on a phone the search."
                }
            },
            Demo {
                component: "ButtonGroup",
                children_text: "",
                children_code: CHILDREN,
                fixed: vec!["\"aria-label\": \"Edit\"".to_string()],
                controls: vec![
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("horizontal"),
                    Control::toggle(
                        "variant",
                        ["filled", "tonal", "elevated", "outlined", "standard", "gradient"],
                    )
                    .labels(["Filled", "Tonal", "Elevated", "Outlined", "Standard", "Gradient"])
                    .default("filled"),
                    // A bare `primary` is what an unset `color` resolves to, so that swatch prints nothing.
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    ButtonGroup {
                        "aria-label": "Edit",
                        orientation: values.str("orientation"),
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        disabled: values.str("disabled") == "true",
                        Button { "Undo" }
                        Button { "Redo" }
                        ActionIcon { aria_label: "Confirm", Pictogram { icon: lucide::check::outlined } }
                    }
                },
            }
        }
    }
}
