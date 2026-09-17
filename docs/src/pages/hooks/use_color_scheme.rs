use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Code, CodeBlock, Flex, SegmentedControl, Text},
    theme::ColorSchemeSetting,
    use_color_scheme,
};

const PICKER: &str = r#"const SETTINGS: [(&str, ColorSchemeSetting); 3] = [
    ("System", ColorSchemeSetting::System),
    ("Light", ColorSchemeSetting::Light),
    ("Dark", ColorSchemeSetting::Dark),
];

#[component]
fn SchemePicker() -> Element {
    let scheme = use_color_scheme();
    let resolved = scheme.resolved().as_str();
    let current = SETTINGS
        .iter()
        .find(|(_, setting)| *setting == scheme.setting())
        .map_or("System", |(label, _)| *label);

    rsx! {
        SegmentedControl {
            "aria-label": "Colour scheme",
            value: current.to_string(),
            options: SETTINGS.map(|(label, _)| label.to_string()).to_vec(),
            onchange: move |next: String| {
                if let Some((_, setting)) = SETTINGS.iter().find(|(label, _)| *label == next) {
                    scheme.set(*setting);
                }
            },
        }
        Text { "On screen: {resolved}" }
    }
}"#;

const SETTINGS: [(&str, ColorSchemeSetting); 3] = [
    ("System", ColorSchemeSetting::System),
    ("Light", ColorSchemeSetting::Light),
    ("Dark", ColorSchemeSetting::Dark),
];

/// `PICKER`, rendered.
#[component]
fn SchemePicker() -> Element {
    let scheme = use_color_scheme();
    let resolved = scheme.resolved().as_str();
    let current = SETTINGS
        .iter()
        .find(|(_, setting)| *setting == scheme.setting())
        .map_or("System", |(label, _)| *label);

    rsx! {
        SegmentedControl {
            "aria-label": "Colour scheme",
            value: current.to_string(),
            options: SETTINGS.map(|(label, _)| label.to_string()).to_vec(),
            onchange: move |next: String| {
                if let Some((_, setting)) = SETTINGS.iter().find(|(label, _)| *label == next) {
                    scheme.set(*setting);
                }
            },
        }
        Text { "On screen: {resolved}" }
    }
}

#[component]
pub fn UseColorSchemePage() -> Element {
    rsx! {
        DocPage {
            title: "use_color_scheme",
            source: "libero/src/hooks/color_scheme.rs",
            markdown: "/md/use_color_scheme.md",
            lead: rsx! {
                Text {
                    Code { source: "use_color_scheme() -> ColorSchemeHandle" }
                    " reads and sets light or dark. "
                    Code { source: "setting()" }
                    " is what the app asked for, "
                    Code { source: "resolved()" }
                    " is the scheme on screen. "
                    Anchor { to: Route::ColorSchemeButtonPage {}, "ColorSchemeButton" }
                    " is the ready-made switch built on it."
                }
            },

            DocSection {
                title: "Usage",
                Flex { direction: "column", align: "flex-start", gap: "sm", SchemePicker {} }
                CodeBlock { source: PICKER, language: "rust" }
                Text {
                    Code { source: "toggle()" }
                    " flips to the other scheme and "
                    Code { source: "cycle()" }
                    " steps through all three settings. "
                    Code { source: "System" }
                    " hands the choice back to the platform, and the component re-renders "
                    "when the platform changes its mind."
                }
            }

            DocSection {
                title: "Web and native",
                Text {
                    "On the web the choice is kept in "
                    Code { source: "localStorage" }
                    ", so a reload comes back to it. "
                    Anchor { to: Route::ThemingPage {}, "Theming" }
                    " shows the script that restores it before the first paint."
                }
            }
        }
    }
}
