use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Text},
    use_localization,
};

const CLOSE_LABEL: &str = r#"#[component]
fn CloseLabel() -> Element {
    let words = use_localization();

    rsx! {
        Text { "Every close button is named \"{words.common.close}\"." }
    }
}"#;

/// `CLOSE_LABEL`, rendered.
#[component]
fn CloseLabel() -> Element {
    let words = use_localization();

    rsx! {
        Text { "Every close button is named \"{words.common.close}\"." }
    }
}

#[component]
pub fn UseLocalizationPage() -> Element {
    rsx! {
        DocPage {
            title: "use_localization",
            source: "libero/src/hooks/localization.rs",
            markdown: "/md/use_localization.md",
            lead: rsx! {
                Text {
                    Code { source: "use_localization() -> &'static Localization" }
                    " returns the words libero's components say on their own, in the active "
                    "language. Read it where your component says the same thing, so it "
                    "switches language with them. "
                    Anchor { to: Route::LocalizationPage {}, "Localization" }
                    " lists what it holds."
                }
            },

            DocSection {
                title: "Usage",
                CloseLabel {}
                CodeBlock { source: CLOSE_LABEL, language: "rust" }
                Text {
                    "The component re-renders when the language is switched with "
                    Anchor { to: Route::UseLocalizationHandlePage {}, "use_localization_handle" }
                    "."
                }
            }
        }
    }
}
