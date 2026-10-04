use super::KeepSite;
use crate::Route;
use crate::components::{Demo, DemoValues, DocPage, DocSection, Wrap, a11y};
use dioxus::prelude::*;
use libero::{
    components::{Anchor, Button, Code, Flex, Text},
    theme::ThemeSet,
    use_theme_set,
};

/// The hook call and the picker, as `ThemePicker` renders them.
// snippet: mirrors ThemePicker
fn code(_: &DemoValues, _: &str) -> String {
    r#"let themes = use_theme_set();

rsx! {
    Flex { direction: "row", gap: "sm",
        for set in ThemeSet::CATALOGUE.iter().take(4) {
            Button {
                variant: if themes.name() == set.name() { "filled" } else { "outlined" },
                aria_pressed: themes.name() == set.name(),
                onclick: {
                    let themes = themes.clone();
                    move |_| themes.set((*set).clone())
                },
                "{set.name()}"
            }
        }
    }
}"#
    .to_string()
}

#[component]
fn ThemePicker() -> Element {
    let themes = use_theme_set();

    rsx! {
        Flex { direction: "row", gap: "sm",
            for set in ThemeSet::CATALOGUE.iter().take(4) {
                Button {
                    variant: if themes.name() == set.name() { "filled" } else { "outlined" },
                    aria_pressed: themes.name() == set.name(),
                    onclick: {
                        let themes = themes.clone();
                        move |_| themes.set((*set).clone())
                    },
                    "{set.name()}"
                }
            }
        }
    }
}

#[component]
pub fn UseThemeSetPage() -> Element {
    rsx! {
        DocPage {
            title: "Theme set",
            source: "libero/src/hooks/theme.rs",
            markdown: "/md/use_theme_set.md",
            accessibility: a11y()
                .handles([
                    "A swap keeps the reader's colour scheme setting, so a reader who pinned dark stays in dark.",
                ])
                .must([
                    "Mark the active set on its control, as the demo's buttons do with `aria_pressed`.",
                    "Check the sets you offer against your own colours: in Kanagawa, Kanagawa Dragon and Vague light `muted.6` falls just under 3:1, the minimum for borders and icons (WCAG 1.4.11).",
                ]),
            lead: rsx! {
                Text {
                    Code { source: "use_theme_set() -> ThemeSetHandle" }
                    " reads and swaps the active theme set, the light and dark pair the app "
                    "is drawn in. A theme picker is built on it. "
                    Anchor { to: Route::ThemingPage {}, "Theming" }
                    " covers theme sets and the catalogue."
                }
            },
            Demo {
                component: "use_theme_set",
                children_text: "",
                controls: Vec::new(),
                render: move |_: DemoValues| rsx! {
                    KeepSite { ThemePicker {} }
                },
                wrap: Wrap(code),
            }

            DocSection {
                title: "Swapping",
                Text {
                    "A swap rebuilds the stylesheet, because the sheet carries the set's "
                    "pair. The color scheme setting survives it, so a reader who pinned "
                    "dark stays in dark. The buttons below switch this site, which gets "
                    "its own set back when you leave the page."
                }
            }
        }
    }
}
