use crate::components::{Child, Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Collapse, Flex, Text},
    hooks::use_focus_return,
};

const TEXTS: [&str; 3] = [
    "Shipping is calculated at checkout.",
    "Standard delivery arrives in three to five working days.",
    "Returns are free within thirty days of delivery.",
];

const DONE: &str = r#"Button {
    onclick: move |_| {
        open.set(false);
        trigger.restore();
    },
    "Done"
}"#;

fn focus_return(values: &DemoValues) -> bool {
    values.str("focus_return") == "true"
}

/// The height only visibly changes if the content's height does, so the two
/// options are a one-liner and a paragraph rather than two of the same size.
/// With focus return the panel also holds the button that closes it.
fn content_code(values: &DemoValues) -> String {
    let count = match values.str("content").as_str() {
        "long" => 3,
        _ => 1,
    };
    let mut lines: Vec<String> = TEXTS[..count]
        .iter()
        .map(|text| format!("Text {{ \"{text}\" }}"))
        .collect();
    if focus_return(values) {
        lines.push(DONE.to_string());
    } else if count == 1 {
        return lines.remove(0);
    }
    let align = match focus_return(values) {
        true => "    align: \"flex-start\",\n",
        false => "",
    };
    format!(
        "Flex {{\n    direction: \"column\",\n{align}    gap: \"sm\",\n{}}}",
        indent(&lines.join("\n"))
    )
}

/// `open` is not a control: it is a signal the trigger toggles, which is the
/// whole controlled-disclosure contract. The preview renders that button, so
/// the snippet has to print it - along with the `use_signal` behind it, or the
/// paste does not compile.
fn wrap_page(values: &DemoValues, source: &str) -> String {
    let collapse = indent(&indent(source));
    let (hook, onclick) = match focus_return(values) {
        // Armed on the opening edge, every open: `restore()` takes the
        // trigger and forgets it, so arming once would work once.
        true => (
            "let trigger = use_focus_return();\n",
            "onclick: move |_| {\n                \
             if !open() {\n                    \
             trigger.remember_active();\n                \
             }\n                \
             open.toggle();\n            \
             },",
        ),
        false => ("", "onclick: move |_| open.toggle(),"),
    };
    format!(
        "let mut open = use_signal(|| false);\n{hook}\n\
         rsx! {{\n    \
         Flex {{\n        \
         direction: \"column\",\n        \
         align: \"flex-start\",\n        \
         gap: \"sm\",\n        \
         Button {{\n            \
         {onclick}\n            \
         aria_expanded: open(),\n            \
         aria_controls: \"shipping-details\",\n            \
         \"Shipping details\"\n        \
         }}\n\
         {collapse}    }}\n}}"
    )
}

/// The `use_signal` lives here rather than in `Demo`'s `render` closure, which
/// runs in `Demo`'s own scope - a hook written there lands in `Demo`'s hook
/// slots.
///
/// **Kept in step with `content_code` and `wrap_page` by hand**; change one and
/// change the other, or the snippet stops reproducing the preview.
#[component]
fn CollapseDemo(
    keep_mounted: Option<bool>,
    duration: Option<u32>,
    long: bool,
    focus_return: bool,
) -> Element {
    let mut open = use_signal(|| false);
    let trigger = use_focus_return();
    let texts = &TEXTS[..if long { 3 } else { 1 }];

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button {
                onclick: move |_| {
                    if focus_return && !open() {
                        trigger.remember_active();
                    }
                    open.toggle();
                },
                aria_expanded: open(),
                aria_controls: "shipping-details",
                "Shipping details"
            }
            Collapse {
                id: "shipping-details",
                open: open(),
                keep_mounted,
                duration,
                if focus_return {
                    Flex {
                        direction: "column",
                        align: "flex-start",
                        gap: "sm",
                        for text in texts {
                            Text { "{text}" }
                        }
                        Button {
                            onclick: move |_| {
                                open.set(false);
                                trigger.restore();
                            },
                            "Done"
                        }
                    }
                } else if long {
                    Flex {
                        direction: "column",
                        gap: "sm",
                        for text in texts {
                            Text { "{text}" }
                        }
                    }
                } else {
                    Text { "{TEXTS[0]}" }
                }
            }
        }
    }
}

#[component]
pub fn CollapsePage() -> Element {
    rsx! {
        DocPage {
            title: "Collapse",
            source: "libero/src/components/layout/collapse.rs",
            markdown: "/md/collapse.md",
            properties: vec![props("Collapse", vec![
                prop("open", "bool")
                    .doc("Whether the panel is expanded. Strictly controlled - `Collapse` holds no open state of its own."),
                prop("keep_mounted", "bool")
                    .default("true")
                    .doc("Keep the children in the DOM while closed. `false` unmounts them when the exit transition ends."),
                prop("duration", "u32")
                    .default("theme.collapse.duration")
                    .doc("Milliseconds. `0` disables the animation."),
                prop("children", "Element").doc("The content that grows and shrinks."),
            ])],
            lead: rsx! {
                Text {
                    "Animates its children's height open and closed. It renders two "
                    Code { source: "div" }
                    "s - a grid whose single row goes from "
                    Code { source: "0fr" }
                    " to "
                    Code { source: "1fr" }
                    ", and the clipped box holding your content - so the animation "
                    "re-measures itself for free whenever the content's own height changes. "
                    Code { source: "open" }
                    " is strictly controlled, and "
                    Code { source: "Collapse" }
                    " renders no role and no ARIA of its own: the disclosure semantics "
                    "belong to whatever owns the trigger."
                }
                Text {
                    "Focus inside a closing panel is not handed back: it never sees the "
                    "trigger. Whoever owns both returns it with "
                    Code { source: "use_focus_return" }
                    " - arm it with "
                    Code { source: "remember_active()" }
                    " on every open, since "
                    Code { source: "restore()" }
                    " forgets the trigger, and call "
                    Code { source: "restore()" }
                    " wherever the panel closes from inside. Switch "
                    Code { source: "Focus return" }
                    " on, open the panel with the keyboard and press Done."
                }
            },
            Demo {
                component: "Collapse",
                children_text: "",
                code_child: Child(content_code),
                fixed: vec![
                    "id: \"shipping-details\"".to_string(),
                    "open: open()".to_string(),
                ],
                controls: vec![
                    Control::slider("duration", ["0", "100", "200", "600", "1200"])
                        .default("200")
                        .labels(["0ms", "100ms", "200ms", "600ms", "1200ms"])
                        .code(|control, values| {
                            let value = values.str("duration");
                            match value == control.default {
                                true => vec![],
                                false => vec![format!("duration: {value}")],
                            }
                        }),
                    // Not a prop - it picks the children, which the code block
                    // prints through `code_child`.
                    Control::toggle("content", ["short", "long"])
                        .labels(["Short", "Long"])
                        .code(|_, _| vec![]),
                    Control::switch("keep_mounted").default("true"),
                    // Not a prop - the caller's wiring, printed by `wrap_page`
                    // and `code_child`.
                    Control::switch("focus_return").code(|_, _| vec![]),
                ],
                wrap: Wrap(wrap_page),
                render: move |values: DemoValues| {
                    let keep_mounted = match values.str("keep_mounted").as_str() {
                        "false" => Some(false),
                        _ => None,
                    };
                    let duration = match values.str("duration").as_str() {
                        "200" => None,
                        value => value.parse::<u32>().ok(),
                    };

                    rsx! {
                        CollapseDemo {
                            keep_mounted,
                            duration,
                            long: values.str("content") == "long",
                            focus_return: focus_return(&values),
                        }
                    }
                },
            }
        }
    }
}
