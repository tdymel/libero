use crate::components::{Demo, DemoFile, DemoValues, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, Text};

mod demo;
use demo::Settings;

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
                    "reduced motion on or off over the system's, and the choice is kept "
                    "for the next visit (libero's local storage) until Follow the system "
                    "clears it. This page clears what its demo forced when you "
                    "leave it."
                }
            },
            Demo {
                component: "Settings",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! { Settings {} },
                file: DemoFile(include_str!("use_accessibility/demo.rs")),
            }

            DocSection {
                title: "Forced settings",
                Text {
                    "A forced reduced motion reaches libero's own CSS and motion on every "
                    "platform. A "
                    Code { source: "<style>" }
                    " the app adds itself still follows the system. The other settings are "
                    "read-only."
                }
            }
        }
    }
}
