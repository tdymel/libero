use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, Flex, HoverCard, Kbd, Text},
    hooks::{Align, Side},
    sx::sx,
    use_theme,
};

const HREF: &str = "https://en.wikipedia.org/wiki/Ada_Lovelace";

// snippet: in HoverCard { .., Button { "Ada Lovelace" } }
const CONTENT: &str = r#"aria_label: "Ada Lovelace",
content: rsx! {
    Flex {
        gap: "xs",
        sx: sx().max_width("16rem"),
        Text { sx: sx().font_weight("600"), "Ada Lovelace" }
        Text { size: "sm", "Wrote the first algorithm meant for a machine, in 1843." }
        Anchor { to: "https://en.wikipedia.org/wiki/Ada_Lovelace", target: "_blank", "Read more" }
    }
}"#;

const TRIGGER: &str = r#"Button { variant: "outlined", "Ada Lovelace" }"#;

fn wrap(_: &DemoValues, source: &str) -> String {
    format!("rsx! {{\n{}}}", indent(source))
}

fn side_of(value: &str) -> Side {
    match value {
        "top" => Side::Top,
        "left" => Side::Left,
        "right" => Side::Right,
        _ => Side::Bottom,
    }
}

fn align_of(value: &str) -> Align {
    match value {
        "center" => Align::Center,
        "end" => Align::End,
        _ => Align::Start,
    }
}

/// `side` and `align` are the popover's enums, not strings, so the default
/// printer's `side: "top"` would not compile.
fn enum_code(control: &Control, values: &DemoValues) -> Vec<String> {
    let value = values.str(control.name);
    if value == control.default {
        return vec![];
    }
    let (kind, variant) = match control.name {
        "side" => ("Side", format!("{:?}", side_of(&value))),
        _ => ("Align", format!("{:?}", align_of(&value))),
    };
    vec![format!("{}: {kind}::{variant}", control.name)]
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
pub fn HoverCardPage() -> Element {
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "HoverCard",
            source: "libero/src/components/overlay/hover_card.rs",
            markdown: "/md/hover_card.md",
            properties: vec![props("HoverCard", vec![
                prop("content", "Element")
                    .doc("What the card shows. It may hold links and buttons - the card is a non-modal dialog, not a tooltip."),
                prop("children", "Element")
                    .doc("The trigger. Note that `class`, `sx`, `states` and spread attributes style the *card*, not this."),
                prop("side", "Side")
                    .default("Bottom")
                    .doc("Which side of the trigger the card opens on. It flips when that side has no room."),
                prop("align", "Align")
                    .default("Start")
                    .doc("Where the card lines up along that side."),
                prop("open_delay", "u32")
                    .default(theme.hover_card.open_delay.to_string())
                    .doc("Milliseconds the pointer must rest on the trigger before the card opens."),
                prop("close_delay", "u32")
                    .default(theme.hover_card.close_delay.to_string())
                    .doc("Milliseconds the card waits after the pointer leaves. It is also the time the pointer has to cross into the card, so `0` makes the card unreachable by pointer."),
                prop("opened", "bool")
                    .default("unset")
                    .doc("Forces the card open or closed; unset leaves it to hover and focus. A card forced open cannot be dismissed."),
                prop("radius", "Size")
                    .default(theme.hover_card.radius.as_str())
                    .doc("The card's corner radius."),
                prop("shadow", "Size")
                    .default(theme.hover_card.shadow.as_str())
                    .doc("The card's elevation."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Renders `children` bare - no wrapper, no card."),
            ])],
            lead: rsx! {
                Text {
                    "A card that opens while its trigger is hovered or focused, and stays open "
                    "while the pointer or focus is inside it, so its links and buttons can be "
                    "used. Unlike "
                    Code { source: "Tooltip" }
                    " it is a "
                    Code { source: "role=\"dialog\"" }
                    " with a name of its own, is portaled so no "
                    Code { source: "overflow: hidden" }
                    " ancestor clips it, flips when its side has no room, and closes on "
                    Kbd { "Esc" }
                    ". The surface is the theme's "
                    Code { source: "paper" }
                    "; "
                    Code { source: "sx" }
                    ", "
                    Code { source: "class" }
                    " and spread attributes land on the card."
                }
            },
            Demo {
                component: "HoverCard",
                children_text: "",
                fixed: vec![CONTENT.to_string()],
                code_child: Child(|_| TRIGGER.to_string()),
                wrap: Wrap(wrap),
                controls: vec![
                    Control::toggle("side", ["bottom", "top", "right", "left"])
                        .default("bottom")
                        .code(enum_code),
                    Control::toggle("align", ["start", "center", "end"])
                        .default("start")
                        .code(enum_code),
                    Control::slider("open_delay", ["auto", "200", "500", "1000"])
                        .code(delay_code),
                    Control::slider("close_delay", ["auto", "0", "300", "1000"])
                        .code(delay_code),
                    Control::toggle("opened", ["auto", "true", "false"]).code(
                        |_, values| match values.str("opened").as_str() {
                            "auto" => vec![],
                            opened => vec![format!("opened: {opened}")],
                        },
                    ),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    HoverCard {
                        side: side_of(&values.str("side")),
                        align: align_of(&values.str("align")),
                        open_delay: delay(values.str("open_delay")),
                        close_delay: delay(values.str("close_delay")),
                        opened: match values.str("opened").as_str() {
                            "auto" => None,
                            opened => Some(opened == "true"),
                        },
                        disabled: values.str("disabled") == "true",
                        aria_label: "Ada Lovelace",
                        content: rsx! {
                            Flex {
                                gap: "xs",
                                sx: sx().max_width("16rem"),
                                Text { sx: sx().font_weight("600"), "Ada Lovelace" }
                                Text { size: "sm", "Wrote the first algorithm meant for a machine, in 1843." }
                                Anchor { to: HREF, target: "_blank", "Read more" }
                            }
                        },
                        Button { variant: "outlined", "Ada Lovelace" }
                    }
                },
            }
            DocSection {
                title: "Keyboard and dismissal",
                Text {
                    "Focusing the trigger from the keyboard opens the card; a click does not "
                    "keep it open. " Kbd { "Tab" } " on the trigger moves into the card, "
                    Kbd { "Tab" } " past its last link moves on to whatever follows the "
                    "trigger, and " Kbd { "Shift" } " " Kbd { "Tab" } " walks back. "
                    Kbd { "Esc" } " closes it wherever focus is - even when the pointer opened "
                    "it and focus never left a text field - and hands focus back to the "
                    "trigger when it was inside."
                }
                Text {
                    "The card is a dialog, so name it: "
                    Code { source: "aria_label" }
                    ", or "
                    Code { source: "aria-labelledby" }
                    " pointing into the content. An unnamed card warns in the console."
                }
                Text {
                    "Escape everywhere relies on a document-level key listener, which only the "
                    "web has. On desktop and mobile the card hears "
                    Kbd { "Esc" }
                    " only while focus is on its trigger or inside it, so a card the pointer "
                    "opened cannot be dismissed from the keyboard there - WCAG 1.4.13 is not met "
                    "on those targets. Touch is sticky: a tap opens it, a tap elsewhere closes it."
                }
            }
        }
    }
}
