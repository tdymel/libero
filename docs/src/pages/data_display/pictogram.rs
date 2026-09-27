use super::icon_catalogue::IconCatalogue;
use crate::components::{
    Control, Demo, DemoValues, DocPage, ExtraTab, PictogramNote, a11y, prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Pictogram, SvgData, Text};
use pictogram_icons_lucide as lucide;

const ICONS: [&str; 5] = ["house", "heart", "star", "bell", "search"];

fn icon(name: &str) -> SvgData {
    match name {
        "heart" => lucide::heart::outlined,
        "star" => lucide::star::outlined,
        "bell" => lucide::bell::outlined,
        "search" => lucide::search::outlined,
        _ => lucide::house::outlined,
    }
}

#[component]
pub fn PictogramPage() -> Element {
    rsx! {
        DocPage {
            title: "Pictogram",
            source: "libero/src/components/data_display/pictogram.rs",
            markdown: "/md/pictogram.md",
            properties: vec![
                props("Pictogram", vec![
                    prop("icon", "SvgData")
                        .default("required")
                        .doc("The glyph: a const from a pictogram icon crate, or `SvgData::new(include_str!(\"x.svg\"))`."),
                    prop("aria_label", "Option<String>")
                        .default("None")
                        .doc("Names the glyph: `role=\"img\"` instead of `aria-hidden`. Leave unset next to a text label."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "A pictogram is hidden from screen readers (`aria-hidden=\"true\"`).",
                    "`aria_label`, or `\"aria-labelledby\"` by its name in quotes, makes it `role=\"img\"` instead.",                ])
                .must([
                    "Name a pictogram that means something on its own with `aria_label`. One next to a text label stays hidden.",
                    "For a clickable glyph, use `ActionIcon { icon }`.",
                ]),
            extra_tab: ExtraTab {
                label: "Icons",
                id: "icons",
                content: rsx! {
                    Text {
                        "Every icon of a set, from its crate's "
                        Code { source: "index" }
                        " feature. Search by words of the name, then copy the path of the icon's const. Only lucide is listed for now. Lobe's colour variants are left out: they hard-code their fills."
                    }
                    IconCatalogue {}
                },
            },
            lead: rsx! {
                PictogramNote {}
                Text {
                    "Draws an "
                    Code { source: "SvgData" }
                    " glyph as an inline svg. It has no size of its own: "
                    Code { source: "Icon" }
                    " and "
                    Code { source: "ActionIcon" }
                    " size it, or set "
                    Code { source: "width" }
                    " and "
                    Code { source: "height" }
                    ". It draws in "
                    Code { source: "currentColor" }
                    ", so it takes the text color. Your attributes win over the glyph's own."
                }
                Text {
                    "Not every glyph is "
                    Code { source: "currentColor" }
                    ": the color variants of "
                    Code { source: "pictogram-icons-lobe" }
                    " and some phosphor glyphs hard-code their fills, so they ignore the text color and dark mode. Lobe's gradient glyphs carry fixed ids: two copies on one page share them."
                }
                Text {
                    "Take icon crates from pictogram's 0.5 line, the one libero builds on. A crate of another minor, such as 0.4, brings a second "
                    Code { source: "SvgData" }
                    " type: its icons fail with a type mismatch that does not name the version."
                }
            },
            Demo {
                component: "Pictogram",
                children_text: "",
                controls: vec![
                    Control::select("icon", ICONS).code(|_, values| {
                        vec![format!("icon: pictogram_icons_lucide::{}::outlined", values.str("icon"))]
                    }),
                    Control::slider("size", ["16px", "24px", "32px", "48px"])
                        .default("32px")
                        .code(|_, values| {
                            let size = values.str("size");
                            vec![format!("width: {size:?}"), format!("height: {size:?}")]
                        }),
                    Control::slider("stroke", ["1", "1.5", "2", "2.5", "3"])
                        .default("2")
                        .code(|control, values| {
                            let stroke = values.str(control.name);
                            if stroke == control.default {
                                vec![]
                            } else {
                                vec![format!("stroke_width: {stroke:?}")]
                            }
                        }),
                ],
                render: move |values: DemoValues| rsx! {
                    Pictogram {
                        icon: icon(&values.str("icon")),
                        width: values.str("size"),
                        height: values.str("size"),
                        stroke_width: values.str("stroke"),
                    }
                },
            }
        }
    }
}
