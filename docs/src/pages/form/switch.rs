use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Switch, Text};

fn describes(values: &DemoValues) -> bool {
    values.str("description") == "true" || values.str("variant") == "card"
}

#[component]
pub fn SwitchPage() -> Element {
    rsx! {
        DocPage {
            title: "Switch",
            source: "libero/src/components/form/switch.rs",
            markdown: "/md/switch.md",
            properties: vec![props("Switch", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Track color when on. A theme color name or any CSS color."),
                prop("size", "Size").default("md").doc("Size of the track, the thumb and the label."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Track corner radius. The thumb stays a circle."),
                prop("checked", "bool")
                    .doc("Whether it is on. Pair it with `onchange`. Left out, the switch keeps its own state, or the form's when `name` binds it."),
                prop("onchange", "EventHandler<bool>")
                    .doc("Called with the value `checked` should take next."),
                prop("name", "FieldName<bool>")
                    .doc("What the switch posts as. A path such as `Signup::FIELDS.terms()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                prop("validate", "Validators<bool>")
                    .doc("Rules over `checked`, shown once the switch loses focus or its form is submitted."),
                prop("label", "Caption")
                    .doc("The caption beside the track, and the switch's name."),
                prop("description", "Caption")
                    .doc("Under the label. What turning it on does."),
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
                    .doc("Disables and dims the switch."),
                prop("readonly", "bool")
                    .default("false")
                    .doc("Focusable and posted with the form, but not editable. `disabled` drops the switch from the tab order and the post instead. Chromium does not announce read-only on a switch, so say it in the label or description where it matters."),
                prop("aria_label", "String")
                    .doc("Names the switch when it has no `label`."),
                prop("variant", "ChoiceVariant")
                    .default("plain")
                    .doc("`card` draws the switch as a bordered surface you can click anywhere. Pair it with a `description`. On the web a link inside the card keeps its own click. Natively the whole card toggles."),
            ])],
            lead: rsx! {
                Text {
                    "An on/off toggle drawn as a track and thumb. It is a checkbox underneath, "
                    "announced as a switch, with the label beside the track and the "
                    "description, helper and status under both. Pass "
                    Code { source: "checked" }
                    " with "
                    Code { source: "onchange" }
                    " to own the state. With neither, the switch keeps its own, or the form's "
                    "when "
                    Code { source: "name" }
                    " binds it."
                }
            },
            // snippet: let mut enabled = use_signal(|| false);
            Demo {
                component: "Switch",
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
                        .default("xl"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"Uses mobile data.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Turn this on to continue.\"".to_string()],
                            _ => vec![],
                        }),
                    // Controlled state is `checked` + `onchange`; the
                    // library warns about one without the other. The preview
                    // writes `onchange` back into this control, so the
                    // snippet shows the pair a caller writes.
                    Control::switch("checked").default("true").code(|_, _| {
                        vec![
                            "checked: enabled()".to_string(),
                            "onchange: move |next| enabled.set(next)".to_string(),
                        ]
                    }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Notifications\"".to_string()],
                            // Unlabelled, it still needs a name.
                            _ => vec!["aria_label: \"Notifications\"".to_string()],
                        }
                    }),
                    // A card only reads as one with a description under
                    // the label, so the card shows one either way.
                    Control::switch("description").code(|_, values| {
                        match describes(values) {
                            true => vec!["description: \"About once a month.\"".to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"You can turn this off later.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Switch {
                        variant: values.str("variant"),
                        color: values.str("color"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        // Both or neither: `checked` alone can never change.
                        checked: match values.str("checked").as_str() {
                            "true" => Some(true),
                            _ => Some(false),
                        },
                        onchange: {
                            let values = values.clone();
                            EventHandler::new(move |next: bool| values.set("checked", next.to_string()))
                        },
                        label: (values.str("label") == "true")
                            .then(|| "Notifications".to_string()),
                        aria_label: (values.str("label") != "true")
                            .then(|| "Notifications".to_string()),
                        description: describes(&values)
                            .then(|| "About once a month.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "You can turn this off later.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Uses mobile data.".to_string()),
                            "error" => {
                                FieldStatus::Error("Turn this on to continue.".to_string())
                            }
                            _ => FieldStatus::Valid,
                        },
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Space toggles the switch. Outside a "
                    Code { source: "Form" }
                    " Enter toggles it too. Inside one, Enter submits the form, as on a native "
                    "checkbox. Without a visible label, set "
                    Code { source: "aria_label" }
                    ". A debug build warns when the switch has neither."
                }
            }
        }
    }
}
