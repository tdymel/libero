use super::KeepSite;
use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, CodeBlock, Flex, Text},
    localization::Formats,
    use_formats_handle,
};

const SWITCH: &str = r#"#[component]
fn RegionSwitch() -> Element {
    let formats = use_formats_handle();
    let german = *formats.get() == Formats::GERMAN;
    let first = formats.get().first_weekday;

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| formats.set(if german { &Formats::AMERICAN } else { &Formats::GERMAN }),
            if german { "Use American formats" } else { "Use German formats" }
        }
        Text { "Weeks start on {first}, and 5.4 MB reads 5{formats.get().decimal_separator}4 MB." }
    }
}"#;

/// `SWITCH`, rendered.
#[component]
fn RegionSwitch() -> Element {
    let formats = use_formats_handle();
    let german = *formats.get() == Formats::GERMAN;
    let first = formats.get().first_weekday;

    rsx! {
        Button {
            variant: "outlined",
            onclick: move |_| formats.set(if german { &Formats::AMERICAN } else { &Formats::GERMAN }),
            if german { "Use American formats" } else { "Use German formats" }
        }
        Text { "Weeks start on {first}, and 5.4 MB reads 5{formats.get().decimal_separator}4 MB." }
    }
}

#[component]
pub fn UseFormatsHandlePage() -> Element {
    rsx! {
        DocPage {
            title: "use_formats_handle",
            source: "libero/src/hooks/formats.rs",
            markdown: "/md/use_formats_handle.md",
            lead: rsx! {
                Text {
                    Code { source: "use_formats_handle() -> FormatsHandle" }
                    " switches the date, time and number formats at runtime, independent of "
                    "the language. A region picker is built on it. "
                    Anchor { to: Route::LocalizationPage {}, "Localization" }
                    " shows how to write formats of your own."
                }
            },

            DocSection {
                title: "Usage",
                Text {
                    "The button switches this site. It gets its own formats back when you "
                    "leave the page."
                }
                KeepSite {
                    Flex { direction: "column", align: "flex-start", gap: "sm", RegionSwitch {} }
                }
                CodeBlock { source: SWITCH, language: "rust" }
                Text {
                    "Every calendar, date field and time picker re-renders on a switch. The "
                    "provider reads its "
                    Code { source: "formats" }
                    " prop once, at mount, so switch through the handle."
                }
            }
        }
    }
}
