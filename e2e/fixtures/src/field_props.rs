//! The shared field props (todo 449): `readonly` and `disabled` on fields, and the removable chip in RTL.

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    Cascader, CascaderOption, Checkbox, DateField, Flex, MultiSelect, PasswordField, RangeSlider, Rating,
    Select, TagsField, TextField, Textarea,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/field-props/readonly", || rsx! { ReadonlyPage {} }),
    ("/field-props/chips", || rsx! { ChipsPage {} }),
    ("/field-props/description", || rsx! { DescriptionPage {} }),
    ("/field-props/description-below", || rsx! { DescriptionBelowPage {} }),
    ("/field-props/label-trailing", || rsx! { LabelTrailingPage {} }),
];

#[component]
fn LabelTrailingPage() -> Element {
    let add = |id: &str| rsx! { button { id: "{id}", r#type: "button", "Add" } };
    rsx! {
        Flex { direction: "column", gap: "md",
            div { id: "text", TextField { label: "Name", label_trailing: add("text-add"), oninput: move |_| {} } }
            div { id: "area", Textarea { label: "Notes", label_trailing: add("area-add"), oninput: move |_| {} } }
            div { id: "select",
                Select { label: "Pick", label_trailing: add("select-add"), options: vec!["Cheese".to_string()], onchange: move |_| {} }
            }
            div { id: "check", Checkbox { label: "Agree", label_trailing: add("check-add") } }
            div { id: "bare", TextField { aria_label: "Bare", label_trailing: add("bare-add"), oninput: move |_| {} } }
        }
    }
}

#[component]
fn DescriptionBelowPage() -> Element {
    let options = || vec!["Cheese".to_string(), "Olives".to_string()];
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "320px",
            div { id: "select",
                Select { label: "Pick", description: "One of these.", description_position: "below", options: options(), onchange: move |_| {} }
            }
            div { id: "multi",
                MultiSelect { label: "Pick", description: "Some of these.", description_position: "below", options: options(), onchange: move |_| {} }
            }
            div { id: "cascader",
                Cascader {
                    label: "Category",
                    description: "A path.",
                    description_position: "below",
                    data: vec![CascaderOption::<String>::new("fruit", "Fruit").children(vec![
                        CascaderOption::new("apple", "Apple"),
                    ])],
                    onchange: move |_| {},
                }
            }
            div { id: "password",
                PasswordField { label: "Password", description: "Eight or more.", description_position: "below", oninput: move |_| {} }
            }
        }
    }
}

#[component]
fn DescriptionPage() -> Element {
    rsx! {
        Flex { direction: "column", gap: "md", max_width: "480px",
            Flex { direction: "row", wrap: "nowrap", align: "flex-start", gap: "md",
                TextField { id: "below-a", label: "First", description: "As on the card.", description_position: "below", oninput: move |_| {} }
                TextField { id: "below-b", label: "Second", description_position: "below", oninput: move |_| {} }
            }
            Flex { direction: "row", wrap: "nowrap", align: "flex-start", gap: "md",
                TextField { id: "above-a", label: "First", description: "As on the card.", oninput: move |_| {} }
                TextField { id: "above-b", label: "Second", oninput: move |_| {} }
            }
        }
    }
}

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
