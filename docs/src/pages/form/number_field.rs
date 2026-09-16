use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, NumberField, NumberValue, Text};

/// Printed above the snippet when the demo is on the custom type - the field
/// is only as short as it is because the impl exists.
// snippet: ignore - pseudocode bodies
const CENTS_IMPL: &str = r#"// Money is not an f64. `Cents` stores whole cents and shows a decimal
// point, which is the one method a custom `NumberValue` has to write.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Cents(i64);

impl Add for Cents { /* self.0 + other.0 */ }
impl Sub for Cents { /* self.0 - other.0 */ }
impl FromStr for Cents { /* (text.parse::<f64>()? * 100.0).round() as i64 */ }
impl Display for Cents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl NumberValue for Cents {
    fn default_step() -> Self { Cents(50) }
}

"#;

/// The demo's custom type. `parse` and `format` come from `FromStr`/`Display`,
/// so the impl is one method.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Cents(i64);

impl std::ops::Add for Cents {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Cents(self.0 + other.0)
    }
}

impl std::ops::Sub for Cents {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Cents(self.0 - other.0)
    }
}

impl std::str::FromStr for Cents {
    type Err = std::num::ParseFloatError;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Ok(Cents((text.parse::<f64>()? * 100.0).round() as i64))
    }
}

impl std::fmt::Display for Cents {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{:02}", self.0 / 100, self.0 % 100)
    }
}

impl NumberValue for Cents {
    fn default_step() -> Self {
        Cents(50)
    }
}

#[component]
pub fn NumberFieldPage() -> Element {
    let mut quantity = use_signal(|| Some(4i32));
    let mut weight = use_signal(|| Some(1.5f64));
    let mut price = use_signal(|| Some(Cents(1234)));

    rsx! {
        DocPage {
            title: "NumberField",
            source: "libero/src/components/form/number_field.rs",
            markdown: "/md/number_field.md",
            properties: vec![
                props("NumberField", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<T>")
                        .doc("The number in the field; strictly controlled. `None` is the empty field."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the number the caller should hold next; `None` once the field is emptied. Silent while the text is not yet a number, so `-` and `1.` never reach it. Leaving the field puts back the value's own text: out of range clamps, text that never parsed reverts."),
                    prop("min", "Option<T>")
                        .doc("Floor. Steps clamp to it; typed text below it clamps once the field is left or Enter is pressed."),
                    prop("max", "Option<T>")
                        .doc("Ceiling, same."),
                    prop("step", "Option<T>")
                        .default("T::default_step()")
                        .doc("What one press of a stepper moves by; `1` for an integer, `1.0` for a float."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("steppers", "bool")
                        .default("false")
                        .doc("Shows the minus/plus buttons in the trailing slot. Off by default: a number is usually typed, and the arrow keys step it either way."),
                    prop("increment_label", "String")
                        .default("number_field.increase")
                        .doc("Announced on the stepper that raises the value, e.g. \"Add a guest\". Unset, the localization's `number_field.increase` - \"Increase\" in English."),
                    prop("decrement_label", "String")
                        .default("number_field.decrease")
                        .doc("Announced on the stepper that lowers it. Unset, the localization's `number_field.decrease` - \"Decrease\" in English."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the field through a `for`/`id` pair."),
                    prop("description", "Caption")
                        .doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control: units, ranges, what the number means."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required, adds `aria-required` and shows an asterisk in the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the field."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A numeric field over the caller's own number type, with optional steppers in its "
                    "trailing slot. Every primitive number implements "
                    Code { source: "NumberValue" }
                    ", so "
                    Code { source: "T" }
                    " is inferred from "
                    Code { source: "value" }
                    " and there is nothing to write. The control keeps the raw text in an edit "
                    "buffer, so "
                    Code { source: "-" }
                    " and "
                    Code { source: "1." }
                    " survive being typed and only a value your type could parse reaches "
                    Code { source: "onchange" }
                    ". Arrow Up and Arrow Down always step it, Page Up and Page Down ten steps; "
                    Code { source: "steppers" }
                    " adds the buttons."
                }
            },
            // snippet: let mut quantity = use_signal(|| Some(4i32));
            // snippet: let mut weight = use_signal(|| Some(1.5f64));
            Demo {
                component: "NumberField",
                children_text: "",
                // The custom type's impl is what makes the `Cents` case short,
                // so the snippet is a lie without it.
                wrap: Wrap(|values: &DemoValues, source: &str| {
                    match values.str("type").as_str() {
                        "cents" => format!("{CENTS_IMPL}{source}"),
                        _ => source.to_string(),
                    }
                }),
                controls: vec![
                    // Three value types over one component: a plain integer, a
                    // float with its own step, and a caller's own type.
                    Control::toggle("type", ["i32", "f64", "cents"])
                        .default("i32")
                        .labels(["i32", "f64", "Cents (custom)"])
                        .code(|_, values| match values.str("type").as_str() {
                            "f64" => vec![
                                "step: 0.5".to_string(),
                                "value: weight()".to_string(),
                                "onchange: move |next| weight.set(next)".to_string(),
                            ],
                            "cents" => vec![
                                "value: price()".to_string(),
                                "onchange: move |next| price.set(next)".to_string(),
                            ],
                            _ => vec![
                                "value: quantity()".to_string(),
                                "onchange: move |next| quantity.set(next)".to_string(),
                            ],
                        }),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That is a lot.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Not in stock.\"".to_string()],
                            _ => vec![],
                        }),
                    // Bounds are typed as `T`, so what the switch prints
                    // follows the value type the same way the props do.
                    Control::switch("steppers").default("true"),
                    Control::switch("range").code(|_, values| {
                        match (values.str("range").as_str(), values.str("type").as_str()) {
                            ("true", "f64") => {
                                vec!["min: 0.0".to_string(), "max: 10.0".to_string()]
                            }
                            ("true", "cents") => vec![
                                "min: Cents(0)".to_string(),
                                "max: Cents(10_000)".to_string(),
                            ],
                            ("true", _) => vec!["min: 1".to_string(), "max: 99".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Quantity\"".to_string()],
                            _ => vec!["aria_label: \"Quantity\"".to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"How many to add to the order.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Up to 99 per order.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| {
                    let label = (values.str("label") == "true").then(|| "Quantity".to_string());
                    let aria_label = label.is_none().then_some("Quantity");
                    let description = (values.str("description") == "true")
                        .then(|| "How many to add to the order.".to_string());
                    let helper =
                        (values.str("helper") == "true").then(|| "Up to 99 per order.".to_string());
                    let status = match values.str("status").as_str() {
                        "warning" => FieldStatus::Warning("That is a lot.".to_string()),
                        "error" => FieldStatus::Error("Not in stock.".to_string()),
                        _ => FieldStatus::Valid,
                    };
                    let size = values.str("size");
                    let radius = values.str("radius");
                    let required = (values.str("required") == "true").then_some(true);
                    let disabled = (values.str("disabled") == "true").then_some(true);
                    let ranged = values.str("range") == "true";
                    let steppers = values.str("steppers") == "true";

                    match values.str("type").as_str() {
                        "f64" => rsx! {
                            NumberField {
                                size, radius, label, aria_label, description, helper, status,
                                required, disabled, steppers,
                                step: 0.5f64,
                                min: ranged.then_some(0.0f64),
                                max: ranged.then_some(10.0f64),
                                value: weight(),
                                onchange: move |next| weight.set(next),
                            }
                        },
                        "cents" => rsx! {
                            NumberField {
                                size, radius, label, aria_label, description, helper, status,
                                required, disabled, steppers,
                                min: ranged.then_some(Cents(0)),
                                max: ranged.then_some(Cents(10_000)),
                                value: price(),
                                onchange: move |next| price.set(next),
                            }
                        },
                        _ => rsx! {
                            NumberField {
                                size, radius, label, aria_label, description, helper, status,
                                required, disabled, steppers,
                                min: ranged.then_some(1i32),
                                max: ranged.then_some(99i32),
                                value: quantity(),
                                onchange: move |next| quantity.set(next),
                            }
                        },
                    }
                },
            }
        }
    }
}
