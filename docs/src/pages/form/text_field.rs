use crate::components::{
    Control, Demo, DemoValues, DocPage, a11y, disabled_prop, prop, props, readonly_prop,
    status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, FieldStatus, Text, TextField};

fn no_trailing(values: &DemoValues) -> bool {
    values.str("trailing") != "true"
}

#[component]
pub fn TextFieldPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        DocPage {
            title: "TextField",
            source: "libero/src/components/form/text_field.rs",
            markdown: "/md/text_field.md",
            properties: vec![
                props("TextField", vec![
                    prop("size", "Size").default("md").doc("Height, padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of `size`."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. Leave it out and the input keeps its own text."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires on every keystroke with the text the field should hold next."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the text, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.email()` also binds the text to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("leading", "Element")
                        .doc("Inside the frame, before the control, such as a search icon or a currency sign."),
                    prop("trailing", "Element")
                        .doc("Inside the frame, after the control, such as a clear button or a unit."),
                    prop("describe_leading", "bool")
                        .default("false")
                        .doc("Set it when `leading` is text that belongs to the value, such as `@`, so a screen reader reads it with the input. Not for an icon or a button."),
                    prop("describe_trailing", "bool")
                        .default("false")
                        .doc("The same for `trailing`, such as `kg` or a `12/20` counter."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the field. Takes a string or an `Element`."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. Formatting rules, limits or a counter."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    disabled_prop("field"),
                    readonly_prop("field"),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Leading, "The slot before the control: an icon, a prefix."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]).extends("input"),
            ],
            accessibility: a11y()
                .handles([
                    "A string `description` or `helper` is read with the input.",
                    "An error status marks the input invalid, a warning does not.",
                ])
                .must([
                    "Leave `label` unset only when something else names the field, such as an `aria_label`.",
                    "Markup in `description` or `helper` is shown but not read, so its accessibility is yours.",
                    "A `leading` or `trailing` slot is not read with the input. When it is text that belongs to the value, such as a unit or a counter, set `describe_leading` or `describe_trailing`.",
                ]),
            lead: rsx! {
                Text {
                    "A single-line text field. Like every field, it stacks a label, a description, "
                    "the control, helper text and a validation message. Pass "
                    Code { source: "value" }
                    " and "
                    Code { source: "oninput" }
                    " to control it, or leave "
                    Code { source: "value" }
                    " out and the input keeps its own text. "
                    Code { source: "leading" }
                    " and "
                    Code { source: "trailing" }
                    " put content inside the border, beside the control."
                }
            },
            // snippet: let mut value = use_signal(String::new);
            Demo {
                component: "TextField",
                children_text: "",
                fixed: vec![
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                ],
                controls: vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That name is close to another one.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"That name is taken.\"".to_string()],
                            _ => vec![],
                        }),
                    // Each caption is a whole element, not a style: unset,
                    // the slot is not in the markup at all.
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Username\"".to_string()],
                            _ => vec!["aria_label: \"Username\"".to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"How other people see you.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Letters, numbers and underscores.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"ada\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("leading").code(|_, values| {
                        match values.str("leading").as_str() {
                            "true" => vec!["leading: rsx! { \"@\" }".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("trailing").code(|_, values| {
                        match values.str("trailing").as_str() {
                            "true" => vec![
                                "trailing: rsx! { \"{value().chars().count()}/20\" }".to_string(),
                                "maxlength: 20".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    // The counter is text a screen reader should hear with the input.
                    Control::switch("describe_trailing").hidden_when(no_trailing),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    TextField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Username".to_string()),
                        aria_label: (values.str("label") != "true").then_some("Username"),
                        description: (values.str("description") == "true")
                            .then(|| "How other people see you.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "Letters, numbers and underscores.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => {
                                FieldStatus::Warning("That name is close to another one.".to_string())
                            }
                            "error" => FieldStatus::Error("That name is taken.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "ada".to_string()),
                        // Plain text: `Icon` colours and sizes an svg, not a prefix.
                        leading: (values.str("leading") == "true").then(|| rsx! { "@" }),
                        trailing: (values.str("trailing") == "true")
                            .then(|| rsx! { "{value().chars().count()}/20" }),
                        maxlength: (values.str("trailing") == "true").then_some(20),
                        describe_trailing: !no_trailing(&values)
                            && values.str("describe_trailing") == "true",
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
