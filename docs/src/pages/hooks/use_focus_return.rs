use crate::Route;
use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Checkbox, Code, Flex, Text},
    hooks::use_focus_return,
    sx::sx,
};

/// The hook call and the panel it returns focus from, as `Filters` renders them.
fn code(_: &DemoValues, _: &str) -> String {
    r#"let mut open = use_signal(|| false);
let mut in_stock = use_signal(|| false);
let trigger = use_focus_return();

rsx! {
    Flex { direction: "column", align: "flex-start", gap: "sm",
        Button {
            variant: "outlined",
            aria_expanded: open(),
            aria_controls: "filters-panel",
            onclick: move |_| {
                // Arm on every open, while the trigger still has focus.
                if !open() {
                    trigger.remember_active();
                }
                open.toggle();
            },
            "Filters"
        }
        if open() {
            Flex {
                id: "filters-panel",
                direction: "column",
                align: "flex-start",
                gap: "sm",
                sx: sx().padding("md").background("muted.1").border_radius("8px"),
                onkeydown: move |event: KeyboardEvent| {
                    if event.key() == Key::Escape {
                        open.set(false);
                        trigger.restore();
                    }
                },
                Checkbox {
                    label: "In stock only",
                    checked: in_stock(),
                    onchange: move |next| in_stock.set(next),
                }
                Button {
                    onclick: move |_| {
                        open.set(false);
                        trigger.restore();
                    },
                    "Apply"
                }
            }
        }
    }
}"#
    .to_string()
}

#[component]
fn Filters() -> Element {
    let mut open = use_signal(|| false);
    let mut in_stock = use_signal(|| false);
    let trigger = use_focus_return();

    rsx! {
        Flex { direction: "column", align: "flex-start", gap: "sm",
            Button {
                variant: "outlined",
                aria_expanded: open(),
                aria_controls: "filters-panel",
                onclick: move |_| {
                    if !open() {
                        trigger.remember_active();
                    }
                    open.toggle();
                },
                "Filters"
            }
            if open() {
                Flex {
                    id: "filters-panel",
                    direction: "column",
                    align: "flex-start",
                    gap: "sm",
                    sx: sx().padding("md").background("muted.1").border_radius("8px"),
                    onkeydown: move |event: KeyboardEvent| {
                        if event.key() == Key::Escape {
                            open.set(false);
                            trigger.restore();
                        }
                    },
                    Checkbox {
                        label: "In stock only",
                        checked: in_stock(),
                        onchange: move |next| in_stock.set(next),
                    }
                    Button {
                        onclick: move |_| {
                            open.set(false);
                            trigger.restore();
                        },
                        "Apply"
                    }
                }
            }
        }
    }
}

#[component]
pub fn UseFocusReturnPage() -> Element {
    rsx! {
        DocPage {
            title: "use_focus_return",
            source: "libero/src/hooks/focus_return.rs",
            markdown: "/md/use_focus_return.md",
            lead: rsx! {
                Text {
                    Code { source: "use_focus_return() -> FocusReturn" }
                    " puts focus back where it came from once a panel or popup closes. "
                    "Without it, focus inside a closing panel drops to the document body "
                    "and a keyboard user loses their place. The overlays in libero do "
                    "this already; use the hook for a panel of your own."
                }
                Text {
                    "Call "
                    Code { source: "remember_active()" }
                    " in the handler that opens, and "
                    Code { source: "restore()" }
                    " wherever it closes. "
                    Code { source: "restore()" }
                    " consumes what "
                    Code { source: "remember_active()" }
                    " saved, so arm it on every open. "
                    Code { source: "remember(event)" }
                    " on a trigger's "
                    Code { source: "onmounted" }
                    " names an element instead, which stays armed. "
                    Code { source: "fallback(handle)" }
                    " names where focus goes if the trigger is gone by then, such as the "
                    "list a deleted row lived in."
                }
            },

            Demo {
                component: "use_focus_return",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Filters {} },
                wrap: Wrap(code),
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "Tab into the panel, then press Apply or Escape, and focus lands on "
                    "Filters again. Some browsers do not focus a button on a mouse click, so "
                    "a panel opened with the mouse may remember the body. That only matters "
                    "to a keyboard user, and for them the trigger has focus. "
                    Anchor { to: Route::CollapsePage {}, "Collapse" }
                    " shows the same return on an animated panel."
                }
            }

            DocSection {
                title: "Web and native",
                Text {
                    Code { source: "remember_active()" }
                    " reads the focused element from the document, which the web and Blitz "
                    "have and a webview does not. There it remembers nothing, so name the "
                    "trigger with "
                    Code { source: "remember(event)" }
                    " instead."
                }
            }
        }
    }
}
