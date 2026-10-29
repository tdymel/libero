//! The list fields below `sm`: `Select`, `MultiSelect` and `PhoneField` open the phone
//! sheet (todos 3897, 3898, 3899, 3925), and a low custom `Cascader` trigger clears it (3914).
//! A searchable `MultiSelect` and a grouped `Select` check touch focus and targets (4834, 4835).

use dioxus::prelude::*;
use libero::components::{
    Button, Cascader, CascaderDropdown, CascaderOption, ChronoField, ColorField, Flex, MultiSelect,
    NumberField, OptionList, Options, PhoneField, Select, Tabs, Text, TouchInput, use_cascader,
};
use libero::chrono::NaiveDate;

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/select-phone", || rsx! { SelectPhonePage {} }),
    ("/select-phone/low-trigger", || rsx! { LowTriggerPage {} }),
    ("/select-phone/low-select", || rsx! { LowSelectPage {} }),
    ("/select-phone/low-color", || rsx! { LowColorPage {} }),
    ("/select-phone/low-phone", || rsx! { LowPhonePage {} }),
    ("/select-phone/low-date", || rsx! { LowDatePage {} }),
    ("/select-phone/low-number", || rsx! { LowNumberPage {} }),
    (
        "/select-phone/open-on-focus",
        || rsx! { OpenOnFocusPage {} },
    ),
];

/// An `open_on_focus` multi-select: closing its sheet must not reopen it (3982).
#[component]
fn OpenOnFocusPage() -> Element {
    let mut many = use_signal(|| vec![Fruit::Apple]);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            MultiSelect {
                label: "Many",
                open_on_focus: true,
                value: many(),
                onchange: move |next| many.set(next),
            }
        }
    }
}

/// One of each list field, values echoed.
#[component]
fn SelectPhonePage() -> Element {
    let mut plain = use_signal(|| None::<Fruit>);
    let mut searched = use_signal(|| None::<Fruit>);
    let mut many = use_signal(|| vec![Fruit::Apple]);
    let mut many_searched = use_signal(Vec::<Fruit>::new);
    let mut grouped = use_signal(|| Some(Fruit::Banana));
    let mut ore = use_signal(|| None::<String>);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            div { id: "plain",
                Select { label: "Plain", value: plain(), onchange: move |next| plain.set(next) }
            }
            div { id: "searched",
                Select {
                    label: "Searched",
                    searchable: true,
                    value: searched(),
                    onchange: move |next| searched.set(next),
                }
            }
            div { id: "many",
                MultiSelect { label: "Many", value: many(), onchange: move |next| many.set(next) }
            }
            div { id: "many-searched",
                MultiSelect {
                    label: "Many searched",
                    searchable: true,
                    value: many_searched(),
                    onchange: move |next| many_searched.set(next),
                }
            }
            // The selection fits at the top, under the first heading (4835).
            div { id: "grouped",
                Select {
                    label: "Grouped",
                    options: OptionList::grouped()
                        .group("Orchard", [Fruit::Apple, Fruit::Cherry])
                        .group("Other", [Fruit::Banana, Fruit::Damson, Fruit::Elderberry]),
                    value: grouped(),
                    onchange: move |next| grouped.set(next),
                }
            }
            div { id: "phone",
                PhoneField { label: "Phone", country: "DE" }
            }
            div { id: "cascade",
                Cascader {
                    label: "Ore",
                    searchable: true,
                    data: ores(),
                    value: ore(),
                    onchange: move |next: Option<String>| ore.set(next),
                }
            }
            Text { id: "echo", "{plain:?} {searched:?} {many:?}" }
        }
    }
}

fn ores() -> Vec<CascaderOption<String>> {
    vec![
        CascaderOption::new("t1", "Tier 1").children(vec![
            CascaderOption::new("iron", "Iron ore"),
            CascaderOption::new("copper", "Copper ore"),
        ]),
        CascaderOption::new("t2", "Tier 2").children(vec![CascaderOption::new("gold", "Gold ore")]),
    ]
}

/// A `Select` in the screen's foot, in frame but under where the sheet lands (4580).
#[component]
fn LowSelectPage() -> Element {
    let mut fruit = use_signal(|| None::<Fruit>);

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            div { style: "height: calc(100vh - 170px)" }
            div { id: "low",
                Select { label: "Low", value: fruit(), onchange: move |next| fruit.set(next) }
            }
            // Room to scroll the trigger up by the sheet's height.
            div { style: "height: 100vh" }
        }
    }
}

/// A field with a helper in the screen's foot: the sheet clears the helper too (4907).
/// Tabs above, as the docs page's: a tap on one only closes the sheet (4907, 4916).
#[component]
fn LowField(children: Element) -> Element {
    let mut section = use_signal(|| Section::Account);

    rsx! {
        // Fixed, so it stays in view once the field scrolls above the sheet.
        div { style: "position: fixed; top: 0; left: 0; right: 0; z-index: 1; background: white",
            Tabs {
                aria_label: "Sections",
                value: section(),
                onchange: move |next| section.set(next),
                options: OptionList::from_options(),
                panel: |_: Section| rsx! { "panel" },
            }
        }
        Flex { direction: "column", gap: "md", padding: "md",
            div { style: "height: calc(100vh - 170px)" }
            div { id: "low", {children} }
            div { style: "height: 100vh" }
        }
    }
}

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    Billing,
}

#[component]
fn LowColorPage() -> Element {
    rsx! {
        LowField {
            ColorField { label: "Brand color", helper: "Shown in the theme preview." }
        }
    }
}

#[component]
fn LowPhonePage() -> Element {
    rsx! {
        LowField {
            PhoneField { label: "Phone", country: "DE", helper: "Include the area code." }
        }
    }
}

#[component]
fn LowDatePage() -> Element {
    let mut due = use_signal(|| None::<NaiveDate>);

    rsx! {
        LowField {
            ChronoField::<NaiveDate> { label: "Due", value: due(), onchange: move |next| due.set(next) }
        }
    }
}

#[component]
fn LowNumberPage() -> Element {
    let mut count = use_signal(|| Some(3u32));

    rsx! {
        LowField {
            NumberField::<u32> {
                label: "Count",
                touch_input: TouchInput::Spin,
                value: count(),
                onchange: move |next| count.set(next),
            }
        }
    }
}

/// A custom trigger at the screen's foot, which the sheet would cover (3914).
#[component]
fn LowTriggerPage() -> Element {
    let mut ore = use_signal(|| None::<String>);
    let cascader = use_cascader(ores(), ore(), move |next| ore.set(next));

    rsx! {
        Flex { direction: "column", gap: "md", padding: "md",
            div { style: "height: calc(100vh - 60px)" }
            CascaderDropdown { state: cascader.clone(),
                Button { attributes: cascader.a11y_attributes(), "Ore" }
            }
            Text { id: "picked", {ore().unwrap_or_default()} }
            // Room to scroll the trigger up by the sheet's height.
            div { style: "height: 100vh" }
        }
    }
}
