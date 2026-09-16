use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Text, TextField};

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
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. `None` leaves the `<input>` uncontrolled - it keeps its own text and needs no handler."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke with the text the field should hold next. Native name, native timing."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("leading", "Element")
                        .doc("Inside the frame, before the control - a search icon, a currency prefix."),
                    prop("trailing", "Element")
                        .doc("Inside the frame, after the control - a clear button, a unit."),
                    prop("describe_leading", "bool")
                        .default("false")
                        .doc("`leading` is text that describes the input - a unit, a counter - so it joins the input's `aria-describedby`."),
                    prop("describe_trailing", "bool")
                        .default("false")
                        .doc("The same for `trailing`."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the field through a `for`/`id` pair. Takes a string or an `Element`."),
                    prop("description", "Caption")
                        .doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control: formatting rules, constraints, counters."),
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
                    "A single-line text field with the five slots every field shares: label, "
                    "description, the control, helper text, and a validation message. Pass "
                    Code { source: "value" }
                    " to control it and "
                    Code { source: "oninput" }
                    " to hear about keystrokes; omit "
                    Code { source: "value" }
                    " and the input keeps its own text. Whichever caption slots are filled "
                    "are named by "
                    Code { source: "aria-describedby" }
                    " automatically. "
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
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
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
                            _ => vec![],
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
                            "true" => {
                                vec!["trailing: rsx! { \"{value().len()}/20\" }".to_string()]
                            }
                            _ => vec![],
                        }
                    }),
                    // The counter is text a screen reader should hear with the input.
                    Control::switch("describe_trailing"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    TextField {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Username".to_string()),
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
                        // Plain text, not an `Icon` - `Icon` colours and sizes an
                        // svg, which is not what a prefix is.
                        leading: (values.str("leading") == "true").then(|| rsx! { "@" }),
                        trailing: (values.str("trailing") == "true")
                            .then(|| rsx! { "{value().len()}/20" }),
                        describe_trailing: values.str("describe_trailing") == "true",
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
