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
        "start" => Side::Start,
        "end" => Side::End,
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
                    .default("required")
                    .doc("What the card shows. Links and buttons are fine."),
                prop("children", "Element")
                    .default("required")
                    .doc("The trigger. It must hold a link or a button, since focus is the keyboard's only way to open the card."),
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
                    .doc("Milliseconds the card waits after the pointer leaves. The pointer needs this time to reach the card, so `0` makes it unreachable."),
                prop("open", "bool")
                    .default("unset")
                    .doc("Forces the card open or closed. Unset, hover and focus decide. A card forced open cannot be dismissed."),
                prop("radius", "Size")
                    .default(theme.hover_card.radius.as_str())
                    .doc("The card's corner radius."),
                prop("shadow", "Size")
                    .default(theme.hover_card.shadow.as_str())
                    .doc("The card's elevation."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Renders `children` alone, with no card."),
            ])],
            lead: rsx! {
                Text {
                    "A card that opens while its trigger is hovered or focused. It stays open "
                    "while the pointer or focus is inside, so its links and buttons work. Unlike "
                    Code { source: "Tooltip" }
                    ", it is a named dialog. It flips when its side has no room, and "
                    Kbd { "Esc" }
                    " closes it. "
                    Code { source: "sx" }
                    ", "
                    Code { source: "class" }
                    " and extra attributes land on the card, not the trigger."
                }
            },
            Demo {
                component: "HoverCard",
                children_text: "",
                fixed: vec![CONTENT.to_string()],
                code_child: Child(|_| TRIGGER.to_string()),
                wrap: Wrap(wrap),
                controls: vec![
                    Control::toggle("side", ["top", "end", "bottom", "start"])
                        .default("bottom")
                        .code(enum_code),
                    Control::toggle("align", ["start", "center", "end"])
                        .default("start")
                        .code(enum_code),
                    Control::slider("open_delay", ["auto", "200", "500", "1000"])
                        .code(delay_code),
                    Control::slider("close_delay", ["auto", "0", "300", "1000"])
                        .code(delay_code),
                    Control::toggle("open", ["auto", "false", "true"])
                        .labels(["Auto", "Off", "On"])
                        .code(
                        |_, values| match values.str("open").as_str() {
                            "auto" => vec![],
                            open => vec![format!("open: {open}")],
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
                        open: match values.str("open").as_str() {
                            "auto" => None,
                            open => Some(open == "true"),
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
                title: "Accessibility",
                Text {
                    "A hover card is a preview for sighted users. A screen reader does not "
                    "announce it, so keep it to extras the trigger's own target already offers. "
                    "Content a user needs goes in a popover ("
                    Code { source: "use_popover" }
                    ") that a click opens."
                }
                Text {
                    "Focusing the trigger opens the card, so "
                    Code { source: "children" }
                    " must hold a link or a button. " Kbd { "Tab" }
                    " on the trigger moves into the card, and past its last link to whatever "
                    "follows the trigger. " Kbd { "Esc" }
                    " closes it and returns focus to the trigger if focus was inside. Name the "
                    "card with "
                    Code { source: "aria_label" }
                    " or "
                    Code { source: "aria-labelledby" }
                    ". A trigger with nothing focusable and an unnamed card both warn in the "
                    "console."
                }
                Text {
                    "On the web, " Kbd { "Esc" }
                    " closes the card wherever focus is. On desktop and mobile it works only "
                    "while focus is on the trigger or in the card, so a card the pointer opened "
                    "cannot be dismissed from the keyboard there (WCAG 1.4.13). On touch, a tap "
                    "opens it and a tap elsewhere closes it."
                }
            }
        }
    }
}
