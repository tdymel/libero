use crate::components::{Demo, DemoValues, DocPage, Wrap};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Switch, Text},
    hooks::use_accessibility,
};

fn code(_: &DemoValues, _: &str) -> String {
    r#"let accessibility = use_accessibility();

rsx! {
    Flex { direction: "column", gap: "sm",
        Switch {
            label: "Reduce motion",
            checked: accessibility.reduced_motion(),
            onchange: {
                let accessibility = accessibility.clone();
                move |on: bool| accessibility.set_reduced_motion(Some(on))
            },
        }
        Button {
            variant: "outlined",
            onclick: {
                let accessibility = accessibility.clone();
                move |_| accessibility.set_reduced_motion(None)
            },
            "Follow the system"
        }
        Text {
            "Contrast: {accessibility.contrast().as_str()}, forced colors: "
            "{accessibility.forced_colors()}, reduced transparency: "
            "{accessibility.reduced_transparency()}"
        }
    }
}"#
    .to_string()
}

#[component]
fn Settings() -> Element {
    let accessibility = use_accessibility();

    rsx! {
        Flex { direction: "column", gap: "sm",
            Switch {
                label: "Reduce motion",
                checked: accessibility.reduced_motion(),
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| accessibility.set_reduced_motion(Some(on))
                },
            }
            Button {
                variant: "outlined",
                onclick: {
                    let accessibility = accessibility.clone();
                    move |_| accessibility.set_reduced_motion(None)
                },
                "Follow the system"
            }
            Text {
                "Contrast: {accessibility.contrast().as_str()}, forced colors: "
                "{accessibility.forced_colors()}, reduced transparency: "
                "{accessibility.reduced_transparency()}"
            }
        }
    }
}

#[component]
pub fn UseAccessibilityPage() -> Element {
    rsx! {
        DocPage {
            title: "Accessibility settings",
            source: "libero/src/hooks/accessibility.rs",
            markdown: "/md/use_accessibility.md",
            lead: rsx! {
                Text {
                    Code { source: "use_accessibility() -> AccessibilityHandle" }
                    " reads the reader's accessibility settings: reduced motion, forced "
                    "colors, contrast and reduced transparency. A settings page can force "
                    "reduced motion on or off over the system's."
                }
                Text {
                    "A forced reduced motion reaches libero's own CSS and motion on every "
                    "platform. A "
                    Code { source: "<style>" }
                    " the app adds itself still follows the system. The other settings are "
                    "read-only."
                }
            },
            Demo {
                component: "use_accessibility",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Settings {} },
                wrap: Wrap(code),
            }
        }
    }
}
