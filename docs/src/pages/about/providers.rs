use crate::components::{DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot, LiberoProvider,
    chrono::NaiveDate,
    components::{Checkbox, Code, CodeBlock, DatePicker, Flex, Paper, Text, Title},
    localization::{Formats, Localization},
};
use pictogram_icons_lucide as lucide;

// snippet: let day = libero::chrono::NaiveDate::from_ymd_opt(2026, 3, 14);
const NESTED: &str = r#"rsx! {
    LiberoProvider {
        DatePicker { value: day, onchange: |_| {} }
        // English words, American formats, like the outer one.
        LiberoProvider {
            localization: &Localization::GERMAN,
            formats: &Formats::GERMAN,
            DatePicker { value: day, onchange: |_| {} }
        }
    }
}"#;

const NESTED_ICONS: &str = r#"rsx! {
    IconProvider { icons: IconSet::new().with(IconSlot::CheckboxCheck, lucide::check_check::outlined),
        Checkbox { label: "Outer", checked: true, onchange: |_| {} }
        IconProvider { icons: IconSet::new().with(IconSlot::CheckboxCheck, lucide::square_check::outlined),
            Checkbox { label: "Inner", checked: true, onchange: |_| {} }
        }
    }
}"#;

#[component]
pub fn ProvidersPage() -> Element {
    // March: "September" is the same word in German.
    let day = NaiveDate::from_ymd_opt(2026, 3, 14);

    rsx! {
        DocPage {
            title: "Providers",
            markdown: "/md/providers.md",
            source: "libero/src/context/libero/mod.rs",
            properties: vec![
                props("LiberoProvider", vec![
                    prop("themes", "ThemeSet")
                        .default("ThemeSet::DEFAULT")
                        .doc("Every theme the app ships; the light and dark halves share one sheet. A lone `&'static Theme` is a set of one."),
                    prop("localization", "&'static Localization")
                        .default("Localization::ENGLISH")
                        .doc("Every string libero shows a reader. Read at mount; switch it later with `use_localization_handle`."),
                    prop("formats", "&'static Formats")
                        .default("Formats::AMERICAN")
                        .doc("How dates, times and numbers are written, whatever the language. Read at mount; switch it later with `use_formats_handle`."),
                    prop("direction", "Option<Direction>")
                        .default("None")
                        .doc("The start text direction, set as the document root's `dir`. A choice made through `use_direction` is kept on the web and wins."),
                ]),
                props("IconProvider", vec![
                    prop("icons", "IconSet")
                        .default("required")
                        .doc("The slots to swap. An empty slot keeps the outer provider's glyph, then libero's lucide default."),
                ]),
            ],
            lead: rsx! {
                Text {
                    Code { source: "LiberoProvider" }
                    " is the root every app renders once: themes, localization, formats, stylesheets and portals. "
                    Code { source: "IconProvider" }
                    " swaps libero's glyphs. Both are plain context providers, so "
                    "different parts of one page can sit below different ones, and the nearest wins."
                }
            },
            DocSection {
                title: "Different providers in one page",
                Text {
                    "A "
                    Code { source: "LiberoProvider" }
                    " inside another gives its own subtree its own "
                    Code { source: "localization" }
                    " and "
                    Code { source: "formats" }
                    ". Everything outside keeps the outer ones. Both calendars below sit in one page: "
                    "the outer one follows this site, English words and German formats, so only the words differ."
                }
                Flex { direction: "row", gap: "lg", wrap: "wrap", align: "start",
                    Paper { padding: "md",
                        Flex { direction: "column", gap: "sm",
                            Title { size: "sm", component: "h3", "Outer provider" }
                            DatePicker { value: day, onchange: |_| {} }
                        }
                    }
                    LiberoProvider { localization: &Localization::GERMAN, formats: &Formats::GERMAN,
                        Paper { padding: "md",
                            Flex { direction: "column", gap: "sm",
                                Title { size: "sm", component: "h3", "Inner provider" }
                                DatePicker { value: day, onchange: |_| {} }
                            }
                        }
                    }
                }
                CodeBlock { source: NESTED, language: "rust" }
                Text {
                    "Three limits. The theme is a sheet on the document root, so a nested provider's "
                    Code { source: "themes" }
                    " do not scope to its subtree. "
                    Code { source: "direction" }
                    " also sets the document root; for one subtree, put a "
                    Code { source: "dir=\"rtl\"" }
                    " attribute on an element around it. And hooks such as "
                    Code { source: "use_localization_handle" }
                    " switch the nearest provider only."
                }
            }
            DocSection {
                title: "Icon providers merge",
                Text {
                    "Nested "
                    Code { source: "IconProvider" }
                    "s merge: the inner one wins per slot, the rest comes from the outer one, then lucide."
                }
                Flex { direction: "column", gap: "sm", align: "start",
                    IconProvider { icons: IconSet::new().with(IconSlot::CheckboxCheck, lucide::check_check::outlined),
                        Checkbox { label: "Outer", checked: true, onchange: |_| {} }
                        IconProvider { icons: IconSet::new().with(IconSlot::CheckboxCheck, lucide::square_check::outlined),
                            Checkbox { label: "Inner", checked: true, onchange: |_| {} }
                        }
                    }
                }
                CodeBlock { source: NESTED_ICONS, language: "rust" }
            }
        }
    }
}
