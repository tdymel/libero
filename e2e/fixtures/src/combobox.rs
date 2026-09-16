//! Bare `Combobox`: a button trigger and `ComboboxOption` rows, as the docs'
//! usage. The pick is the first row, the one the list opens on.

use dioxus::prelude::*;
use libero::components::{
    Button, Combobox, ComboboxOption, ComboboxOptionArgs, Options, use_combobox,
};

use crate::{Routes, common::Fruit};

pub const ROUTES: Routes = &[("/combobox", || rsx! { ComboboxPage {} })];

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
                attributes: fruit.a11y_attributes(),
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
