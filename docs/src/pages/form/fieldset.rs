use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, FieldStatus, Fields, Fieldset, Rule, Text, TextField};

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
    let mut address = use_signal(Address::default);

    rsx! {
        Fieldset {
            legend: "Delivery address",__PROPS__
            value: address(),
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
                value: address().street,
                oninput: move |next| address.write().street = next,
            }
            TextField {
                label: "Zip code",
                name: Address::FIELDS.zip(),
                value: address().zip,
                oninput: move |next| address.write().zip = next,
            }
            TextField {
                label: "City",
                name: Address::FIELDS.city(),
                value: address().city,
                oninput: move |next| address.write().city = next,
            }
        }
    }
}"#;

const IN_FORM_CODE: &str = r#"use libero::components::{Button, Fields, Fieldset, Form, Rule, TextField, not_empty};

#[derive(Clone, PartialEq, Default, Fields)]
struct Order {
    name: String,
    #[fields(nested)]
    address: Address,
}

rsx! {
    Form {
        value: order(),
        TextField {
            label: "Name",
            name: Order::FIELDS.name(),
            value: order().name,
            oninput: move |next| order.write().name = next,
            validate: not_empty.error("Enter your name."),
        }
        Fieldset {
            legend: "Delivery address",
            // Where the group sits: its rules stay rooted at `Address`.
            path: Order::FIELDS.address(),
            value: order().address,
            validate: (|a: &Address| a.city.is_empty() || !a.zip.is_empty())
                .error("A city needs its zip code.")
                .on([Address::FIELDS.zip()]),
            TextField {
                label: "Zip code",
                // The full path - posts as "address.zip".
                name: Order::FIELDS.address().zip(),
                value: order().address.zip,
                oninput: move |next| order.write().address.zip = next,
            }
            // ..
        }
        Button { r#type: "submit", "Order" }
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
                prop("legend", "Caption").doc("The group's caption, a `<legend>`."),
                prop("description", "Caption").doc("Under the legend."),
                prop("helper", "Caption").doc("Under the fields."),
                prop("status", "FieldStatus")
                    .default("Valid")
                    .doc("The group's own status, under the fields. A bare `&str` is an error."),
                prop("value", "V")
                    .doc("The group's value, which `validate` checks. `V` is inferred from it."),
                prop("validate", "Validators<V>")
                    .doc("Composite rules over `value` - one rule, or an array. With `.on(..)` a status lands on the named fields; without, under the fields."),
                prop("path", "String")
                    .doc("Where `value` sits inside a `Form`, e.g. `Order::FIELDS.address()`. The rules' paths are put under it."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("`<fieldset disabled>`: disables every native control inside."),
                prop("children", "Element").doc("The fields."),
            ])],
            lead: rsx! {
                Text {
                    "Several fields that form one value - an address, a date range - grouped in a "
                    Code { source: "<fieldset>" }
                    " under one "
                    Code { source: "<legend>" }
                    ", with a description, helper and status of its own. Its "
                    Code { source: "validate" }
                    " rules run over the group's "
                    Code { source: "value" }
                    " and put a problem on the fields they name. It works on its own or inside a "
                    Code { source: "Form" }
                    "."
                }
            },
            Demo {
                component: "Fieldset",
                children_text: "",
                wrap: Wrap(fieldset_code),
                controls: vec![
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
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
                    "Inside a "
                    Code { source: "Form" }
                    ", give the group a "
                    Code { source: "path" }
                    ". Its rules keep using paths rooted at the group's own type, and the fieldset "
                    "puts its prefix in front of them - so "
                    Code { source: "Address::FIELDS.zip()" }
                    " reaches the field named "
                    Code { source: "address.zip" }
                    ". On its own, a fieldset opens a scope of its own and needs no "
                    Code { source: "path" }
                    "."
                }
                CodeBlock { source: IN_FORM_CODE, language: "rust" }
            }
            DocSection { title: "Accessibility",
                Text {
                    "The legend names the group, so a screen reader announces "
                    "\"Delivery address\" as focus enters it. The description, helper and status join "
                    "the fieldset's "
                    Code { source: "aria-describedby" }
                    ". A rule that names no field shows in the group's status slot once any field "
                    "in the group was touched."
                }
            }
        }
    }
}

#[component]
fn AddressFieldset(description: bool, helper: bool, disabled: bool, status: String) -> Element {
    let mut address = use_signal(Address::default);

    rsx! {
        Fieldset {
            sx: libero::sx::sx().width("320px"),
            legend: "Delivery address",
            description: description.then(|| "Where the parcel goes.".to_string()),
            helper: helper.then(|| "We deliver Monday to Saturday.".to_string()),
            status: match status.as_str() {
                "warning" => FieldStatus::Warning("We only ship within Germany.".to_string()),
                "error" => FieldStatus::Error("We cannot deliver to this address.".to_string()),
                _ => FieldStatus::Valid,
            },
            disabled,
            value: address(),
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
                value: address().street,
                oninput: move |next| address.write().street = next,
            }
            TextField {
                label: "Zip code",
                name: Address::FIELDS.zip(),
                value: address().zip,
                oninput: move |next| address.write().zip = next,
            }
            TextField {
                label: "City",
                name: Address::FIELDS.city(),
                value: address().city,
                oninput: move |next| address.write().city = next,
            }
        }
    }
}
