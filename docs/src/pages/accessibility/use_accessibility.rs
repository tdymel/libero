use crate::components::{Demo, DemoFile, DemoValues, DocPage, a11y};
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
            accessibility: a11y()
                .handles([
                    "`set_reduced_motion(Some(on))` forces reduced motion on or off over the system's, and reaches libero's own CSS and motion on every platform. `None` follows the system again.",
                    "The system's settings are the page's media queries in a browser and Android's WebView; natively they come from the Linux desktop portal (`org.freedesktop.appearance`, GNOME's and older KDE's own keys as a fallback), and other native platforms report no preference.",
                    "A forced choice is kept under `lsx-reduced-motion`: in a browser's `localStorage`, and in a file on Android, the desktop WebView and native where `libero::platform::set_storage_dir` (or libero's `desktop` or `native` feature) gives local storage a directory. Elsewhere it lasts for the session.",
                ])
                .must([
                    "Offer the choice with a way back: a \"Follow the system\" control that calls `set_reduced_motion(None)`, as the demo does.",
                    "Apply `reduced_motion()` yourself to a `<style>` your app adds: it still follows the system's setting.",
                ])
                .limits([
                    "Only reduced motion can be forced. `forced_colors()`, `contrast()` and `reduced_transparency()` are read-only.",
                ]),
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
        }
    }
}
