use crate::components::{
    Child, Control, Demo, DemoFile, DemoValues, DocPage, DocSection, Wrap, a11y, indent, prop,
    props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Collapse, Flex, Text},
    hooks::use_focus_return,
};

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("collapse.rs"));

const TEXTS: [&str; 3] = [
    "Shipping is calculated at checkout.",
    "Standard delivery arrives in three to five working days.",
    "Returns are free within thirty days of delivery.",
];

fn focus_return(values: &DemoValues) -> bool {
    values.str("focus_return") == "true"
}

/// A one-liner or a paragraph, so the height visibly changes. With focus return the panel also
/// holds the button that closes it.
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
        lines.push(FILE.section("done").to_string());
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

/// Prints the trigger and its `open` signal: the controlled-disclosure contract, and needed
/// for the paste to compile.
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

/// Its own component: a hook in `render` would land in `Demo`'s scope.
/// **Kept in step with `content_code` and `wrap_page` by hand.**
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
                        // demo-code: done start
                        Button {
                            onclick: move |_| {
                                open.set(false);
                                trigger.restore();
                            },
                            "Done"
                        }
                        // demo-code: done end
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
                prop("open", "bool").default("required")
                    .doc("Whether the panel is expanded. You own this state."),
                prop("keep_mounted", "bool")
                    .default("true")
                    .doc("Keeps the children in the DOM while closed, out of the focus order and hidden from screen readers, so a half-typed form survives. `false` removes them once the panel has closed."),
                prop("duration", "u32")
                    .default("200")
                    .doc("Animation length in milliseconds. `0` turns the animation off."),
                prop("children", "Element").doc("The content that grows and shrinks."),
            ])],
            accessibility: a11y()
                .handles([
                    "With `keep_mounted`, the closed children stay in the DOM but out of the focus order and hidden from screen readers.",
                ])
                .must([
                    "Give the trigger `aria_expanded` and an `aria_controls` pointing at the panel's `id`: `Collapse` has no role or ARIA.",
                    "Return focus yourself when the panel closes from inside: use `use_focus_return`, with `remember_active()` on every open and `restore()` where the panel closes.",
                    "Make wide content fit: `Collapse` clips whatever is wider than the panel, with no scrollbar. Let text wrap (`overflow-wrap: anywhere`) and put a wide table or code block in a box with `overflow-x: auto`.",
                    "Put padding on the content, not on `Collapse`: the root keeps its own padding, border and margin when closed.",
                ])
                .example("A \"Show details\" button with `aria_expanded: open()` and `aria_controls: \"details\"` above a `Collapse { open: open(), id: \"details\", .. }`: a screen reader reads the button as expanded or collapsed, and the closed panel is skipped."),
            lead: rsx! {
                Text {
                    "Animates its children's height open and closed, and follows the "
                    "content when its height changes. You own "
                    Code { source: "open" }
                    ". "
                    Code { source: "Collapse" }
                    " has no role or ARIA, so the trigger carries "
                    Code { source: "aria_expanded" }
                    " and an "
                    Code { source: "aria_controls" }
                    " pointing at the panel's "
                    Code { source: "id" }
                    "."
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

            DocSection {
                title: "Focus",
                Text {
                    "Focus inside a closing panel does not return to the trigger on its "
                    "own. Use "
                    Code { source: "use_focus_return" }
                    ", with "
                    Code { source: "remember_active()" }
                    " on every open and "
                    Code { source: "restore()" }
                    " where the panel closes from inside. Switch on Focus return, open "
                    "the panel with the keyboard and press Done."
                }
            }
        }
    }
}
