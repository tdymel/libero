use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, Input, Text, Tooltip},
    sx::sx,
    use_theme,
};

const SIZES: [&str; 5] = ["xs", "sm", "md", "lg", "xl"];

// snippet: in Tooltip { .., Button { "Save" } }
const LABEL: &str = r#"label: rsx! { "Saves the current draft" }"#;
const TRIGGER: &str = r#"Button { variant: "outlined", "Save" }"#;
// snippet: in Tooltip { label: rsx! { "Save" }, .., Button { "Save" } }
const STYLED: &str = r#"sx: sx().background("primary.6").white_space("normal").max_width("12rem")"#;

/// The bubble escapes the trigger's box, and the demo card clips what leaves
/// it - so the preview keeps a bubble's worth of room on every side.
fn wrap_room(_: &DemoValues, code: &str) -> String {
    format!(
        "Box {{\n    sx: sx().padding(\"40px\"),\n{}}}",
        indent(code)
    )
}

/// Milliseconds print unquoted, and `auto` is the theme's own delay.
fn delay_code(control: &Control, values: &DemoValues) -> Vec<String> {
    match values.str(control.name).as_str() {
        "auto" => vec![],
        delay => vec![format!("{}: {delay}", control.name)],
    }
}

fn delay(value: String) -> Option<u32> {
    value.parse().ok()
}

#[component]
pub fn TooltipPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Tooltip",
            source: "libero/src/components/overlay/tooltip.rs",
            markdown: "/md/tooltip.md",
            properties: vec![props("Tooltip", vec![
                prop("label", "Element").doc("The bubble's content."),
                prop("side", "Side")
                    .default("top")
                    .doc("Which side of the trigger the bubble sits on, centred on that side. No viewport flipping."),
                prop("gap", "Size")
                    .default("xs")
                    .doc("Distance to the trigger, rendered as transparent padding so the pointer can cross it."),
                prop("size", "Size").default("sm").doc("Font size of the bubble."),
                prop("z_index", "ThemeAwareValue")
                    .default("the float layer")
                    .doc("Overrides the stacking level, for a bubble that loses to a neighbouring overlay."),
                prop("open_delay", "u32")
                    .default("0")
                    .doc("Milliseconds the pointer must rest before the bubble appears."),
                prop("close_delay", "u32")
                    .default("0")
                    .doc("Milliseconds the bubble lingers after the pointer leaves."),
                prop("open", "bool")
                    .default("unset")
                    .doc("Forces the bubble open or closed; unset leaves it to hover and focus."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Renders `children` bare - no wrapper, no bubble."),
                prop("label_id", "String")
                    .doc("The bubble's `id`, so the trigger can carry `aria-describedby`."),
                prop("children", "Element")
                    .doc("The trigger. Note that `class`, `sx`, `states` and spread attributes style the *bubble*, not this."),
            ])],
            lead: rsx! {
                Text {
                    "A label that appears while its child is hovered or focused. Pure CSS - it "
                    "wraps the trigger in a "
                    Code { source: "span" }
                    " and needs no state, so there are no open/close callbacks. "
                    Code { source: "gap" }
                    " is rendered as transparent padding, not empty space, so the pointer can "
                    "travel from the trigger into the bubble without it closing. "
                    Code { source: "sx" }
                    ", "
                    Code { source: "class" }
                    ", "
                    Code { source: "states" }
                    " and spread attributes land on the bubble, not the wrapper."
                }
            },
            Demo {
                component: "Tooltip",
                children_text: "",
                children_code: TRIGGER.to_string(),
                fixed: vec![LABEL.to_string()],
                controls: vec![
                    Control::toggle("side", ["top", "right", "bottom", "left"])
                        .default(theme.tooltip.side.as_str()),
                    Control::slider("size", SIZES).default(theme.tooltip.size.as_str()),
                    Control::slider("gap", SIZES).default(theme.tooltip.gap.as_str()),
                    Control::slider("open_delay", ["auto", "200", "500", "1000"])
                        .code(delay_code),
                    Control::slider("close_delay", ["auto", "200", "500", "1000"])
                        .code(delay_code),
                    // `Some(false)` pins it *shut*, which hover cannot
                    // override - a different thing from leaving it unset.
                    Control::toggle("open", ["auto", "true", "false"]).code(
                        |_, values| match values.str("open").as_str() {
                            "auto" => vec![],
                            open => vec![format!("open: {open}")],
                        },
                    ),
                    Control::switch("disabled"),
                    // `sx` lands on the bubble, not the wrapper - the
                    // bubble is the part worth styling.
                    Control::switch("styled").code(|_, values| {
                        match values.str("styled").as_str() {
                            "true" => vec![STYLED.to_string()],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Box {
                        sx: sx().padding("40px"),
                        Tooltip {
                            label: rsx! { "Saves the current draft" },
                            side: values.str("side"),
                            size: values.str("size"),
                            gap: values.str("gap"),
                            open_delay: delay(values.str("open_delay")),
                            close_delay: delay(values.str("close_delay")),
                            open: match values.str("open").as_str() {
                                "auto" => None,
                                open => Some(open == "true"),
                            },
                            disabled: (values.str("disabled") == "true").then_some(true),
                            sx: match values.str("styled").as_str() {
                                "true" => Input::from(
                                    sx().background("primary.6")
                                        .white_space("normal")
                                        .max_width("12rem"),
                                ),
                                _ => Input::None,
                            },
                            Button { variant: "outlined", "Save" }
                        }
                    }
                },
                wrap: Wrap(wrap_room),
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The wrapper is not focusable, so an "
                    Code { source: "aria-describedby" }
                    " on it would never be announced. Give the bubble an id with "
                    Code { source: "label_id" }
                    " and point your own trigger at it instead."
                }
                Tooltip {
                    label_id: "save-tip",
                    label: rsx! { "Saves the current draft" },
                    Button { aria_describedby: "save-tip", "Save" }
                }
                Text {
                    "Escape does not dismiss it, and an ancestor with "
                    Code { source: "overflow: hidden" }
                    " - a scroll container, a card - clips it. Both need measurement and state; "
                    "reach for a popover there."
                }
            }
        }
    }
}
