use crate::Route;
use crate::components::{Control, Demo, DemoValues, DocPage, PictogramNote, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot,
    components::{
        Anchor, Checkbox, Code, Flex, NumberField, Options, Pagination, PasswordField, Select,
        SvgData, Text,
    },
};
use pictogram_icons_lucide as lucide;

// "set" keeps the set's own glyph for the slot.
const CHEVRONS: [&str; 3] = ["set", "chevrons_down", "circle_chevron_down"];
const CHECKS: [&str; 3] = ["set", "check_check", "square_check"];
const EYES: [&str; 3] = ["set", "scan_eye", "view"];
const SETS: [&str; 7] = [
    "lucide",
    "material_rounded",
    "material_filled",
    "tabler_outlined",
    "bootstrap_outlined",
    "phosphor_regular",
    "phosphor_bold",
];
const SLOTS: [(&str, &str); 3] = [
    ("chevron", "ChevronDown"),
    ("check", "CheckboxCheck"),
    ("eye", "Eye"),
];

fn icon_set(name: &str) -> IconSet {
    match name {
        "material_rounded" => IconSet::material_rounded(),
        "material_filled" => IconSet::material_filled(),
        "tabler_outlined" => IconSet::tabler_outlined(),
        "bootstrap_outlined" => IconSet::bootstrap_outlined(),
        "phosphor_regular" => IconSet::phosphor_regular(),
        "phosphor_bold" => IconSet::phosphor_bold(),
        _ => IconSet::new(),
    }
}

fn icon(name: &str) -> Option<SvgData> {
    Some(match name {
        "chevrons_down" => lucide::chevrons_down::outlined,
        "circle_chevron_down" => lucide::circle_chevron_down::outlined,
        "check_check" => lucide::check_check::outlined,
        "square_check" => lucide::square_check::outlined,
        "scan_eye" => lucide::scan_eye::outlined,
        "view" => lucide::view::outlined,
        _ => return None,
    })
}

/// The demo's set with its slot overrides on top.
fn demo_icons(values: &DemoValues) -> IconSet {
    let slots = [
        IconSlot::ChevronDown,
        IconSlot::CheckboxCheck,
        IconSlot::Eye,
    ];
    SLOTS
        .iter()
        .zip(slots)
        .fold(
            icon_set(&values.str("set")),
            |set, ((control, _), slot)| match icon(&values.str(control)) {
                Some(glyph) => set.with(slot, glyph),
                None => set,
            },
        )
}

fn demo_code(values: &DemoValues) -> String {
    let mut code = match values.str("set").as_str() {
        "lucide" => "icons: IconSet::new()".to_string(),
        set => format!("icons: IconSet::{set}()"),
    };
    for (control, slot) in SLOTS {
        let glyph = values.str(control);
        if glyph != "set" {
            code += &format!(
                "\n    .with(IconSlot::{slot}, pictogram_icons_lucide::{glyph}::outlined)"
            );
        }
    }
    code
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
                    ". Slots you leave empty keep lucide, libero's default. Nested providers merge: the inner one wins per slot. More on "
                    Anchor { to: Route::ProvidersPage {}, "providers" }
                    " in one page."
                }
                Text {
                    "One slot: "
                    Code { source: "IconSet::new().with(IconSlot::Close, icon)" }
                    ". A whole set: turn on its libero feature ("
                    Code { source: "icons-material" }
                    ", "
                    Code { source: "icons-tabler" }
                    ", "
                    Code { source: "icons-bootstrap" }
                    ", "
                    Code { source: "icons-phosphor" }
                    ") and start from its constructor, such as "
                    Code { source: "IconSet::material_rounded()" }
                    ". Only the set you call is compiled into your app. Brand marks are no slots: they name a service."
                }
            },
            // snippet: ignore - the sets need libero's `icons-*` features, off in its doc-tests
            Demo {
                component: "IconProvider",
                children_text: "",
                children_code: "Flex {{ direction: \"column\", gap: \"md\",\n    Select {{ label: \"Fruit\", value: Fruit::Apple, onchange: |_| {{}} }}\n    Checkbox {{ label: \"Ripe\", checked: true, onchange: |_| {{}} }}\n    PasswordField {{ label: \"Password\" }}\n    NumberField {{ label: \"Quantity\", steppers: true, value: Some(1), onchange: |_: Option<i32>| {{}} }}\n    Pagination {{ total: 3, page: 2, onchange: |_| {{}}, aria_label: \"Pages\", with_edges: true }}\n}}",
                controls: vec![
                    Control::select("set", SETS).code(|_, values| vec![demo_code(values)]),
                    Control::select("chevron", CHEVRONS).code(|_, _| vec![]),
                    Control::select("check", CHECKS).code(|_, _| vec![]),
                    Control::select("eye", EYES).code(|_, _| vec![]),
                ],
                render: move |values: DemoValues| rsx! {
                    IconProvider { icons: demo_icons(&values),
                        Flex { direction: "column", gap: "md",
                            Select { label: "Fruit", value: Fruit::Apple, onchange: |_| {} }
                            Checkbox { label: "Ripe", checked: true, onchange: |_| {} }
                            PasswordField { label: "Password" }
                            NumberField { label: "Quantity", steppers: true, value: Some(1), onchange: |_: Option<i32>| {} }
                            Pagination { total: 3, page: 2, onchange: |_| {}, aria_label: "Pages", with_edges: true }
                        }
                    }
                },
            }
        }
    }
}
