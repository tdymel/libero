use crate::components::{Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Code, Flex, FocusTrap, Text};

/// The page's own source: the printed code is the live `Preview`'s.
const FILE: DemoFile = DemoFile(include_str!("focus_trap.rs"));

/// Prints `Preview` with its signal: on, Release and Escape switch the trap off, and
/// `restore_focus` hands focus back to the button that switched it on.
fn wrap_page(values: &DemoValues, _code: &str) -> String {
    let on = values.str("activate_focus_trap") == "true";
    format!(
        "let mut trapped = use_signal(|| {on});\n\n{}",
        FILE.section("preview")
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
                prop("restore_focus", "bool")
                    .default("false")
                    .doc("On unmount, hands focus back to whatever held it when the trap mounted."),
            ])],
            accessibility: a11y()
                .key(["Tab", "Shift+Tab"], "Cycles through the focusable children, wrapping at either end.")
                .handles([
                    "On mount the trap focuses the element marked `data-autofocus`, or else its first focusable child.",
                    "With `restore_focus`, unmounting the trap puts focus back where it was before the trap took it.",
                ])
                .must([
                    "To focus nothing visible, so a dialog does not open with its first button looking pressed, render `FocusTrapInitialFocus` as the first child.",
                    "Keep a trap only around content that is the one thing that matters on screen, such as an open overlay. A keyboard user who cannot Tab out of a region has no way back to the page.",
                    "Give the user a way out: the trap has no Escape of its own, so close it on Escape and on a button.",
                ])
                .example("A delete confirmation, `FocusTrap { restore_focus: true, .. }` around its message and two buttons: Tab from the last button wraps to the first, Escape and Cancel close it, and focus goes back to the button that opened it.")
                .limits([
                    "Without `restore_focus`, focus is not restored on unmount: it falls to the page body.",
                    "In a desktop WebView or on Android, Tab and Shift+Tab move between the children but do not wrap: at either end they leave the trap.",
                ]),
            lead: rsx! {
                Text {
                    "Keeps Tab and Shift+Tab cycling inside its children, as inside an open "
                    "dialog. It focuses its first focusable child on mount and adds no box of "
                    "its own. Switch it on below, and Tab cycles First, Second, Third and "
                    "Release without reaching Before or After. Release or Escape switches "
                    "it off, and "
                    Code { source: "restore_focus" }
                    " puts focus back on the button or switch that turned it on."
                }
            },
            Demo {
                component: "FocusTrap",
                children_text: "",
                controls: vec![
                    // Not a prop: whether the trap exists. It prints nothing,
                    // and `wrap_page` drops the component itself.
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

/// Before, the trap and After. The switch and the preview's own buttons move one state; Release
/// or Escape switches the trap off and `restore_focus` hands focus back (2.1.2, todos 1528, 1534).
#[component]
fn Preview(values: DemoValues) -> Element {
    let on = values.str("activate_focus_trap") == "true";
    let mut trapped = use_signal(|| on);
    use_effect(use_reactive!(|on| trapped.set(on)));
    use_effect(move || values.set("activate_focus_trap", trapped().to_string()));

    // demo-code: preview start
    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            align: "flex-start",
            Button { variant: "text", "Before" }
            // Stays mounted, so `restore_focus` has it to return to.
            Button { variant: "outlined", onclick: move |_| trapped.set(true), "Trap focus" }
            if trapped() {
                FocusTrap {
                    restore_focus: true,
                    Flex {
                        direction: "row",
                        gap: "sm",
                        onkeydown: move |event: KeyboardEvent| {
                            if event.key() == Key::Escape {
                                trapped.set(false);
                            }
                        },
                        Button { variant: "outlined", "First" }
                        Button { variant: "outlined", "Second" }
                        Button { variant: "outlined", "Third" }
                        Button { variant: "filled", onclick: move |_| trapped.set(false), "Release" }
                    }
                }
            }
            Button { variant: "text", "After" }
        }
    }
    // demo-code: preview end
}
