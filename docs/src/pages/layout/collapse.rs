use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, DocSection, Wrap, indent, prop, props,
};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, CodeBlock, Collapse, Flex, Text},
    hooks::use_focus_return,
    sx::sx,
};

const SHORT: &str = r#"Text { "Shipping is calculated at checkout." }"#;

const LONG: &str = r#"Flex {
    direction: "column",
    gap: "sm",
    Text { "Shipping is calculated at checkout." }
    Text { "Standard delivery arrives in three to five working days." }
    Text { "Returns are free within thirty days of delivery." }
}"#;

/// The height only visibly changes if the content's height does, so the two
/// options are a one-liner and a paragraph rather than two of the same size.
fn content_code(values: &DemoValues) -> String {
    match values.str("content").as_str() {
        "long" => LONG.to_string(),
        _ => SHORT.to_string(),
    }
}

/// `open` is not a control: it is a signal the trigger toggles, which is the
/// whole controlled-disclosure contract. The preview renders that button, so
/// the snippet has to print it - along with the `use_signal` behind it, or the
/// paste does not compile.
fn wrap_page(_: &DemoValues, source: &str) -> String {
    let collapse = indent(&indent(source));
    format!(
        "let mut open = use_signal(|| false);\n\n\
         rsx! {{\n    \
         Flex {{\n        \
         direction: \"column\",\n        \
         align: \"flex-start\",\n        \
         gap: \"sm\",\n        \
         Button {{\n            \
         onclick: move |_| open.toggle(),\n            \
         aria_expanded: open(),\n            \
         aria_controls: \"shipping-details\",\n            \
         \"Shipping details\"\n        \
         }}\n\
         {collapse}    }}\n}}"
    )
}

/// Printed verbatim below the live example, which renders exactly this.
const FOCUS_RETURN: &str = r#"let mut open = use_signal(|| false);
let trigger = use_focus_return();

rsx! {
    Button {
        onmounted: move |event| trigger.remember(event),
        onclick: move |_| open.toggle(),
        aria_expanded: open(),
        aria_controls: "returning-panel",
        "Edit address"
    }
    Collapse {
        id: "returning-panel",
        open: open(),
        keep_mounted: false,
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Text { "Focus this button, then close the panel with it." }
            Button {
                onclick: move |_| {
                    open.set(false);
                    trigger.restore();
                },
                "Done"
            }
        }
    }
}"#;

/// The pattern the section describes, rendered so it can actually be tabbed
/// through - a closing panel that hands focus back instead of dropping it.
#[component]
fn FocusReturnDemo() -> Element {
    let mut open = use_signal(|| false);
    let trigger = use_focus_return();

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button {
                onmounted: move |event| trigger.remember(event),
                onclick: move |_| open.toggle(),
                aria_expanded: open(),
                aria_controls: "returning-panel",
                "Edit address"
            }
            Collapse {
                id: "returning-panel",
                open: open(),
                keep_mounted: false,
                Flex {
                    direction: "column",
                    align: "flex-start",
                    gap: "sm",
                    Text { "Focus this button, then close the panel with it." }
                    Button {
                        onclick: move |_| {
                            open.set(false);
                            trigger.restore();
                        },
                        "Done"
                    }
                }
            }
        }
    }
}

/// The `use_signal` lives here rather than in `Demo`'s `render` closure, which
/// runs in `Demo`'s own scope - a hook written there lands in `Demo`'s hook
/// slots.
#[component]
fn CollapseDemo(keep_mounted: Option<bool>, duration: Option<u32>, long: bool) -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            gap: "sm",
            Button {
                onclick: move |_| open.toggle(),
                aria_expanded: open(),
                aria_controls: "shipping-details",
                "Shipping details"
            }
            Collapse {
                id: "shipping-details",
                open: open(),
                keep_mounted,
                duration,
                if long {
                    Flex {
                        direction: "column",
                        gap: "sm",
                        Text { "Shipping is calculated at checkout." }
                        Text { "Standard delivery arrives in three to five working days." }
                        Text { "Returns are free within thirty days of delivery." }
                    }
                } else {
                    Text { "Shipping is calculated at checkout." }
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
                    Control::switch("keep_mounted").default("true"),
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
                        }
                    }
                },
            }
            DocSection {
                title: "What closed content costs",
                Text {
                    "A closed panel keeps its children in the DOM by default, so a half-typed "
                    "form survives being collapsed and a trigger's "
                    Code { source: "aria-controls" }
                    " always resolves - the root element is present whatever "
                    Code { source: "keep_mounted" }
                    " says. What closed content is not is reachable: the inner box is "
                    Code { source: "visibility: hidden" }
                    ", which takes it out of the focus order and out of the accessibility "
                    "tree, and that step is delayed by the animation's duration so the panel "
                    "stays visible and announced for the whole close rather than vanishing on "
                    "the first frame."
                }
                Text {
                    sx: sx().margin_top("sm"),
                    Code { source: "keep_mounted: false" }
                    " goes further and removes the children once the exit transition ends. "
                    "Use it when there is nothing to preserve - content built from a closure, "
                    "or a long list you would rather not pay for while it is hidden - and "
                    "expect the state inside to be gone on reopen."
                }
                Text {
                    sx: sx().margin_top("sm"),
                    "One degradation to know about: under "
                    Code { source: "prefers-reduced-motion: reduce" }
                    " there is no transition, so no "
                    Code { source: "transitionend" }
                    " ever arrives and a "
                    Code { source: "keep_mounted: false" }
                    " panel keeps its children after the close. They are still "
                    Code { source: "visibility: hidden" }
                    ", so neither focusable nor announced - the mode degrades to "
                    Code { source: "keep_mounted: true" }
                    " rather than breaking, at the cost of the DOM weight and the retained "
                    "state. A "
                    Code { source: "duration" }
                    " of "
                    Code { source: "0" }
                    " has the same missing event and is handled: the panel unmounts straight "
                    "from "
                    Code { source: "open" }
                    "."
                }
                Text {
                    sx: sx().margin_top("sm"),
                    "One more limit of that mode: a "
                    Code { source: "Collapse" }
                    " nested inside another one's content can unmount the outer one's "
                    "children early. "
                    Code { source: "transitionend" }
                    " bubbles, and the filter discriminates on the property name rather than "
                    "on the element the event came from, so an inner panel's event reaches the "
                    "outer root. If both are closing and the inner one is faster, the outer "
                    "unmounts at the inner one's end time."
                }
                Text {
                    sx: sx().margin_top("sm"),
                    Code { source: "Collapse" }
                    " renders no role and no ARIA, so the trigger carries the disclosure "
                    "semantics - that is what the example above wires: "
                    Code { source: "aria_expanded" }
                    " on the button and an "
                    Code { source: "aria_controls" }
                    " pointing at an "
                    Code { source: "id" }
                    " you set on the "
                    Code { source: "Collapse" }
                    ", which resolves whether the panel is open, closed, or unmounted."
                }
            }
            DocSection {
                title: "Returning focus when it closes",
                Text {
                    "If focus is inside the panel when it closes, it does not come back on its "
                    "own - the browser drops it to the document body, and a keyboard user "
                    "loses their place. "
                    Code { source: "Collapse" }
                    " cannot fix this for you: it never sees the trigger, so it has no element "
                    "to hand focus back to. Whoever owns both the trigger and the panel owns "
                    "the return."
                }
                Text {
                    sx: sx().margin_top("sm"),
                    "Use "
                    Code { source: "use_focus_return" }
                    " rather than reaching for the element yourself - it is the one focus-return "
                    "contract in the library, and it spawns the focus call, which matters "
                    "because focusing inside the dispatch of the event that closed the panel "
                    "re-enters a handler whose click is still bubbling. Name the trigger from "
                    "its "
                    Code { source: "onmounted" }
                    ", then call "
                    Code { source: "restore()" }
                    " wherever you close the panel from inside it. Tab to the Done button below "
                    "and press it: focus lands back on the trigger, not on the body."
                }
                FocusReturnDemo {}
                CodeBlock { source: FOCUS_RETURN, language: "rust" }
            }
        }
    }
}
