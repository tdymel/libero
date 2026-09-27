//! The shared field props (todo 449): `readonly` and `disabled` on fields, and the removable chip in RTL.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Cascader, CascaderOption, DateField, Flex, MultiSelect, RangeSlider, Rating, TagsField,
    TextField, Textarea,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/field-props/readonly", || rsx! { ReadonlyPage {} }),
    ("/field-props/chips", || rsx! { ChipsPage {} }),
];

#[component]
fn ReadonlyPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TextField { id: "text", label: "Name", description: "As on the card.", helper: "Read only.", value: "Ada", readonly: true, required: true, oninput: move |_| {} }
            Textarea { id: "area", label: "Notes", value: "Hello", readonly: true, oninput: move |_| {} }
            Rating { id: "rating", label: "Stars", value: 3.0f64, readonly: true, onchange: move |_| {} }
            RangeSlider { id: "range", label: "Price", value: (20.0f64, 80.0f64), readonly: true, oninput: move |_| {} }
            Cascader {
                id: "cascader",
                label: "Category",
                data: vec![CascaderOption::<String>::new("fruit", "Fruit").children(vec![
                    CascaderOption::new("apple", "Apple"),
                ])],
                value: Some("apple".to_string()),
                readonly: true,
                onchange: move |_| {},
            }
            DateField {
                id: "date",
                label: "Due",
                value: NaiveDate::from_ymd_opt(2026, 1, 2),
                readonly: true,
                onchange: move |_| {},
            }
        }
    }
}

#[component]
fn ChipsPage() -> Element {
    let mut tags =
        use_signal(|| vec!["rust".to_string(), "dioxus".to_string(), "wasm".to_string()]);
    let mut toppings = use_signal(|| vec!["Cheese".to_string(), "Olives".to_string()]);
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            TagsField {
                id: "tags",
                label: "Tags",
                value: tags(),
                onchange: move |next| tags.set(next),
            }
            MultiSelect {
                id: "multi",
                label: "Toppings",
                options: vec!["Cheese".to_string(), "Olives".to_string(), "Ham".to_string()],
                value: toppings(),
                onchange: move |next| toppings.set(next),
            }
        }
    }
}
