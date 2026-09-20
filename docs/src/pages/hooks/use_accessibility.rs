use crate::components::{Demo, DemoValues, DocPage, Wrap};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, Flex, Switch, Text},
    hooks::use_accessibility,
    theme::{AccessibilityOverrides, Contrast},
};

fn code(_: &DemoValues, _: &str) -> String {
    r#"let accessibility = use_accessibility();
let now = accessibility.get();

rsx! {
    Flex { direction: "column", gap: "sm",
        Switch {
            label: "Reduce motion",
            checked: now.reduced_motion,
            onchange: {
                let accessibility = accessibility.clone();
                move |on: bool| accessibility.set_overrides(AccessibilityOverrides {
                    reduced_motion: Some(on),
                    ..accessibility.overrides()
                })
            },
        }
        Switch {
            label: "More contrast",
            checked: now.contrast == Contrast::More,
            onchange: {
                let accessibility = accessibility.clone();
                move |on: bool| accessibility.set_overrides(AccessibilityOverrides {
                    contrast: Some(if on { Contrast::More } else { Contrast::NoPreference }),
                    ..accessibility.overrides()
                })
            },
        }
        Button {
            variant: "outlined",
            onclick: move |_| accessibility.set_overrides(AccessibilityOverrides::default()),
            "Follow the system"
        }
    }
}"#
    .to_string()
}

#[component]
fn Settings() -> Element {
    let accessibility = use_accessibility();
    let now = accessibility.get();

    rsx! {
        Flex { direction: "column", gap: "sm",
            Switch {
                label: "Reduce motion",
                checked: now.reduced_motion,
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| accessibility.set_overrides(AccessibilityOverrides {
                        reduced_motion: Some(on),
                        ..accessibility.overrides()
                    })
                },
            }
            Switch {
                label: "More contrast",
                checked: now.contrast == Contrast::More,
                onchange: {
                    let accessibility = accessibility.clone();
                    move |on: bool| accessibility.set_overrides(AccessibilityOverrides {
                        contrast: Some(if on { Contrast::More } else { Contrast::NoPreference }),
                        ..accessibility.overrides()
                    })
                },
            }
            Button {
                variant: "outlined",
                onclick: move |_| accessibility.set_overrides(AccessibilityOverrides::default()),
                "Follow the system"
            }
        }
    }
}

#[component]
pub fn UseAccessibilityPage() -> Element {
    rsx! {
        DocPage {
            title: "use_accessibility",
            source: "libero/src/hooks/accessibility.rs",
            markdown: "/md/use_accessibility.md",
            lead: rsx! {
                Text {
                    Code { source: "use_accessibility() -> AccessibilityHandle" }
                    " reads the reader's accessibility preferences: reduced motion, forced "
                    "colors, contrast and reduced transparency. An app's settings page can "
                    "answer them over the system with overrides."
                }
                Text {
                    "The overrides act on native renderers only. In a browser the media "
                    "queries decide, and the switches below show what the browser answers "
                    "but change nothing. "
                    Code { source: "LiberoProvider" }
                    " takes the same overrides at mount as "
                    Code { source: "accessibility" }
                    "."
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
