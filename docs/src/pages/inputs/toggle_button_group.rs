use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, or_unset, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Flex, Input, Text, ToggleButton, ToggleButtonGroup};

/// The buttons are the fixture - every prop belongs to the group - so the code
/// block prints them verbatim.
const CHILDREN: &str = r#"ToggleButton { value: "left", "Left" }
ToggleButton { value: "center", "Center" }
ToggleButton { value: "right", "Right" }"#;

#[component]
pub fn ToggleButtonGroupPage() -> Element {
    let mut alignment = use_signal(|| vec!["left".to_string()]);
    let mut disabled_one = use_signal(|| vec!["a".to_string()]);

    rsx! {
        DocPage {
            title: "ToggleButtonGroup",
            source: "libero/src/components/inputs/toggle_button_group",
            markdown: "/md/toggle_button_group.md",
            properties: vec![
                props("ToggleButtonGroup", vec![
                    prop("value", "Vec<String>")
                        .doc("Strictly controlled - pair it with `onchange`. `exclusive` holds it to at most one entry."),
                    prop("onchange", "EventHandler<Vec<String>>")
                        .doc("Called with the selection the group should take next."),
                    prop("exclusive", "bool")
                        .default("true")
                        .doc("Single-select: picking one clears the rest."),
                    prop("orientation", "Orientation")
                        .default("horizontal")
                        .doc("Row or column layout."),
                    prop("variant", "ButtonVariant")
                        .default("outlined")
                        .doc("The unselected look, passed to every button."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Accent color; a theme color name or a literal CSS color."),
                    prop("size", "Size").default("md").doc("Passed to every button."),
                    prop("radius", "Size")
                        .default("md")
                        .doc("Corner radius of the group's outer corners; inner ones are square."),
                    prop("gap", "Size")
                        .doc("Space between the buttons. Set it and they stop sharing borders - each keeps its own, and its own radius."),
                    prop("full_width", "bool")
                        .default("false")
                        .doc("Buttons share the width evenly instead of sizing to their label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every button in the group."),
                    prop("children", "Element").doc("`ToggleButton`s."),
                ]),
                props("ToggleButton", vec![
                    prop("value", "String").doc("Its identity in the group's selection."),
                    prop("disabled", "bool")
                        .default("follows the group")
                        .doc("Overrides the group's `disabled` for this button alone."),
                    prop("children", "Element").doc("The button's label."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A row of connected buttons sharing one selection. Each member is a real "
                    Code { source: "button" }
                    " carrying "
                    Code { source: "aria-pressed" }
                    ", so tab order and Space/Enter are the browser's own. Strictly controlled: "
                    Code { source: "value" }
                    " drives the look, "
                    Code { source: "onchange" }
                    " reports the selection the group should take next."
                }
            },
            Demo {
                component: "ToggleButtonGroup",
                children_text: "",
                children_code: CHILDREN,
                // Required as a pair, and neither is a value a control
                // varies - the group is only ever as selected as its owner.
                fixed: vec![
                    "value: alignment()".to_string(),
                    "onchange: move |next| alignment.set(next)".to_string(),
                ],
                controls: vec![
                    Control::toggle("variant", ["outlined", "filled", "text"])
                        .labels(["Outlined", "Filled", "Text"]),
                    // A bare `primary` is what an unset `color` resolves
                    // to, so that swatch prints nothing.
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    ),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("orientation", ["horizontal", "vertical"]),
                    // "auto" is no gap at all: the buttons stay connected
                    // and share their borders.
                    Control::slider("gap", ["auto", "xs", "sm", "md", "lg", "xl", "xxl"]),
                    Control::switch("exclusive").default("true"),
                    Control::switch("full_width"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    ToggleButtonGroup {
                        variant: values.str("variant"),
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        radius: values.str("radius"),
                        orientation: values.str("orientation"),
                        gap: or_unset(values.str("gap")),
                        exclusive: values.str("exclusive") == "true",
                        full_width: values.str("full_width") == "true",
                        disabled: values.str("disabled") == "true",
                        value: alignment(),
                        onchange: move |next| alignment.set(next),
                        ToggleButton { value: "left", "Left" }
                        ToggleButton { value: "center", "Center" }
                        ToggleButton { value: "right", "Right" }
                    }
                },
            }
            DocSection {
                title: "Per-button disabled",
                Text {
                    Code { source: "disabled" }
                    " on the group covers every button; a single "
                    Code { source: "ToggleButton" }
                    " can set its own instead."
                }
                Flex {
                    direction: "column",
                    gap: "md",
                    align: "start",
                    ToggleButtonGroup {
                        value: disabled_one(),
                        onchange: move |next| disabled_one.set(next),
                        ToggleButton { value: "a", "One" }
                        ToggleButton { value: "b", disabled: true, "Two" }
                    }
                }
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The group is a "
                    Code { source: "role=\"group\"" }
                    " of ordinary buttons, each carrying "
                    Code { source: "aria-pressed" }
                    ". Tab reaches every button and Space or Enter toggles it - there is no "
                    "arrow-key navigation to learn, and none is added."
                }
                Text {
                    "For a single-select group, APG would prefer a "
                    Code { source: "radiogroup" }
                    ". Following MUI, this component stays with pressed buttons: the roving "
                    "tabindex a radiogroup requires costs more than it returns here, and a "
                    "toggle button reads correctly to a screen reader either way. Name the group "
                    "with an "
                    Code { source: "aria_label" }
                    " where its purpose is not obvious from the buttons themselves."
                }
                ToggleButtonGroup {
                    aria_label: "Text alignment",
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                    ToggleButton { value: "left", "Left" }
                    ToggleButton { value: "center", "Center" }
                    ToggleButton { value: "right", "Right" }
                }
            }
        }
    }
}
