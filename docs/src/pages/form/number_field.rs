use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, FieldStatus, NumberField, NumberValue, Text};
use libero::use_theme;

/// Printed above the snippet when the demo is on the custom type - the field
/// is only as short as it is because the impl exists.
// snippet: ignore - pseudocode bodies
const CENTS_IMPL: &str = r#"// Money is not an f64. `Cents` stores whole cents and shows a decimal
// point, which is the one method a custom `NumberValue` has to write.
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct Cents(i64);

impl Add for Cents { /* self.0.saturating_add(other.0) */ }
impl Sub for Cents { /* self.0.saturating_sub(other.0) */ }
impl FromStr for Cents { /* an f64 that is_finite(), times 100, rounded */ }
impl Display for Cents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let cents = self.0.unsigned_abs();
        write!(f, "{sign}{}.{:02}", cents / 100, cents % 100)
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
        Cents(self.0.saturating_add(other.0))
    }
}

impl std::ops::Sub for Cents {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Cents(self.0.saturating_sub(other.0))
    }
}

impl std::str::FromStr for Cents {
    type Err = &'static str;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let number = text.parse::<f64>().map_err(|_| "not a number")?;
        // `f64` parses "nan" and "inf" too.
        if !number.is_finite() {
            return Err("not a finite number");
        }
        Ok(Cents((number * 100.0).round() as i64))
    }
}

impl std::fmt::Display for Cents {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sign = if self.0 < 0 { "-" } else { "" };
        let cents = self.0.unsigned_abs();
        write!(formatter, "{sign}{}.{:02}", cents / 100, cents % 100)
    }
}

impl NumberValue for Cents {
    fn default_step() -> Self {
        Cents(50)
    }
}

#[component]
pub fn NumberFieldPage() -> Element {
    let theme = use_theme();
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
                    prop("size", "Size").default(theme.number_field.size.as_str()).doc("Height, padding and font size."),
                    prop("radius", "Size")
                        .default(theme.number_field.radius.as_str())
                        .doc("Corner radius, independent of `size`."),
                    prop("value", "Option<T>")
                        .doc("The number in the field, strictly controlled. `None` is the empty field. A signal that starts at `None` needs its type, such as `None::<i32>`."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the number the caller should hold next, `None` once the field is emptied. Half-typed text such as `-` or `1.` never reaches it. Leaving the field clamps a number out of range and reverts text that never parsed."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the number, shown once the field loses focus or its form is submitted."),
                    prop("min", "Option<T>")
                        .doc("Floor. Steps clamp to it. Typed text below it clamps once the field is left or Enter is pressed."),
                    prop("max", "Option<T>")
                        .doc("Ceiling, the same way."),
                    prop("step", "Option<T>")
                        .default("T::default_step()")
                        .doc("What one step moves by, `1` for an integer, `1.0` for a float."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.age()` also binds the number to the surrounding `Form`'s value when the field has no `onchange`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("steppers", "bool")
                        .default("false")
                        .doc("Shows minus and plus buttons in the trailing slot. The arrow keys step the value either way."),
                    prop("increment_label", "String")
                        .default("number_field.increase")
                        .doc("Names the plus button, such as \"Add a guest\". Unset, the localization's `number_field.increase`, \"Increase\" in English."),
                    prop("decrement_label", "String")
                        .default("number_field.decrease")
                        .doc("Names the minus button. Unset, the localization's `number_field.decrease`, \"Decrease\" in English."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the field."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. Units, ranges, what the number means."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]).extends("input"),
                props("NumberValue", vec![
                    prop("default_step", "fn() -> Self")
                        .default("required")
                        .doc("What one step moves by when `step` is unset."),
                    prop("parse", "fn(&str) -> Option<Self>")
                        .default("FromStr")
                        .doc("`None` while the text is not a number yet. `f64`'s `FromStr` also reads `nan` and `inf`, so a type built on it should refuse them."),
                    prop("format", "fn(&self) -> String")
                        .default("Display")
                        .doc("The text the field shows for a value."),
                    prop("zero", "fn() -> Option<Self>")
                        .default("parse(\"0\")")
                        .doc("Where a step starts in an empty field. `None` makes the step do nothing."),
                    prop("step_up", "fn(self, Self) -> Self")
                        .default("Add")
                        .doc("One step up. Override it for a wrapping angle or a logarithmic step."),
                    prop("step_down", "fn(self, Self) -> Self")
                        .default("Sub")
                        .doc("One step down."),
                    prop("clamp_between", "fn(self, Option<Self>, Option<Self>) -> Self")
                        .default("PartialOrd")
                        .doc("The value pulled into the field's range."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Up", "Down"], "Step the value, with or without `steppers`.")
                .key(["PageUp", "PageDown"], "Step the value ten steps.")
                .handles([
                    "The stepper buttons are not tab stops, since the arrow keys do the same from the field.",
                    "The phone keypad follows the type and `min`: digits for a whole number from 0, digits and a point for a fraction from 0, the full keyboard where a negative is allowed, since iOS keypads have no minus. An `inputmode` attribute overrides it.",
                ])
                .must([
                    "Leave `label` unset only when something else names the field.",
                    "With `steppers` on several fields of one form, name the buttons by their field with `increment_label` and `decrement_label`, such as \"Add a guest\". Unset, every field's are \"Increase\" and \"Decrease\".",
                ]),
            lead: rsx! {
                Text {
                    "A numeric field over your own number type, with optional steppers. Every "
                    "primitive number implements "
                    Code { source: "NumberValue" }
                    ", so "
                    Code { source: "T" }
                    " is inferred from "
                    Code { source: "value" }
                    ". A type of your own implements "
                    Code { source: "default_step" }
                    " and gets the rest from "
                    Code { source: "FromStr" }
                    " and "
                    Code { source: "Display" }
                    ", as the Cents case shows. A float field writes and reads the decimal "
                    "separator of the provider's "
                    Code { source: "Formats" }
                    ", so "
                    Code { source: "1,5" }
                    " under "
                    Code { source: "Formats::GERMAN" }
                    "."
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
                        .labels(["Valid", "Warning", "Error"])
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
                    // On in the preview, but the prop defaults to false, so the
                    // snippet prints it whenever it is on.
                    Control::switch("steppers").default("true").code(|_, values| {
                        match values.str("steppers").as_str() {
                            "true" => vec!["steppers: true".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("increment_label")
                        .hidden_when(|values| values.str("steppers") != "true")
                        .code(|_, values| match values.str("increment_label").as_str() {
                            "true" => vec![
                                "increment_label: \"Add one\"".to_string(),
                                "decrement_label: \"Remove one\"".to_string(),
                            ],
                            _ => vec![],
                        }),
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
                    let named = values.str("increment_label") == "true";
                    let increment_label = named.then(|| "Add one".to_string());
                    let decrement_label = named.then(|| "Remove one".to_string());

                    match values.str("type").as_str() {
                        "f64" => rsx! {
                            NumberField {
                                size, radius, label, aria_label, description, helper, status,
                                required, disabled, steppers, increment_label, decrement_label,
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
                                required, disabled, steppers, increment_label, decrement_label,
                                min: ranged.then_some(Cents(0)),
                                max: ranged.then_some(Cents(10_000)),
                                value: price(),
                                onchange: move |next| price.set(next),
                            }
                        },
                        _ => rsx! {
                            NumberField {
                                size, radius, label, aria_label, description, helper, status,
                                required, disabled, steppers, increment_label, decrement_label,
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
