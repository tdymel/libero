use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
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
                        .doc("The box's color when checked; a theme color name or a literal CSS color."),
                    prop("size", "Size").default("md").doc("The size of the box, and of the label and captions beside it."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the box, independent of size."),
                    prop("checked", "bool")
                        .doc("Pair it with `onchange`. Left out, the box keeps its own state unless a `name` binds it to the form around it."),
                    prop("indeterminate", "bool")
                        .default("false")
                        .doc("Draws the mixed state and reads as `aria-checked=\"mixed\"`. Outranks `checked` visually; toggling from it gives `true`."),
                    prop("onchange", "EventHandler<bool>")
                        .doc("Called with the value `checked` should take next."),
                    prop("label", "Caption")
                        .doc("The caption beside the box. Names the checkbox through a `for`/`id` pair."),
                    prop("description", "Caption")
                        .doc("Under the label: what checking it means."),
                    prop("helper", "Caption")
                        .doc("Under the description, in the same column as the label."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the checkbox required, adds `aria-required` and shows an asterisk in the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the checkbox."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                    prop("aria_label", "String")
                        .doc("Names the checkbox when it has no `label`."),
                    prop("variant", "ChoiceVariant")
                        .default("plain")
                        .doc("`card` draws the checkbox as a bordered surface and makes all of it the hit area. Pair it with a `description`. On the web a link inside the card keeps its own click; natively it toggles the card."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A checkbox, with its label beside the box and the description, helper text "
                    "and validation message under both. It takes the same slots every field has; "
                    "what it does not take is a frame - the box "
                    Code { source: "is" }
                    " the control. The browser never toggles the input itself, so the box, "
                    "the DOM property and the form submission can never disagree with Rust. "
                    Code { source: "checked" }
                    " drives the look when you pass it, paired with "
                    Code { source: "onchange" }
                    "; with neither, and outside a form binding, the box keeps its own state."
                }
            },
            Demo {
                component: "Checkbox",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["plain", "card"])
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
                            _ => vec![],
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
