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
                    "Each component page has its own Accessibility section with the keys the "
                    "component answers and the props you must set, mostly a name for "
                    "something without a visible label. This page covers the library as a "
                    "whole, including what libero does not do."
                }
            },

            DocSection {
                title: "On and disabled states",
                Text {
                    "A pressed, selected or current control never differs by colour alone. It "
                    "also carries a line. "
                    Code { source: "Button" }
                    ", "
                    Code { source: "ActionIcon" }
                    ", "
                    Code { source: "Chip" }
                    ", "
                    Code { source: "SegmentedControl" }
                    ", the current "
                    Code { source: "Pagination" }
                    " page and the current "
                    Code { source: "Stepper" }
                    " marker draw a thin ring in their own text colour just inside their edge. "
                    "An active "
                    Code { source: "NavLink" }
                    " and a selected row in a "
                    Code { source: "Select" }
                    ", "
                    Code { source: "MultiSelect" }
                    " or "
                    Code { source: "Combobox" }
                    " list get a light tint of their colour and a 2px bar at their start edge, "
                    "in a darker shade of that colour that reaches 4.5:1 on the tint. "
                    "A disabled control fades to half."
                }
            }

            DocSection {
                title: "Forced colors and Windows High Contrast",
                Text {
                    "Libero supports forced colors only in part. The states above still show. "
                    "An on state takes the system's "
                    Code { source: "Highlight" }
                    " colours, and a disabled control's text turns "
                    Code { source: "GrayText" }
                    ". A few components handle "
                    Code { source: "(forced-colors: active)" }
                    " themselves. The rest of the library is untested in that mode."
                }
                Text {
                    "In Windows High Contrast the system replaces every colour an author "
                    "picked and drops "
                    Code { source: "box-shadow" }
                    ". So anything libero draws with a shadow or a colour alone can disappear. "
                    "A "
                    Code { source: "Paper" }
                    " with "
                    Code { source: "bordered: false" }
                    " loses its edge, and a state shown only by colour can no longer be told "
                    "apart. Borders, text, outline focus rings and the layout survive."
                }
                Text {
                    "If your users run High Contrast, prefer a border over a shadow where an "
                    "edge matters, and add the media query yourself:"
                }
                CodeBlock { source: FORCED_COLORS, language: "rust" }
                Text {
                    "Forced colors is a Windows mode, not a WCAG success criterion, and "
                    "covering it properly means work on most of the library plus tests to "
                    "keep it. If you need more of it, open an issue. A real user asking is "
                    "what would change this."
                }
            }
        }
    }
}
