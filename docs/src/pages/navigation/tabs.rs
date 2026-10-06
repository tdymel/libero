use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, a11y, prop, props};
use libero::components::Pictogram;
use pictogram_icons_lucide as lucide;

use dioxus::prelude::*;
use libero::components::{
    Code, Icon, Input, OptionLabel, OptionList, Options, Tabs, TabsPart, Text,
};

/// The enum is the tab strip, so the snippet has to show it.
const SECTION_ENUM: &str = r#"#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    #[option(label = "Admin area")]
    Admin,
    Billing,
}

"#;

/// Printed verbatim when the `labels` control asks for it, and rendered by the
/// closure right below, so the two must match.
// snippet: after SECTION_ENUM
// snippet: let mut section = use_signal(|| Section::Account);
// snippet: in Tabs { value: section(), onchange: move |next| section.set(next), panel: |_: Section| rsx! {}, .. }
const RENAMED: &str = r#"option_label: |section: Section| -> OptionLabel {
    match section {
        Section::Account => "Konto".into(),
        Section::Admin => "Verwaltung".into(),
        Section::Billing => "Rechnung".into(),
    }
}"#;

// A tab is a `button`, so its content has to stay phrasing content: `Icon` is
// an inline-flex `span` (and it is what sizes the raw svg), a `Flex` is a `div`.
// snippet: after SECTION_ENUM
// snippet: let mut section = use_signal(|| Section::Account);
// snippet: in Tabs { value: section(), onchange: move |next| section.set(next), panel: |_: Section| rsx! {}, .. }
const RICH: &str = r#"option_label: |section: Section| OptionLabel::rich(
    section.label(),
    rsx! {
        Icon { variant: "standard", size: "sm", Pictogram { icon: lucide::file::outlined } }
        "{section.label()}"
    },
)"#;

/// The flag sits on the option, inside the one `options` prop - a tab this
/// strip refuses, rather than a section the type refuses everywhere.
// snippet: after SECTION_ENUM
// snippet: let mut section = use_signal(|| Section::Account);
// snippet: in Tabs { value: section(), onchange: move |next| section.set(next), panel: |_: Section| rsx! {}, .. }
const DISABLED_OPTION: &str =
    r#"options: OptionList::from_options().disabling(|section| *section == Section::Billing)"#;

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    #[option(label = "Admin area")]
    Admin,
    Billing,
}

fn renamed(section: Section) -> OptionLabel {
    match section {
        Section::Account => "Konto".into(),
        Section::Admin => "Verwaltung".into(),
        Section::Billing => "Rechnung".into(),
    }
}

fn rich(section: Section) -> OptionLabel {
    OptionLabel::rich(
        section.label(),
        rsx! {
            Icon { variant: "standard", size: "sm", Pictogram { icon: lucide::file::outlined } }
            "{section.label()}"
        },
    )
}

#[component]
pub fn TabsPage() -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        DocPage {
            title: "Tabs",
            source: "libero/src/components/navigation/tabs",
            markdown: "/md/tabs.md",
            properties: vec![
                props("Tabs", vec![
                    prop("value", "T").default("required").doc("The selected tab. Pair it with `onchange`."),
                    prop("onchange", "EventHandler<T>").doc("Called with the tab that should become selected."),
                    prop("panel", "Callback<T, Element>").doc("The body of the selected tab. Only the selected panel is built."),
                    prop("options", "OptionSource<T>").default("T::options()").doc("The tabs to show. A `Vec<T>` converts, and an `OptionList<T>` can disable single tabs. Groups draw flattened. A source still loading draws no tabs."),
                    prop("option_label", "Callback<T, OptionLabel>")
                        .default("T::label()")
                        .doc("Overrides a tab's label. Runs during render, so it can read a locale."),
                    prop("size", "Size").default("md").doc("Tab strip size."),
                    prop("color", "ThemeAwareValue").default("primary").doc("Indicator and selected-label color."),
                    prop("full_width", "bool").default("false").doc("Tabs grow to fill the row, never below their label. A crowded strip still scrolls."),
                    prop("activation", "TabsActivation").default("Automatic").doc("`Automatic` selects as the arrows move. `Manual` moves only the focus, and Enter or Space selects. Use it for slow panels."),
                    prop("parts", "Parts<TabsPart>")
                        .doc("Styles for the inner parts in the Style API tab, under `sx`."),
                ])
                .parts("TabsPart", vec![
                    (TabsPart::List, "The `tablist` strip."),
                    (TabsPart::Tab, "One tab. The selected one has `aria-selected=\"true\"`."),
                    (TabsPart::Panel, "The selected tab's panel."),
                ]),
                props("OptionLabel", vec![
                    prop("name", "String").default("required").doc("The tab's visible text and accessible name."),
                    prop("content", "Element")
                        .doc("Drawn in place of the name, such as an icon, via `OptionLabel::rich`. `name` still names the tab. Keep it inline: an `Icon` fits, a `Flex` does not."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the strip at the selected tab, the only one in the tab order.")
                .key(["Left", "Right"], "Moves to the previous or next tab and selects it, skipping disabled ones.")
                .key(["Home", "End"], "Jumps to the first or last tab.")
                .key(["Enter", "Space"], "With `activation: TabsActivation::Manual`, where the arrows move only the focus: selects the focused tab.")
                .handles([
                    "Only the selected tab is in the tab order.",
                    "`aria_label` and `aria_labelledby` land on the tablist, not the root.",
                    "In a strip too wide for its box, the selected tab scrolls into view, also when `value` changes from outside.",
                ])
                .must([
                    "Name the strip with `aria_label` or `aria_labelledby`. Without either it warns in debug builds.",
                    "If you remove the focused tab from `options`, move the focus back to the strip yourself.",
                ])
                .example("A settings strip, `Tabs { aria_label: \"Settings\" }`: Tab lands on the selected tab, the arrows move and select, and Home and End jump to the first and last tab."),
            lead: rsx! {
                Text {
                    "A strip of tabs over an enum, with the selected tab's panel below it. "
                    Code { source: "#[derive(Options)]" }
                    " lists and names the tabs. "
                    Code { source: "panel" }
                    " matches on the same enum, so a missing tab does not compile. Only the "
                    "selected panel is built."
                }
            },
            // snippet: let mut section = use_signal(|| Section::Account);
            Demo {
                component: "Tabs",
                children_text: "",
                // Printed above the snippet: the strip is the enum, so the
                // code block is a lie without it.
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{SECTION_ENUM}{source}")),
                fixed: vec![
                    "aria_label: \"Settings\"".to_string(),
                    "value: section()".to_string(),
                    "onchange: move |next| section.set(next)".to_string(),
                    "panel: |section: Section| match section {\n    Section::Account => rsx! { \"Account settings\" },\n    Section::Admin => rsx! { \"Admin area\" },\n    Section::Billing => rsx! { \"Billing details\" },\n}".to_string(),
                ],
                controls: vec![
                    Control::toggle("labels", ["derived", "renamed", "rich"])
                        .labels(["Derived", "Renamed", "Rich"])
                        .code(|_, values| match values.str("labels").as_str() {
                            "renamed" => vec![RENAMED.to_string()],
                            "rich" => vec![RICH.to_string()],
                            _ => vec![],
                        }),
                    Control::color("color"),
                    Control::sizes("size")
                        .default("md"),
                    Control::toggle("activation", ["automatic", "manual"])
                        .labels(["Automatic", "Manual"])
                        .default("automatic"),
                    Control::switch("full_width"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named tab rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        match values.str("disabled_option") == "true" {
                            true => vec![DISABLED_OPTION.to_string()],
                            false => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    Tabs {
                        aria_label: "Settings",
                        options: {
                            let off = values.str("disabled_option") == "true";
                            OptionList::from_options()
                                .disabling(move |section| off && *section == Section::Billing)
                        },
                        color: match values.str("color").as_str() {
                            "primary" => Input::None,
                            color => Input::from(color),
                        },
                        size: values.str("size"),
                        full_width: values.str("full_width") == "true",
                        activation: values.str("activation"),
                        option_label: match values.str("labels").as_str() {
                            "renamed" => Some(Callback::new(renamed)),
                            "rich" => Some(Callback::new(rich)),
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

            DocSection {
                title: "Labels",
                Text {
                    Code { source: "option_label" }
                    " renames tabs during render, so the strip follows a locale signal. "
                    Code { source: "OptionLabel::rich" }
                    " draws a tab as rsx, such as an icon, and takes a text name for screen "
                    "readers."
                }
            }
        }
    }
}
