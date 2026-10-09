use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, disabled_prop, field_controls,
    field_props, prop, props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::CheckboxPart;
use libero::components::{Checkbox, Code, Text};
use libero::use_theme;

struct TermsCopy;

impl FieldCopy for TermsCopy {
    const LABEL: &'static str = "Accept the terms";
    const DESCRIPTION: &'static str = "The licence and the privacy policy.";
    const HELPER: &'static str = "You can withdraw consent at any time.";
    const WARNING: &'static str = "You can change this later.";
    const ERROR: &'static str = "Accept the terms to continue.";
}

fn describes(values: &DemoValues) -> bool {
    values.str("description") == "true" || values.str("variant") == "card"
}

#[component]
pub fn CheckboxPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Checkbox",
            source: "libero/src/components/form/checkbox.rs",
            markdown: "/md/checkbox.md",
            properties: vec![
                props("Checkbox", vec![
                    prop("color", "ThemeAwareValue")
                        .default(theme.checkbox.color.as_str())
                        .doc("The box's color when checked. A theme color name or any CSS color; `theme.checkbox.color` when unset."),
                    prop("size", "Size")
                        .default(theme.checkbox.size.as_str())
                        .doc("Size of the box, the label and the captions."),
                    prop("radius", "Size")
                        .default(theme.checkbox.radius.as_str())
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
                    status_prop(),
                    required_prop().also("Inside a `Form`, an empty one fails the submit."),
                    disabled_prop("checkbox"),
                    readonly_prop("checkbox").also("Chromium does not announce read-only on a checkbox, so say it in the label or description where it matters."),
                    prop("aria_label", "String")
                        .doc("Names the checkbox when it has no `label`."),
                    prop("variant", "ChoiceVariant")
                        .default(theme.checkbox.variant.as_str())
                        .doc("`card` draws the checkbox as a bordered surface you can click anywhere. Pair it with a `description`. On the web a link inside the card keeps its own click. Natively the whole card toggles."),
                ])
                .parts("CheckboxPart", vec![
                    (CheckboxPart::Label, "The label beside the control."),
                    (CheckboxPart::Required, "The required asterisk, in the label."),
                    (CheckboxPart::Description, "The caption between the label and the control."),
                    (CheckboxPart::Control, "Holds the hidden input and the box, beside the label."),
                    (CheckboxPart::Box, "The drawn square and its mark."),
                    (CheckboxPart::Helper, "The caption under the control."),
                    (CheckboxPart::Status, "The validation message."),
                ]),
            ],
            accessibility: a11y()
                .key(["Space"], "Toggles the checkbox.")
                .must(["Without a visible label, set `aria_label`."])
                .example("A terms checkbox, `Checkbox { label: \"Accept the terms\" }`: Tab lands on the box, the label is its name, and Space ticks or clears it."),
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
            // snippet: let mut ticked = use_signal(|| true);
            Demo {
                component: "Checkbox",
                children_text: "",
                controls: [vec![
                    Control::toggle("variant", ["plain", "card"])
                        .labels(["Plain", "Card"])
                        .default(theme.checkbox.variant.as_str())
                        .code(|_, values| match values.str("variant").as_str() {
                            // A card only reads as one with a description under
                            // the label, so the card shows one either way.
                            "card" if values.str("description") != "true" => vec![
                                r#"variant: "card""#.to_string(),
                                format!("description: {:?}", TermsCopy::DESCRIPTION),
                            ],
                            "card" => vec![r#"variant: "card""#.to_string()],
                            _ => vec![],
                        }),
                    Control::color("color").default(theme.checkbox.color.as_str()),
                    Control::sizes("size")
                        .default(theme.checkbox.size.as_str()),
                    Control::sizes("radius")
                        .default(theme.checkbox.radius.as_str()),
                ], field_controls::<TermsCopy>(), vec![
                    // `checked` + `onchange` as a pair (the library warns on one alone); the
                    // preview writes `onchange` back into this control.
                    Control::switch("checked").default("true").code(|_, _| {
                        vec![
                            "checked: ticked()".to_string(),
                            "onchange: move |next| ticked.set(next)".to_string(),
                        ]
                    }),
                    Control::switch("indeterminate"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<TermsCopy>(&values);
                    rsx! {
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
                            label: field.label,
                            aria_label: field.aria_label.map(String::from),
                            description: describes(&values).then(|| TermsCopy::DESCRIPTION.to_string()),
                            helper: field.helper,
                            status: field.status,
                            required: (values.str("required") == "true").then_some(true),
                            disabled: (values.str("disabled") == "true").then_some(true),
                        }
                    }
                },
            }
        }
    }
}
