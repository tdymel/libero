use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Fields, Fieldset, Input, Rule, Text, TextField};

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Address {
    pub street: String,
    pub zip: String,
    pub city: String,
}

const FIELDSET_CODE: &str = r#"use dioxus::prelude::*;
use libero::components::{Fields, Fieldset, Rule, TextField};

#[derive(Clone, PartialEq, Default, Fields)]
struct Address {
    street: String,
    zip: String,
    city: String,
}

#[component]
fn AddressFieldset() -> Element {
    let address = use_store(Address::default);

    rsx! {
        Fieldset {
            label: "Delivery address",__PROPS__
            value: address,
            validate: [
                (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                    .error("A city needs its zip code.")
                    .on([Address::FIELDS.zip()]),
                (|a: &Address| !a.street.is_empty() || a.city.is_empty())
                    .warn("Without a street we can only deliver to a pickup point."),
            ],
            TextField {
                label: "Street",
                name: Address::FIELDS.street(),
            }
            TextField {
                label: "Zip code",
                name: Address::FIELDS.zip(),
            }
            TextField {
                label: "City",
                name: Address::FIELDS.city(),
            }
        }
    }
}"#;

fn fieldset_code(values: &DemoValues, _: &str) -> String {
    let mut props = String::new();
    if values.str("description") == "true" {
        props.push_str("\n            description: \"Where the parcel goes.\",");
    }
    if values.str("helper") == "true" {
        props.push_str("\n            helper: \"We deliver Monday to Saturday.\",");
    }
    match values.str("status").as_str() {
        "warning" => props.push_str(
            "\n            status: FieldStatus::Warning(\"We only ship within Germany.\".into()),",
        ),
        "error" => props.push_str("\n            status: \"We cannot deliver to this address.\","),
        _ => {}
    }
    if values.str("disabled") == "true" {
        props.push_str("\n            disabled: true,");
    }
    let code = FIELDSET_CODE.replace("__PROPS__", &props);
    match values.str("status").as_str() {
        "warning" => code.replace(
            "use libero::components::{Fields,",
            "use libero::components::{FieldStatus, Fields,",
        ),
        _ => code,
    }
}

fn silent(_: &Control, _: &DemoValues) -> Vec<String> {
    Vec::new()
}

#[component]
pub fn FieldsetPage() -> Element {
    rsx! {
        DocPage {
            title: "Fieldset",
            source: "libero/src/components/form/fieldset.rs",
            markdown: "/md/fieldset.md",
            properties: vec![props("Fieldset", vec![
                prop("label", "Caption").doc("The group's caption, rendered as its `<legend>`."),
                prop("description", "Caption").doc("Under the label."),
                prop("helper", "Caption").doc("Under the fields."),
                prop("status", "FieldStatus")
                    .default("Valid")
                    .doc("The group's own status, under the fields. A bare `&str` is an error."),
                prop("value", "Store<V>")
                    .doc("The group's own value, for a fieldset outside a `Form`. A fieldset with no `value`, `path` or rules needs `Fieldset::<()>` so Rust can infer its type."),
                prop("validate", "Validators<V>")
                    .doc("Rules over the group's value, one or an array. A rule with `.on(..)` shows on the fields it names. One without shows under the fields once any field in the group was touched."),
                prop("path", "FieldName<V>")
                    .doc("Where the group sits inside a `Form`'s value, such as `Order::FIELDS.address()`. The names of the fields inside and the paths in `validate` are relative to it."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables every field inside, nested fieldsets included. A field's own `disabled: false` cannot re-enable it."),
                prop("children", "Element").default("required").doc("The fields."),
            ]).extends("fieldset")],
            lead: rsx! {
                Text {
                    "Several fields that form one value, such as an address or a date range, under "
                    "one "
                    Code { source: "<legend>" }
                    ". The group has its own description, helper and status, and rules over its "
                    "value. Inside a "
                    Code { source: "Form" }
                    ", give it a "
                    Code { source: "path" }
                    ". On its own, give it a "
                    Code { source: "value" }
                    " store. The Form getting started page shows how to build reusable parts "
                    "around it."
                }
            },
            Demo {
                component: "Fieldset",
                children_text: "",
                wrap: Wrap(fieldset_code),
                controls: vec![
                    Control::toggle("status", ["auto", "warning", "error"])
                        .labels(["Auto", "Warning", "Error"])
                        .default("auto")
                        .code(silent),
                    Control::switch("description").code(silent),
                    Control::switch("helper").code(silent),
                    Control::switch("disabled").code(silent),
                ],
                render: move |values: DemoValues| rsx! {
                    AddressFieldset {
                        description: values.str("description") == "true",
                        helper: values.str("helper") == "true",
                        disabled: values.str("disabled") == "true",
                        status: values.str("status"),
                    }
                },
            }
        }
    }
}

#[component]
fn AddressFieldset(description: bool, helper: bool, disabled: bool, status: String) -> Element {
    let address = use_store(Address::default);

    rsx! {
        Fieldset {
            sx: libero::sx::sx().width("100%").max_width("320px"),
            label: "Delivery address",
            description: description.then(|| "Where the parcel goes.".to_string()),
            helper: helper.then(|| "We deliver Monday to Saturday.".to_string()),
            // "auto" sets no status, so the rules below decide it.
            status: match status.as_str() {
                "warning" => Input::Value(FieldStatus::Warning("We only ship within Germany.".to_string())),
                "error" => Input::Value(FieldStatus::Error("We cannot deliver to this address.".to_string())),
                _ => Input::None,
            },
            disabled,
            value: address,
            validate: [
                (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                    .error("A city needs its zip code.")
                    .on([Address::FIELDS.zip()]),
                (|a: &Address| !a.street.is_empty() || a.city.is_empty())
                    .warn("Without a street we can only deliver to a pickup point."),
            ],
            TextField {
                label: "Street",
                name: Address::FIELDS.street(),
            }
            TextField {
                label: "Zip code",
                name: Address::FIELDS.zip(),
            }
            TextField {
                label: "City",
                name: Address::FIELDS.city(),
            }
        }
    }
}
