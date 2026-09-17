use super::KeepSite;
use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, CodeBlock, Flex, Text},
    localization::Localization,
    use_localization_handle,
};

const SWITCH: &str = r#"#[component]
fn LanguageSwitch() -> Element {
    let localization = use_localization_handle();
    let german = *localization.get() == Localization::GERMAN;

    rsx! {
        Button {
            variant: "outlined",
            lang: if german { "en" } else { "de" },
            onclick: move |_| {
                localization.set(if german { &Localization::ENGLISH } else { &Localization::GERMAN });
            },
            if german { "English" } else { "Deutsch" }
        }
        Text { "Close buttons say \"{localization.get().common.close}\"." }
    }
}"#;

/// `SWITCH`, rendered.
#[component]
fn LanguageSwitch() -> Element {
    let localization = use_localization_handle();
    let german = *localization.get() == Localization::GERMAN;

    rsx! {
        Button {
            variant: "outlined",
            lang: if german { "en" } else { "de" },
            onclick: move |_| {
                localization.set(if german { &Localization::ENGLISH } else { &Localization::GERMAN });
            },
            if german { "English" } else { "Deutsch" }
        }
        Text { "Close buttons say \"{localization.get().common.close}\"." }
    }
}

#[component]
pub fn UseLocalizationHandlePage() -> Element {
    rsx! {
        DocPage {
            title: "use_localization_handle",
            source: "libero/src/hooks/localization.rs",
            markdown: "/md/use_localization_handle.md",
            lead: rsx! {
                Text {
                    Code { source: "use_localization_handle() -> LocalizationHandle" }
                    " switches the language at runtime. A language picker is built on it. "
                    Anchor { to: Route::LocalizationPage {}, "Localization" }
                    " covers the languages that ship and how to change the words."
                }
            },

            DocSection {
                title: "Usage",
                Text {
                    "The button switches this site. It gets its own language back when you "
                    "leave the page."
                }
                KeepSite {
                    Flex { direction: "column", align: "flex-start", gap: "sm", LanguageSwitch {} }
                }
                CodeBlock { source: SWITCH, language: "rust" }
                Text {
                    "Every component that reads the localization re-renders on a switch. The "
                    "provider reads its "
                    Code { source: "localization" }
                    " prop once, at mount, so switch through the handle."
                }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    "The button is labelled in the language it switches to, and its "
                    Code { source: "lang" }
                    " says so, so a screen reader pronounces \"Deutsch\" in German."
                }
            }
        }
    }
}
