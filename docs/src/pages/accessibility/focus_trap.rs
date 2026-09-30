use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, indent, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Button, Flex, FocusTrap, Text},
    hooks::use_focus_return,
};

const TRAPPED: &str = r#"Flex {
    direction: "row",
    gap: "sm",
    Button { variant: "outlined", "First" }
    Button { variant: "outlined", "Second" }
    Button { variant: "outlined", "Third" }
}"#;

/// The switch is whether the `FocusTrap` exists at all. The two outside buttons always print:
/// "focus cannot leave" is measured against them.
fn wrap_page(values: &DemoValues, code: &str) -> String {
    let inner = match values.str("activate_focus_trap") == "true" {
        true => indent(code),
        false => indent(TRAPPED),
    };
    format!(
        "Flex {{\n    direction: \"column\",\n    gap: \"sm\",\n    align: \"flex-start\",\n    Button {{ variant: \"text\", \"Before\" }}\n{inner}    Button {{ variant: \"text\", \"After\" }}\n}}"
    )
}

#[component]
pub fn FocusTrapPage() -> Element {
    rsx! {
        DocPage {
            title: "FocusTrap",
            source: "libero/src/components/accessibility/focus_trap.rs",
            markdown: "/md/focus_trap.md",
            properties: vec![props("FocusTrap", vec![
                prop("children", "Element")
                    .default("required")
                    .doc("The content that keeps the focus."),
            ])],
            accessibility: a11y()
                .key(["Tab", "Shift+Tab"], "Cycles through the focusable children, wrapping at either end.")
                .handles([
                    "On mount the trap focuses the element marked `data-autofocus`, or else its first focusable child.",
                ])
                .must([
                    "To focus nothing visible, so a dialog does not open with its first button looking pressed, render `FocusTrapInitialFocus` as the first child.",
                    "Keep a trap only around content that is the one thing that matters on screen, such as an open overlay. A keyboard user who cannot Tab out of a region has no way back to the page.",
                ]),
            lead: rsx! {
                Text {
                    "Keeps Tab and Shift+Tab cycling inside its children, as inside an open "
                    "dialog. It focuses its first focusable child on mount and adds no box of "
                    "its own. Switch it on below, and Tab cycles First, Second, Third and "
                    "Release without reaching Before or After. Release or Escape switches "
                    "it off and puts focus back on the switch."
                }
            },
            Demo {
                component: "FocusTrap",
                children_text: "",
                children_code: TRAPPED.to_string(),
                controls: vec![
                    // Not a prop - `FocusTrap` has none - so it prints
                    // nothing and `wrap_page` drops the component itself.
                    Control::switch("activate_focus_trap").code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    Preview { values }
                },
                wrap: Wrap(wrap_page),
            }
        }
    }
}

/// Before, the three buttons and After. On, the buttons sit in a trap that Release or
/// Escape switches off, handing focus back to the switch (2.1.2, todo 1528).
#[component]
fn Preview(values: DemoValues) -> Element {
    let back = use_focus_return();
    let release = {
        let values = values.clone();
        move || {
            values.set("activate_focus_trap", "false");
            back.restore();
        }
    };
    let on_escape = release.clone();
    let on_click = release;

    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            align: "flex-start",
            Button { variant: "text", "Before" }
            if values.str("activate_focus_trap") == "true" {
                FocusTrap {
                    Remember { onrender: move |_| back.remember_active() }
                    Flex {
                        direction: "row",
                        gap: "sm",
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Escape {
                                on_escape();
                            }
                        },
                        Button { variant: "outlined", "First" }
                        Button { variant: "outlined", "Second" }
                        Button { variant: "outlined", "Third" }
                        Button { variant: "filled", onclick: move |_| on_click(), "Release" }
                    }
                }
            } else {
                Flex {
                    direction: "row",
                    gap: "sm",
                    Button { variant: "outlined", "First" }
                    Button { variant: "outlined", "Second" }
                    Button { variant: "outlined", "Third" }
                }
            }
            Button { variant: "text", "After" }
        }
    }
}

/// Remembers what held focus as the trap renders, before the trap's mount moves it.
#[component]
fn Remember(onrender: Callback) -> Element {
    use_hook(|| onrender.call(()));
    rsx! {}
}
