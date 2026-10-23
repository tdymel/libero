//! The list fields below `sm`: `Select`, `MultiSelect` and `PhoneField` open the phone
//! sheet (todos 3897, 3898, 3899, 3925), and a low custom `Cascader` trigger clears it (3914).

use dioxus::prelude::*;
use libero::components::{
    Button, Cascader, CascaderDropdown, CascaderOption, Flex, MultiSelect, PhoneField, Select,
    Text, use_cascader,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/select-phone", || rsx! { SelectPhonePage {} }),
    ("/select-phone/low-trigger", || rsx! { LowTriggerPage {} }),
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
