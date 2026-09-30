use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Box, Button, Code, Input, Text, Tooltip},
    sx::sx,
    use_theme,
};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

// snippet: in Tooltip { .., Button { "Save" } }
const LABEL: &str = r#"label: rsx! { "Saves the current draft" }"#;
// The trigger points at the bubble, so a copied snippet is described too.
// snippet: in Tooltip { label: rsx! { "Save" }, .., Button { "Save" } }
const LABEL_ID: &str = r#"label_id: "draft-tip""#;
const TRIGGER: &str = r#"Button { variant: "outlined", aria_describedby: "draft-tip", "Save" }"#;
// snippet: in Tooltip { label: rsx! { "Save" }, .., Button { "Save" } }
const STYLED: &str = r#"sx: sx().background("primary.6").white_space("normal").max_width("12rem")"#;

/// A bubble's worth of room on every side, so the preview shows the chosen
/// side rather than a flip.
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
                prop("label", "Element").default("required").doc("The bubble's content."),
                prop("side", "Side")
                    .default("top")
                    .doc("The preferred side of the trigger. The bubble flips when that side has no room."),
                prop("gap", "Size")
                    .default("xs")
                    .doc("Distance to the trigger. The pointer can cross it without closing the bubble."),
                prop("size", "Size").default("sm").doc("Font size of the bubble."),
                prop("z_index", "ThemeAwareValue")
                    .default("the popover layer")
                    .doc("Overrides the stacking level, for a bubble hidden by another overlay."),
                prop("open_delay", "u32")
                    .default("0")
                    .doc("Milliseconds the pointer must rest before the bubble appears."),
                prop("close_delay", "u32")
                    .default("0")
                    .doc("Milliseconds the bubble stays after the pointer leaves. While it counts down, the bubble carries `data-closing`."),
                prop("open", "bool")
                    .default("unset")
                    .doc("Forces the bubble open or closed. Unset, hover and focus decide. A bubble forced open ignores Escape."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Renders `children` alone, with no wrapper and no bubble."),
                prop("label_id", "String")
                    .doc("The bubble's `id`, for the trigger's `aria-describedby`. It exists while the bubble is closed, too."),
                prop("children", "Element")
                    .default("required")
                    .doc("The trigger."),
            ])],
            accessibility: a11y()
                .key(["Escape"], "Hides the bubble until the pointer or focus comes back.")
                .handles([
                    "Keyboard focus anywhere inside `Tooltip` shows the bubble, a click does not.",
                    "On touch, a tap shows nothing: a 500 ms hold shows the bubble, and it stays 1.5 s after the release.",
                ])
                .must([
                    "Give the bubble an id with `label_id` and point your trigger's `aria-describedby` at it, so a screen reader reads the label. The demo's Save button does.",
                ]),
            lead: rsx! {
                Text {
                    "A label that appears while its child is hovered or focused by keyboard. "
                    "The bubble is portaled, so no "
                    Code { source: "overflow: hidden" }
                    " ancestor clips it, and it flips when its side has no room. "
                    Code { source: "sx" }
                    ", "
                    Code { source: "class" }
                    ", "
                    Code { source: "states" }
                    " and extra attributes land on the bubble, not the trigger."
                }
            },
            Demo {
                component: "Tooltip",
                children_text: "",
                children_code: TRIGGER.to_string(),
                fixed: vec![LABEL.to_string(), LABEL_ID.to_string()],
                controls: vec![
                    Control::toggle("side", ["top", "end", "bottom", "start"])
                        .labels(["Top", "End", "Bottom", "Start"])
                        .default(theme.tooltip.side.as_str()),
                    Control::slider("size", SIZES).default(theme.tooltip.size.as_str()),
                    Control::slider("gap", SIZES).default(theme.tooltip.gap.as_str()),
                    Control::slider("open_delay", ["auto", "200", "500", "1000"])
                        .code(delay_code),
                    Control::slider("close_delay", ["auto", "200", "500", "1000"])
                        .code(delay_code),
                    // `Some(false)` pins it shut, which hover cannot override.
                    Control::toggle("open", ["auto", "false", "true"])
                        .labels(["Auto", "Off", "On"])
                        .code(
                        |_, values| match values.str("open").as_str() {
                            "auto" => vec![],
                            open => vec![format!("open: {open}")],
                        },
                    ),
                    Control::switch("disabled"),
                    // `sx` lands on the bubble, the part worth styling.
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
                            label_id: "draft-tip",
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
                            Button { variant: "outlined", aria_describedby: "draft-tip", "Save" }
                        }
                    }
                },
                wrap: Wrap(wrap_room),
            }
        }
    }
}
