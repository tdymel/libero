use crate::Route;
use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{
    Anchor, Button, Code, CodeBlock, Combobox, ComboboxOption, ComboboxOptionArgs, Flex, Options,
    Text, use_combobox,
};

const FRUIT: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

#[component]
fn FruitPicker() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);

    rsx! {
        Combobox {
            state: fruit,
            options: Fruit::options().to_vec(),
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    selected: picked() == Some(o.value),
                    onpick: move |_| {
                        picked.set(Some(o.value));
                        fruit.close();
                    },
                    "{o.value.label()}"
                }
            },
            Button {
                variant: "outlined",
                attributes: fruit.a11y_attributes(),
                onclick: move |_| fruit.toggle(),
                match picked() {
                    Some(fruit) => rsx! { "{fruit.label()}" },
                    None => rsx! { "Pick a fruit" },
                }
            }
        }
    }
}"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

/// `FRUIT`, rendered.
#[component]
fn FruitPicker() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);

    rsx! {
        Combobox {
            state: fruit,
            options: Fruit::options().to_vec(),
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    selected: picked() == Some(o.value),
                    onpick: move |_| {
                        picked.set(Some(o.value));
                        fruit.close();
                    },
                    "{o.value.label()}"
                }
            },
            Button {
                variant: "outlined",
                attributes: fruit.a11y_attributes(),
                onclick: move |_| fruit.toggle(),
                match picked() {
                    Some(fruit) => rsx! { "{fruit.label()}" },
                    None => rsx! { "Pick a fruit" },
                }
            }
        }
    }
}

#[component]
pub fn UseComboboxPage() -> Element {
    rsx! {
        DocPage {
            title: "use_combobox",
            source: "libero/src/components/form/combobox/state.rs",
            markdown: "/md/use_combobox.md",
            lead: rsx! {
                Text {
                    Code { source: "use_combobox() -> ComboboxState" }
                    " keeps a "
                    Code { source: "Combobox" }
                    "'s open state and active option in your scope, so your own trigger "
                    "opens, closes and wires it. Pass it as the combobox's "
                    Code { source: "state" }
                    ". "
                    Anchor { to: Route::ComboboxPage {}, "Combobox" }
                    " documents the component and its keyboard."
                }
            },

            DocSection {
                title: "Usage",
                Flex { align: "flex-start", FruitPicker {} }
                CodeBlock { source: FRUIT, language: "rust" }
            }

            DocSection {
                title: "Accessibility",
                Text {
                    Code { source: "a11y_attributes()" }
                    " gives the trigger its role, "
                    Code { source: "aria-haspopup" }
                    " and "
                    Code { source: "aria-expanded" }
                    ". While the list is open and has rows, it adds "
                    Code { source: "aria-controls" }
                    " and "
                    Code { source: "aria-activedescendant" }
                    ". Spread them on whatever control sits inside the combobox."
                }
            }
        }
    }
}
