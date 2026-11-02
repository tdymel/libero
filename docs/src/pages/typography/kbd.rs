use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Kbd, Text};
use libero::use_theme;

#[component]
pub fn KbdPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Kbd",
            source: "libero/src/components/typography/kbd.rs",
            markdown: "/md/kbd.md",
            properties: vec![props("Kbd", vec![
                prop("size", "Size")
                    .default(theme.kbd.size.as_str())
                    .doc("Font size. The rest of the look comes from the theme."),
                prop("children", "Element").default("required").doc("The key label."),
            ])],
            accessibility: a11y()
                .handles(["Each key is a real `<kbd>`."])
                .must([
                    "Put the separator in the text around the keys. A screen reader reads `Kbd { \"Ctrl\" } \" + \" Kbd { \"S\" }` as \"Ctrl plus S\", but one `Kbd { \"Ctrl+S\" }` as a single token. Put `dir: \"ltr\"` on the element around a chord, or a right-to-left page shows it as `S + Ctrl`.",
                ])
                .example("A save hint, `Kbd { \"Ctrl\" } \" + \" Kbd { \"S\" }`: a screen reader reads \"Ctrl plus S\", key by key."),
            lead: rsx! {
                Text {
                    "One keyboard key in a real "
                    Code { source: "<kbd>" }
                    ", styled from the theme. A shortcut is several keys with your own "
                    "separator: save with "
                    // One unit: a 375px line broke between the keys. A chord reads left to right.
                    span { style: "white-space: nowrap", dir: "ltr",
                        Kbd { "Ctrl" }
                        " + "
                        Kbd { "S" }
                    }
                    "."
                }
            },
            Demo {
                component: "Kbd",
                children_text: "Ctrl",
                controls: vec![
                    Control::sizes("size")
                        .default(theme.kbd.size.as_str()),
                ],
                render: move |values: DemoValues| rsx! {
                    Kbd { size: values.str("size"), "Ctrl" }
                },
            }
        }
    }
}
