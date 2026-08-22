use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap};
use crate::icons::FileIcon;
use dioxus::prelude::*;
use libero::components::{Code, Icon, Input, TabValue, Tabs, Text};

/// The enum is the tab strip, so the snippet has to show it.
const SECTION_ENUM: &str = r#"#[derive(Clone, PartialEq, TabValue)]
enum Section {
    Account,
    #[tab(label = "Admin area")]
    Admin,
    Billing,
}

"#;

/// Printed verbatim when the `labels` control asks for it, and rendered by the
/// closure right below - the block is a promise that the two are the same.
const TRANSLATED: &str = r#"label: |section: Section| match section {
    Section::Account => "Konto".to_string(),
    Section::Admin => "Verwaltung".to_string(),
    Section::Billing => "Rechnung".to_string(),
}"#;

// A tab is a `button`, so its content has to stay phrasing content: `Icon` is
// an inline-flex `span` (and it is what sizes the raw svg), a `Flex` is a `div`.
const RICH: &str = r#"render_label: |section: Section| rsx! {
    Icon { variant: "transparent", size: "sm", FileIcon {} }
    "{section.label()}"
}"#;

#[derive(Clone, PartialEq, TabValue)]
enum Section {
    Account,
    #[tab(label = "Admin area")]
    Admin,
    Billing,
}

fn translated(section: Section) -> String {
    match section {
        Section::Account => "Konto".to_string(),
        Section::Admin => "Verwaltung".to_string(),
        Section::Billing => "Rechnung".to_string(),
    }
}

#[component]
pub fn TabsPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        DocPage {
            title: "Tabs",
            lead: rsx! {
                Text {
                    "One strip of tabs over an enum, with the selected tab's panel below it. The "
                    "tabs are the enum's variants - "
                    Code { source: "#[derive(TabValue)]" }
                    " lists them in declaration order and names each one - and "
                    Code { source: "panel" }
                    " is a match over the same type, so a forgotten or misspelled tab is a "
                    "compile error rather than a blank page. Only the selected panel is built at "
                    "all; the others cost nothing until they are picked."
                }
                Text {
                    Code { source: "label" }
                    " overrides what the derive named a tab, and it runs during render - so it "
                    "can read a locale from a signal or from context, and the strip repaints when "
                    "that changes. "
                    Code { source: "render_label" }
                    " fills the tab with rsx instead (an icon, a badge) while "
                    Code { source: "label" }
                    " still supplies the accessible name."
                }
            },
            DocSection {
                title: "Usage",
                Demo {
                    component: "Tabs",
                    children_text: "",
                    // Printed above the snippet: the strip is the enum, so the
                    // code block is a lie without it.
                    wrap: Wrap(|_: &DemoValues, source: &str| format!("{SECTION_ENUM}{source}")),
                    fixed: vec![
                        "value: section()".to_string(),
                        "onchange: move |next| section.set(next)".to_string(),
                        "panel: |section: Section| match section {\n    Section::Account => rsx! { \"Account settings\" },\n    Section::Admin => rsx! { \"Admin area\" },\n    Section::Billing => rsx! { \"Billing details\" },\n}".to_string(),
                    ],
                    controls: vec![
                        // `label` and `render_label` are one choice, not two
                        // switches: a tab has one set of labels.
                        Control::toggle("labels", ["derived", "translated", "rich"])
                            .labels(["Derived", "Translated", "Rich"])
                            .code(|_, values| match values.str("labels").as_str() {
                                "translated" => vec![TRANSLATED.to_string()],
                                "rich" => vec![RICH.to_string()],
                                _ => vec![],
                            }),
                        Control::color(
                            "color",
                            ["primary", "secondary", "success", "error", "warning", "info"],
                        ),
                        Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                            .default("md"),
                        Control::switch("full_width"),
                    ],
                    render: move |values: DemoValues| rsx! {
                        Tabs {
                            color: match values.str("color").as_str() {
                                "primary" => Input::None,
                                color => Input::from(color),
                            },
                            size: values.str("size"),
                            full_width: values.str("full_width") == "true",
                            label: match values.str("labels").as_str() {
                                "translated" => Some(Callback::new(translated)),
                                _ => None,
                            },
                            render_label: match values.str("labels").as_str() {
                                "rich" => Some(Callback::new(|section: Section| {
                                    rsx! {
                                        Icon { variant: "transparent", size: "sm", FileIcon {} }
                                        "{section.label()}"
                                    }
                                })),
                                _ => None,
                            },
                            value: section(),
                            onchange: move |next| section.set(next),
                            panel: |section: Section| match section {
                                Section::Account => rsx! { "Account settings" },
                                Section::Admin => rsx! { "Admin area" },
                                Section::Billing => rsx! { "Billing details" },
                            },
                        }
                    },
                }
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "The strip is a "
                    Code { source: "role=\"tablist\"" }
                    " of buttons, each pointing at its panel through "
                    Code { source: "aria-controls" }
                    ", and the panel points back with "
                    Code { source: "aria-labelledby" }
                    ". Only the selected tab is in the tab order; Left and Right move between "
                    "tabs and select as they go, Home and End jump to the ends."
                }
                Text {
                    "A tab named in "
                    Code { source: "disabled" }
                    " gets "
                    Code { source: "aria-disabled" }
                    " rather than the "
                    Code { source: "disabled" }
                    " attribute, so it still reads to a screen reader and the arrow keys simply "
                    "step over it."
                }
            }
        }
    }
}
