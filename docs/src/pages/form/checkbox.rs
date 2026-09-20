use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::{Checkbox, Code, FieldStatus, Text};

fn describes(values: &DemoValues) -> bool {
    values.str("description") == "true" || values.str("variant") == "card"
}

#[component]
pub fn CheckboxPage() -> Element {
    rsx! {
        DocPage {
            title: "Checkbox",
            source: "libero/src/components/form/checkbox.rs",
            markdown: "/md/checkbox.md",
            properties: vec![
                props("Checkbox", vec![
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The box's color when checked. A theme color name or any CSS color."),
                    prop("size", "Size").default("md").doc("Size of the box, the label and the captions."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the box."),
                    prop("checked", "bool")
                        .doc("Whether it is checked. Pair it with `onchange`. Left out, the box keeps its own state, or the form's when `name` binds it."),
                    prop("indeterminate", "bool")
                        .default("false")
                        .doc("Draws a dash and reads as mixed. It wins over `checked`, and toggling from it gives `true`."),
                    prop("onchange", "EventHandler<bool>")
                        .doc("Called with the value `checked` should take next."),
                    prop("name", "FieldName<bool>")
                        .doc("What the checkbox posts as. A path such as `Signup::FIELDS.terms()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<bool>")
                        .doc("Rules over `checked`, shown once the checkbox loses focus or its form is submitted."),
                    prop("label", "Caption")
                        .doc("The caption beside the box, and the checkbox's name."),
                    prop("description", "Caption")
                        .doc("Under the label. What checking it means."),
                    prop("helper", "Caption")
                        .doc("Under the description, in the label's column."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Sets `aria-required` and marks the label with an asterisk."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the checkbox."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` drops the checkbox from the tab order and the post instead. Chromium does not announce read-only on a checkbox, so say it in the label or description where it matters."),
                    prop("aria_label", "String")
                        .doc("Names the checkbox when it has no `label`."),
                    prop("variant", "ChoiceVariant")
                        .default("plain")
                        .doc("`card` draws the checkbox as a bordered surface you can click anywhere. Pair it with a `description`. On the web a link inside the card keeps its own click. Natively the whole card toggles."),
                ]),
            ],
            accessibility: a11y()
                .key(["Space"], "Toggles the checkbox.")
                .must(["Without a visible label, set `aria_label`."]),
            lead: rsx! {
                Text {
                    "A checkbox with its label beside the box and the description, helper and "
                    "status under the label. It has no frame, since the box is the control. Pass "
                    Code { source: "checked" }
                    " with "
                    Code { source: "onchange" }
                    " to own the state. With neither, the box keeps its own, or the form's when "
                    Code { source: "name" }
                    " binds it."
                }
            },
            Demo {
                component: "Checkbox",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["plain", "card"])
                        .labels(["Plain", "Card"])
                        .default("plain")
                        .code(|_, values| match values.str("variant").as_str() {
                            "card" => vec![r#"variant: "card""#.to_string()],
                            _ => vec![],
                        }),
                    Control::color("color"),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"You can change this later.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Accept the terms to continue.\"".to_string()],
                            _ => vec![],
                        }),
                    // Controlled state is `checked` + `onchange`; the
                    // library warns about one without the other.
                    Control::switch("checked").default("true").code(|_, values| {
                        match values.str("checked").as_str() {
                            "true" => vec![
                                "checked: true".to_string(),
                                "onchange: move |_| {}".to_string(),
                            ],
                            _ => vec!["onchange: move |_| {}".to_string()],
                        }
                    }),
                    Control::switch("indeterminate"),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Accept the terms\"".to_string()],
                            _ => vec!["aria_label: \"Accept the terms\"".to_string()],
                        }
                    }),
                    // A card only reads as one with a description under
                    // the label, so the card shows one either way.
                    Control::switch("description").code(|_, values| {
                        match describes(values) {
                            true => vec![
                                "description: \"The licence and the privacy policy.\"".to_string(),
                            ],
                            false => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"You can withdraw consent at any time.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Checkbox {
                        variant: values.str("variant"),
                        color: values.str("color"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        // Both or neither: `checked` alone can never change.
                        checked: match values.str("checked").as_str() {
                            "true" => Some(true),
                            _ => Some(false),
                        },
                        // The preview writes `onchange` back into the switches,
                        // so the box can be ticked; a mixed box turns `true`.
                        onchange: {
                            let values = values.clone();
                            EventHandler::new(move |next: bool| {
                                values.set("checked", next.to_string());
                                values.set("indeterminate", "false");
                            })
                        },
                        indeterminate: (values.str("indeterminate") == "true").then_some(true),
                        label: (values.str("label") == "true")
                            .then(|| "Accept the terms".to_string()),
                        aria_label: (values.str("label") != "true")
                            .then(|| "Accept the terms".to_string()),
                        description: describes(&values)
                            .then(|| "The licence and the privacy policy.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "You can withdraw consent at any time.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => {
                                FieldStatus::Warning("You can change this later.".to_string())
                            }
                            "error" => {
                                FieldStatus::Error("Accept the terms to continue.".to_string())
                            }
                            _ => FieldStatus::Valid,
                        },
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                    }
                },
            }
        }
    }
}
