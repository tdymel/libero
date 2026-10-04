use crate::components::{DemoFile, DocPage, DocSection};
use dioxus::prelude::*;
use libero::components::{Code, CodeBlock, Text};

// snippet: ignore - a diagram
const SHAPE_CODE: &str = r#"TextField                      // a field, one value with its label and rules
  └ PasswordField, EmailField  // specialized fields, each with a narrower contract
AddressFieldset                // a composed part, fields grouped into one value
  └ Fieldset { path: .. }
OrderForm                      // the whole value, its parts submitted together
  └ Form { value: order }"#;

const RULES_CODE: &str = r#"use libero::components::{Rule, Validator, max_length, not_empty};

// A rule is any `Fn(&V) -> bool`, so a plain function is one.
fn no_spaces(value: &String) -> bool {
    !value.contains(' ')
}

// A rule with a parameter is a function that returns one, like `min_length(8)`.
fn starts_with(prefix: &'static str) -> impl Fn(&String) -> bool {
    move |value| value.starts_with(prefix)
}

// A message turns a rule into a `Validator`. `.error` blocks a submit, `.warn` never does.
let username: Validator<String> = no_spaces.error("No spaces, please.");
let coupon = starts_with("LIB-").and(max_length(12)).warn("That does not look like our coupon.");

// A closure works too, and composite rules are closures over the whole value.
let adult = (|age: &Option<u8>| age.is_some_and(|age| age >= 18)).error("You must be 18.");

// A reusable rule for your own domain is a function that returns a `Validator`.
fn required(field: &str) -> Validator<String> {
    not_empty.error(format!("Enter your {field}."))
}"#;

// snippet: ignore - a list of signatures
const CATALOG_CODE: &str = r#"use libero::components::{Rule, is_email, max, max_length, min, min_length, not_empty};

not_empty                        // String (whitespace is empty), Option<T>, Vec<T>, bool
min_length(3) / max_length(12)   // characters, not bytes
min(18) / max(99)                // any PartialOrd value, bounds included
is_email                         // a shape check, one @ and a dotted domain

rule.error("..")                 // an error blocks a submit
rule.warn("..")                  // a warning never does
rule.and(other) / rule.or(other) // compose before the message"#;

// snippet: ignore - a list of paths, two of them wrong on purpose
const PATHS_CODE: &str = r#"#[derive(Clone, PartialEq, Default, Fields)]
struct Order {
    email: String,
    #[fields(nested)] // `Address` derives `Fields` too
    shipping: Address,
}

Order::FIELDS.email()               // FieldPath<Order, String>   posts as "email"
Order::FIELDS.shipping().zip()      // FieldPath<Order, String>   posts as "shipping.zip"
Order::FIELDS.shipping().path()     // FieldPath<Order, Address>  "shipping"
Order::FIELDS.shiping().zip()       // does not compile
path!(Order => shipping.zip)        // the same path, for a type that cannot derive

TextField { name: Order::FIELDS.email() }         // binds and posts
TextField { name: Order::FIELDS.same_billing() }  // does not compile, a TextField holds a String
rule.on([Order::FIELDS.email()])                  // a composite rule over `Order` names a field"#;

// snippet: item #[derive(Clone, PartialEq, Default, Fields)] struct Address { zip: String }
// snippet: item #[derive(Clone, PartialEq, Default, Fields)] struct Order { email: String, #[fields(nested)] shipping: Address }
// snippet: let mut draft = use_signal(String::new);
const BINDING_CODE: &str = r#"let order = use_store(Order::default);

Form {
    value: order,                                  // the form holds the store
    TextField { name: Order::FIELDS.email() }      // reads and writes order.email
    Fieldset {
        path: Order::FIELDS.shipping(),            // everything inside is relative to order.shipping
        TextField { name: Address::FIELDS.zip() }  // reads and writes order.shipping.zip, posts "shipping.zip"
    }
    TextField {                                    // a handler of its own takes the field back
        name: Order::FIELDS.email(),
        value: draft(),
        oninput: move |next| draft.set(next),
    }
}"#;

// Each file prints from its `demo-code: start` marker, below the imports it compiles with.
#[rustfmt::skip]
mod address_fieldset;
#[rustfmt::skip]
mod email_field;
#[rustfmt::skip]
mod order_form;
use address_fieldset::{Address, AddressFieldset};
use email_field::EmailField;
use order_form::OrderForm;

const SPECIALIZE_CODE: DemoFile = DemoFile(include_str!("getting_started/email_field.rs"));
const COMPOSE_CODE: DemoFile = DemoFile(include_str!("getting_started/address_fieldset.rs"));
const FORM_CODE: DemoFile = DemoFile(include_str!("getting_started/order_form.rs"));

#[component]
pub fn FormGettingStartedPage() -> Element {
    rsx! {
        DocPage {
            // Not "Getting started": the About page has that title; the nav label stays.
            title: "Form getting started",
            markdown: "/md/form_getting_started.md",
            lead: rsx! {
                Text {
                    "libero's forms are built in layers. A field holds one value. A specialized field "
                    "is a field with a narrower contract. A composed part groups fields into one value "
                    "with a "
                    Code { source: "Fieldset" }
                    ", and a "
                    Code { source: "Form" }
                    " holds the whole value in one store, validates it and submits it."
                }
            },
            DocSection { title: "The layers",
                Text {
                    "Rules live at the layer that can see the values they check, and typed paths from "
                    Code { source: "#[derive(Fields)]" }
                    " tie every layer to your own structs."
                }
                CodeBlock { source: SHAPE_CODE }
                Text {
                    "The example below uses all four layers. An "
                    Code { source: "EmailField" }
                    " and an "
                    Code { source: "AddressFieldset" }
                    ", used twice, sit in an order form. Submit it empty, then fill it in."
                }
                div { style: "max-width: 360px;", OrderForm {} }
            }
            DocSection { title: "Specialization",
                Text {
                    "Every field is a plain component with the same props, "
                    Code { source: "label" }
                    ", "
                    Code { source: "description" }
                    ", "
                    Code { source: "helper" }
                    ", "
                    Code { source: "status" }
                    ", "
                    Code { source: "validate" }
                    " and "
                    Code { source: "name" }
                    ". A specialized field wraps one and fixes part of its contract. "
                    Code { source: "PasswordField" }
                    " is one, a "
                    Code { source: "TextField" }
                    " with a reveal button. Build your own domain fields the same way, such as an "
                    "email, an IBAN or a phone number. Their rules are built in, so no form repeats them."
                }
                CodeBlock { source: SPECIALIZE_CODE.shown(), language: "rust" }
                Text {
                    "Forward "
                    Code { source: "name" }
                    " as a "
                    Code { source: "FieldName<T>" }
                    " and the specialized field binds to a form like any other. Forward "
                    Code { source: "validate" }
                    " and a caller can still add rules on top of the built-in ones."
                }
            }
            DocSection { title: "Composition",
                Text {
                    "A part of a form that recurs, such as an address, a date range or a contact, "
                    "becomes a component around a "
                    Code { source: "Fieldset" }
                    ". It takes a "
                    Code { source: "path" }
                    " saying where its value sits, names its fields relative to its own type, and "
                    "carries the rules over its own fields. It knows nothing about the form around "
                    "it, so the same part serves the shipping and the billing address."
                }
                CodeBlock { source: COMPOSE_CODE.shown(), language: "rust" }
                Text { "The form then arranges the parts and adds the rules only the whole value can decide." }
                CodeBlock { source: FORM_CODE.shown(), language: "rust" }
            }
            DocSection { title: "Validators",
                Text {
                    "A rule is any function from a value to a "
                    Code { source: "bool" }
                    ", "
                    Code { source: "true" }
                    " when the value is fine. "
                    Code { source: ".error(\"..\")" }
                    " or "
                    Code { source: ".warn(\"..\")" }
                    " gives it a message and turns it into a "
                    Code { source: "Validator<V>" }
                    ". "
                    Code { source: ".and(..)" }
                    " and "
                    Code { source: ".or(..)" }
                    " combine rules before the message. Your own rules work like the built-in ones, "
                    "such as "
                    Code { source: "not_empty" }
                    ", "
                    Code { source: "min_length" }
                    " and "
                    Code { source: "is_email" }
                    ". The "
                    Code { source: "Rule" }
                    " trait has to be in scope."
                }
                CodeBlock { source: RULES_CODE, language: "rust" }
                Text { "The built-in rules cover the common cases, over the value types they make sense for." }
                CodeBlock { source: CATALOG_CODE, language: "rust" }
                Text {
                    "Every layer takes "
                    Code { source: "validate" }
                    ", one validator or an array, typed over the value that layer can see."
                }
                ul {
                    li {
                        Text {
                            "A field checks its own value, as in "
                            Code { source: "TextField {{ validate: not_empty.error(\"..\") }}" }
                            ". Rules that need nothing but that value belong here."
                        }
                    }
                    li {
                        Text {
                            "A specialized field builds in the rules that define it, and still "
                            "accepts more."
                        }
                    }
                    li {
                        Text {
                            "A fieldset checks its fields against each other, such as a city that "
                            "needs a zip. "
                            Code { source: ".on([..])" }
                            " names the fields the problem shows on. Without it, the problem shows "
                            "under the group."
                        }
                    }
                    li {
                        Text {
                            "A form checks what spans parts, such as a billing address unless the "
                            "bill goes to the shipping address. A form rule without "
                            Code { source: ".on([..])" }
                            " shows only in the error summary."
                        }
                    }
                }
                Text {
                    "Put a rule on the lowest layer that sees every value it reads. A field's rules "
                    "show once it loses focus. A composite problem shows once every field it names "
                    "was touched. A submit shows everything, and an error cancels it and focuses a "
                    "summary of every problem. An explicit "
                    Code { source: "status" }
                    ", such as a server's answer, shows at once, and the worse of it and the rules wins."
                }
            }
            DocSection { title: "Fields and paths",
                Text {
                    Code { source: "#[derive(Fields)]" }
                    " gives a struct a "
                    Code { source: "FIELDS" }
                    " constant with one method per field. Each returns a "
                    Code { source: "FieldPath<Root, T>" }
                    ", which knows where a "
                    Code { source: "T" }
                    " sits inside a "
                    Code { source: "Root" }
                    " and how it is spelled. "
                    Code { source: "#[fields(nested)]" }
                    " continues into a field whose type derives "
                    Code { source: "Fields" }
                    " too, and the spelling becomes a dotted name like "
                    Code { source: "shipping.zip" }
                    "."
                }
                CodeBlock { source: PATHS_CODE, language: "rust" }
                Text {
                    "One path is the name the field posts under, the name a composite rule "
                    "addresses and the place the field reads and writes, so the three cannot drift "
                    "apart. A misspelled field is a compile error, not a rule that never shows. "
                    "A path to a "
                    Code { source: "bool" }
                    " on a "
                    Code { source: "TextField" }
                    " is a compile error too, because a field's "
                    Code { source: "name" }
                    " is typed by the value it holds. "
                    Code { source: ".on(..)" }
                    " only takes paths rooted at the rule's own value type."
                }
            }
            DocSection { title: "Binding",
                Text {
                    "A "
                    Code { source: "Form" }
                    " takes your value as a "
                    Code { source: "Store" }
                    ", made with "
                    Code { source: "use_store" }
                    ". A field inside it named by a path reads its value from that store and writes "
                    "every change back, so no field needs a "
                    Code { source: "value" }
                    " and "
                    Code { source: "oninput" }
                    " pair. Typing re-renders only the field that changed, plus whatever "
                    "reads the whole value, such as the form's own rules. A "
                    Code { source: "Fieldset" }
                    " with a "
                    Code { source: "path" }
                    " moves everything inside it one level down, which is what lets a composed part "
                    "use paths rooted at its own type."
                }
                CodeBlock { source: BINDING_CODE, language: "rust" }
                Text {
                    "A field with a handler of its own is controlled by that handler, as outside a "
                    "form. A plain string "
                    Code { source: "name" }
                    " only posts. A path rooted at a different type than the form's value warns in "
                    "debug builds and leaves the field unbound."
                }
            }
        }
    }
}
