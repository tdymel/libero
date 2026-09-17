use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, indent, prop, props};
use dioxus::prelude::*;
use libero::components::{Button, Flex, FocusTrap, Text};

const TRAPPED: &str = r#"Flex {
    direction: "row",
    gap: "sm",
    Button { variant: "outlined", "First" }
    Button { variant: "outlined", "Second" }
    Button { variant: "outlined", "Third" }
}"#;

/// The switch is not a prop - it is whether the `FocusTrap` is there at all,
/// so off prints the bare children. Either way the two outside buttons are
/// printed: they are what "focus cannot leave" is measured against.
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
                prop("children", "Element").doc("The content Tab/Shift+Tab cycling is confined to."),
            ])],
            lead: rsx! {
                Text {
                    "Confines Tab/Shift+Tab cycling to its children - the same mechanism "
                    "the modal layer uses internally to keep keyboard focus inside an open dialog. It "
                    "focuses its first focusable child on mount, so flipping the switch below "
                    "moves focus into the trap; Tab from there cycles First/Second/Third "
                    "without ever reaching Before or After."
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
                render: move |values: DemoValues| {
                    let trapped = rsx! {
                        Flex {
                            direction: "row",
                            gap: "sm",
                            Button { variant: "outlined", "First" }
                            Button { variant: "outlined", "Second" }
                            Button { variant: "outlined", "Third" }
                        }
                    };

                    rsx! {
                        Flex {
                            direction: "column",
                            gap: "sm",
                            align: "flex-start",
                            Button { variant: "text", "Before" }
                            if values.str("activate_focus_trap") == "true" {
                                FocusTrap { {trapped} }
                            } else {
                                {trapped}
                            }
                            Button { variant: "text", "After" }
                        }
                    }
                },
                wrap: Wrap(wrap_page),
            }
        }
    }
}
