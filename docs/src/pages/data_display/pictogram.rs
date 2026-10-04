use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, ExtraTab, IconCatalogue, PictogramNote, a11y,
    prop, props,
};
use dioxus::prelude::*;
use libero::components::{Code, Pictogram, SvgData, Text};
use pictogram_icons_lucide as lucide;

const ICONS: [&str; 5] = ["house", "heart", "star", "bell", "search"];

/// The `aria_label` switch's name for a glyph, such as "House".
fn icon_name(name: &str) -> String {
    let mut chars = name.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

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
                    prop("attributes", "Vec<Attribute>")
                        .doc("Any svg attribute, winning over the glyph's own: `width`, `height`, `stroke_width`, `class`. Others by name in quotes, as `\"aria-labelledby\": \"logo-title\"`."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .handles([
                    "A pictogram is hidden from screen readers (`aria-hidden=\"true\"`).",
                    "`aria_label`, or an `aria-labelledby` attribute, makes it `role=\"img\"` instead.",
                ])
                .must([
                    "Name a pictogram that means something on its own with `aria_label`. One next to a text label stays hidden.",
                    "For a clickable glyph, use `ActionIcon { icon }`.",
                ])
                .example("A status column with no text, `Pictogram { icon: .., aria_label: \"Synced\" }`: a screen reader reads an image named \"Synced\". The same glyph next to the word \"Synced\" stays hidden."),
            extra_tab: ExtraTab {
                label: "Icons",
                id: "icons",
                content: rsx! { IconCatalogue {} },
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
                    // Off, the glyph stays hidden, as one beside a text label should.
                    Control::switch("aria_label").code(|_, values| match values.str("aria_label").as_str() {
                        "true" => vec![format!("aria_label: {:?}", icon_name(&values.str("icon")))],
                        _ => vec![],
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Pictogram {
                        icon: icon(&values.str("icon")),
                        width: values.str("size"),
                        height: values.str("size"),
                        stroke_width: values.str("stroke"),
                        aria_label: (values.str("aria_label") == "true").then(|| icon_name(&values.str("icon"))),
                    }
                },
            }

            DocSection {
                title: "Colours",
                Text {
                    "Not every glyph is "
                    Code { source: "currentColor" }
                    ": the color variants of "
                    Code { source: "pictogram-icons-lobe" }
                    " and some phosphor glyphs hard-code their fills, so they ignore the text color and dark mode. Lobe's gradient glyphs carry fixed ids: two copies on one page share them."
                }
            }

            DocSection {
                title: "Versions",
                Text {
                    "Take icon crates from pictogram's 0.5 line, the one libero builds on. A crate of another minor, such as 0.4, brings a second "
                    Code { source: "SvgData" }
                    " type: its icons fail with a type mismatch that does not name the version."
                }
            }
        }
    }
}
