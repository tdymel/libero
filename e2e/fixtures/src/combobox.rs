//! Bare `Combobox`: a button trigger and `ComboboxOption` rows, as the docs'
//! usage. The pick is the first row, the one the list opens on.

use dioxus::prelude::*;
use libero::components::{
    Button, Combobox, ComboboxOption, ComboboxOptionArgs, OptionList, Options, use_combobox,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[
    ("/combobox", || rsx! { ComboboxPage {} }),
    ("/combobox/refused", || rsx! { RefusedPage {} }),
    ("/combobox/none", || rsx! { NonePage {} }),
];

/// No options and no `empty`: an open list draws nothing.
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
