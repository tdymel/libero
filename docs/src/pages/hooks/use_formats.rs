use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Text},
    use_formats,
};

const FILE_SIZE: &str = r#"#[component]
fn FileSize() -> Element {
    let formats = use_formats();

    rsx! {
        Text { "report.pdf, 5{formats.decimal_separator}4 MB" }
    }
}"#;

/// `FILE_SIZE`, rendered.
#[component]
fn FileSize() -> Element {
    let formats = use_formats();

    rsx! {
        Text { "report.pdf, 5{formats.decimal_separator}4 MB" }
    }
}

#[component]
pub fn UseFormatsPage() -> Element {
    rsx! {
        DocPage {
            title: "use_formats",
            source: "libero/src/hooks/formats.rs",
            markdown: "/md/use_formats.md",
            lead: rsx! {
                Text {
                    Code { source: "use_formats() -> &'static Formats" }
                    " returns how the active region writes dates, times and numbers: the "
                    "first weekday, the date and time patterns, and the range and decimal "
                    "separators. "
                    Anchor { to: Route::LocalizationPage {}, "Localization" }
                    " describes each field."
                }
            },

            DocSection {
                title: "Usage",
                FileSize {}
                CodeBlock { source: FILE_SIZE, language: "rust" }
                Text {
                    "This site runs in "
                    Code { source: "Formats::GERMAN" }
                    ", so the size reads with a comma. The component re-renders when the "
                    "formats are switched with "
                    Anchor { to: Route::UseFormatsHandlePage {}, "use_formats_handle" }
                    "."
                }
            }
        }
    }
}
