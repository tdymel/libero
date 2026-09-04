use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{
    Code, CodeBlock, FieldStatus, Fields, Fieldset, Input, Rule, Text, TextField,
};

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

const IN_FORM_CODE: &str = r#"Form {
    value: order,
    Fieldset {
        label: "Delivery address",
        path: Order::FIELDS.address(),               // the group's value is order.address
        validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
            .error("A city needs its zip code.")
            .on([Address::FIELDS.zip()]),            // rules are rooted at Address
        TextField { label: "Zip code", name: Address::FIELDS.zip() }  // posts "address.zip"
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
                    .doc("The group's own value, for a fieldset outside a `Form`. Inside one, the value is the form's at `path`."),
                prop("validate", "Validators<V>")
                    .doc("Composite rules over `value` - one rule, or an array. With `.on(..)` a status lands on the named fields; without, under the fields, once any field in the group was touched."),
                prop("path", "FieldName<V>")
                    .doc("Where the group sits inside a `Form`'s value, e.g. `Order::FIELDS.address()`. The names of the fields inside and the paths of `validate` are relative to it."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables every field inside, nested fieldsets included - their look and their own controls, not only native ones. A field's own `disabled: false` cannot re-enable it."),
                prop("children", "Element").doc("The fields."),
            ])],
            lead: rsx! {
                Text {
                    "Several fields that form one value - an address, a date range - under one "
                    Code { source: "<legend>" }
                    ", with a description, helper and status of its own, and rules over the group's "
                    "value. Building reusable parts around it is explained in Forms: Getting Started."
                }
            },
            Demo {
                component: "Fieldset",
                children_text: "",
                wrap: Wrap(fieldset_code),
                controls: vec![
                    Control::toggle("status", ["auto", "warning", "error"])
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
            DocSection { title: "Inside a Form",
                Text {
                    "Give the group a "
                    Code { source: "path" }
                    ". Field names and rule paths inside are relative to it. On its own a fieldset "
                    "takes a "
                    Code { source: "value" }
                    " store instead."
                }
                CodeBlock { source: IN_FORM_CODE, language: "rust" }
            }
        }
    }
}

#[component]
fn AddressFieldset(description: bool, helper: bool, disabled: bool, status: String) -> Element {
    let address = use_store(Address::default);

    rsx! {
        Fieldset {
            sx: libero::sx::sx().width("320px"),
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
