use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Text};

/// The arm a caller writes for themselves where a component would otherwise
/// vanish under forced colors. Printed, not run - the page has no preview.
const FORCED_COLORS: &str = r#"Paper {
    // A border with no colour is currentColor, which the OS repaints.
    sx: sx().media("(forced-colors: active)", sx().border("1px solid")),
    "Still has an edge in High Contrast"
}"#;

#[component]
pub fn AccessibilityPage() -> Element {
    rsx! {
        DocPage {
            title: "Accessibility",
            markdown: "/md/accessibility.md",
            lead: rsx! {
                Text {
                    "Every component page has its own Accessibility section: the keys that "
                    "component answers, and the props you have to set because it cannot work "
                    "them out - a name for something that has no visible label, mostly. This "
                    "page holds what applies to the library as a whole, starting with what "
                    "libero does not do, so you can decide what to do about it."
                }
            },

            DocSection {
                title: "Forced colors and Windows High Contrast",
                Text {
                    "Libero does not support forced colors. No component carries a "
                    Code { source: "(forced-colors: active)" }
                    " arm, and nothing is tested in that mode."
                }
                Text {
                    "Windows High Contrast is the mode this affects. In it the operating "
                    "system replaces every colour an author picked and drops "
                    Code { source: "box-shadow" }
                    " entirely. Anything libero draws with a shadow or a themed colour alone "
                    "can therefore disappear: a "
                    Code { source: "Paper" }
                    " that relies on its shadow for its edge ("
                    Code { source: "bordered: false" }
                    ") has no visible boundary at all, and a state carried only by colour is "
                    "no longer distinguishable. Borders, text, focus rings drawn as outlines "
                    "and the layout itself survive, because the system repaints them."
                }
                Text {
                    "If your users run High Contrast, prefer a border over a shadow where a "
                    "boundary matters, and write the arm yourself - "
                    Code { source: "sx" }
                    " takes any media query:"
                }
                CodeBlock { source: FORCED_COLORS, language: "rust" }
                Text {
                    "The reason we stop there: forced colors is a Windows platform mode, not "
                    "a WCAG success criterion at any conformance level, and covering it "
                    "properly means an arm on most of the library plus a test tier to keep it "
                    "honest. We would rather say so than half-support it. If you need it, "
                    "open an issue - a real user asking is what would change this."
                }
            }
        }
    }
}
