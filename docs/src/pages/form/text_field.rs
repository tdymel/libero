use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, disabled_prop, field_controls,
    field_props, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, Text, TextField};

struct UsernameCopy;

impl FieldCopy for UsernameCopy {
    const LABEL: &'static str = "Username";
    const DESCRIPTION: &'static str = "How other people see you.";
    const HELPER: &'static str = "Letters, numbers and underscores.";
    const WARNING: &'static str = "That name is close to another one.";
    const ERROR: &'static str = "That name is taken.";
}

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
                        .doc("Marks the field required and adds an asterisk to the label. Inside a `Form`, an empty one fails the submit."),
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
                ])
                .example("An email field, `TextField { label: \"Email\", helper: \"We never share it.\" }`: a screen reader reads the label, then the helper. An error status marks the input invalid and is read the same way."),
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
                controls: [vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                ], field_controls::<UsernameCopy>(), vec![
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
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<UsernameCopy>(&values);
                    rsx! {
                    TextField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: field.label,
                        aria_label: field.aria_label,
                        description: field.description,
                        helper: field.helper,
                        status: field.status,
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
                }
                },
            }
        }
    }
}
