use crate::components::{Control, Demo, DemoValues, DocPage, PictogramNote, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot,
    components::{Checkbox, Code, Flex, Options, PasswordField, Select, SvgData, Text},
};
use pictogram_icons_lucide as lucide;

const CHEVRONS: [&str; 3] = ["chevron_down", "chevrons_down", "circle_chevron_down"];
const CHECKS: [&str; 3] = ["check", "check_check", "square_check"];
const EYES: [&str; 3] = ["eye", "scan_eye", "view"];

fn icon(name: &str) -> SvgData {
    match name {
        "chevrons_down" => lucide::chevrons_down::outlined,
        "circle_chevron_down" => lucide::circle_chevron_down::outlined,
        "check" => lucide::check::outlined,
        "check_check" => lucide::check_check::outlined,
        "square_check" => lucide::square_check::outlined,
        "eye" => lucide::eye::outlined,
        "scan_eye" => lucide::scan_eye::outlined,
        "view" => lucide::view::outlined,
        _ => lucide::chevron_down::outlined,
    }
}

#[derive(Clone, PartialEq, Options)]
enum Fruit {
    Apple,
    Pear,
}

#[component]
pub fn IconProviderPage() -> Element {
    rsx! {
        DocPage {
            title: "IconProvider",
            source: "libero/src/context/icons.rs",
            markdown: "/md/icon_provider.md",
            properties: vec![
                props("IconProvider", vec![
                    prop("icons", "IconSet")
                        .default("required")
                        .doc("The slots to swap. An empty slot keeps the outer provider's glyph, then libero's lucide default."),
                ]),
            ],
            accessibility: a11y()
                .handles([
                    "A swapped glyph stays decorative (`aria-hidden=\"true\"`): the control it sits in keeps its own name.",
                ])
                .must([
                    "Pick a glyph that means the same as the one it replaces: a chevron for `ChevronDown`, a check for `CheckboxCheck`.",
                ]),
            lead: rsx! {
                PictogramNote {}
                Text {
                    "Swaps the glyphs libero draws itself, slot by slot, for everything below it. Each "
                    Code { source: "IconSlot" }
                    " names one glyph by what it means; an "
                    Code { source: "IconSet" }
                    " maps slots to "
                    Code { source: "SvgData" }
                    ". Slots you leave empty keep lucide, libero's default. Nested providers merge: the inner one wins per slot."
                }
                Text {
                    "One slot: "
                    Code { source: "IconSet::new().with(IconSlot::Close, icon)" }
                    ". The whole set: fill every slot once from another pictogram crate, such as "
                    Code { source: "pictogram-icons-tabler" }
                    ", in a provider at your app's root. Brand marks are no slots: they name a service."
                }
            },
            // snippet: item #[derive(Clone, PartialEq, Options)] enum Fruit { Apple, Pear }
            Demo {
                component: "IconProvider",
                children_text: "",
                children_code: "Flex {{ direction: \"column\", gap: \"md\",\n    Select {{ label: \"Fruit\", value: Fruit::Apple, onchange: |_| {{}} }}\n    Checkbox {{ label: \"Ripe\", checked: true, onchange: |_| {{}} }}\n    PasswordField {{ label: \"Password\" }}\n}}",
                controls: vec![
                    Control::select("chevron", CHEVRONS).code(|_, values| {
                        vec![format!(
                            "icons: IconSet::new()\n    .with(IconSlot::ChevronDown, pictogram_icons_lucide::{}::outlined)\n    .with(IconSlot::CheckboxCheck, pictogram_icons_lucide::{}::outlined)\n    .with(IconSlot::Eye, pictogram_icons_lucide::{}::outlined)",
                            values.str("chevron"),
                            values.str("check"),
                            values.str("eye"),
                        )]
                    }),
                    Control::select("check", CHECKS).code(|_, _| vec![]),
                    Control::select("eye", EYES).code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    IconProvider {
                        icons: IconSet::new()
                            .with(IconSlot::ChevronDown, icon(&values.str("chevron")))
                            .with(IconSlot::CheckboxCheck, icon(&values.str("check")))
                            .with(IconSlot::Eye, icon(&values.str("eye"))),
                        Flex { direction: "column", gap: "md",
                            Select { label: "Fruit", value: Fruit::Apple, onchange: |_| {} }
                            Checkbox { label: "Ripe", checked: true, onchange: |_| {} }
                            PasswordField { label: "Password" }
                        }
                    }
                },
            }
        }
    }
}
