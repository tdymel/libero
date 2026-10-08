//! Bare `Combobox`: a button trigger and `ComboboxOption` rows, as the docs'
//! usage. The pick is the first row, the one the list opens on.

use dioxus::prelude::*;
use libero::components::{
    Button, Combobox, ComboboxOption, ComboboxOptionArgs, OptionList, Options, TextField,
    use_combobox,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/combobox", || rsx! { ComboboxPage {} }),
    ("/combobox/refused", || rsx! { RefusedPage {} }),
    ("/combobox/none", || rsx! { NonePage {} }),
    ("/combobox/suggest", || rsx! { SuggestPage {} }),
];

/// A suggestion list on a text field, as the docs' demo: `#typed` echoes the field's text (2458).
#[component]
fn SuggestPage() -> Element {
    let fruit = use_combobox();
    let mut text = use_signal(String::new);
    let matches: Vec<Fruit> = Fruit::options()
        .iter()
        .copied()
        .filter(|f| f.label().to_lowercase().contains(&text().to_lowercase()))
        .collect();

    rsx! {
        Combobox {
            state: fruit,
            options: matches,
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        text.set(o.value.label());
                        fruit.close();
                    },
                    "{o.value.label()}"
                }
            },
            TextField {
                label: "Fruit",
                value: text(),
                attributes: fruit.a11y_attributes(),
                onblur: move |_| fruit.close(),
                oninput: move |next: String| {
                    text.set(next);
                    fruit.open();
                },
            }
        }
        span { id: "typed", "{text}" }
    }
}

/// No options and no `empty`: an open list draws and says "No results" (1269).
#[component]
fn NonePage() -> Element {
    let fruit = use_combobox();

    rsx! {
        Combobox {
            state: fruit,
            options: Vec::<Fruit>::new(),
            option: move |_: ComboboxOptionArgs<Fruit>| rsx! {},
            Button {
                variant: "outlined",
                attributes: named(fruit.a11y_attributes()),
                onclick: move |_| fruit.toggle(),
                "Pick a fruit"
            }
        }
    }
}

/// Apple and Cherry are refused, so the list opens on Banana.
#[component]
fn RefusedPage() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);

    rsx! {
        Combobox {
            state: fruit,
            options: OptionList::from_options()
                .disabling(|f| matches!(f, Fruit::Apple | Fruit::Cherry)),
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        picked.set(Some(o.value));
                        fruit.close();
                    },
                    "{o.value.label()}"
                }
            },
            Button {
                variant: "outlined",
                attributes: named(fruit.a11y_attributes()),
                onclick: move |_| fruit.toggle(),
                onblur: move |_| fruit.close(),
                "Pick a fruit"
            }
        }
        span { id: "picked", "{picked():?}" }
    }
}

#[component]
fn ComboboxPage() -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| Some(Fruit::Apple));

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
                attributes: named(fruit.a11y_attributes()),
                onclick: move |_| fruit.toggle(),
                onblur: move |_| fruit.close(),
                match picked() {
                    Some(fruit) => rsx! { "{fruit.label()}" },
                    None => rsx! { "Pick a fruit" },
                }
            }
        }
    }
}

/// A `combobox` button takes no name from its text.
fn named(mut attributes: Vec<Attribute>) -> Vec<Attribute> {
    attributes.push(Attribute::new("aria-label", "Fruit", None, false));
    attributes
}
