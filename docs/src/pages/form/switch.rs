use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, disabled_prop, field_controls,
    field_props, prop, props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::SwitchPart;
use libero::components::{Code, Switch, Text};
use libero::use_theme;

struct NotificationsCopy;

impl FieldCopy for NotificationsCopy {
    const LABEL: &'static str = "Notifications";
    const DESCRIPTION: &'static str = "About once a month.";
    const HELPER: &'static str = "You can turn this off later.";
    const WARNING: &'static str = "Uses mobile data.";
    const ERROR: &'static str = "Turn this on to continue.";
}

fn describes(values: &DemoValues) -> bool {
    values.str("description") == "true" || values.str("variant") == "card"
}

#[component]
pub fn SwitchPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Switch",
            source: "libero/src/components/form/switch.rs",
            markdown: "/md/switch.md",
            properties: vec![props("Switch", vec![
                prop("color", "ThemeAwareValue")
                    .default("primary")
                    .doc("Track color when on. A theme color name or any CSS color."),
                prop("size", "Size")
                    .default(theme.switch.size.as_str())
                    .doc("Size of the track, the thumb and the label."),
                prop("radius", "Size")
                    .default(theme.switch.radius.as_str())
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
                status_prop(),
                required_prop().also("Inside a `Form`, an empty one fails the submit."),
                disabled_prop("switch"),
                readonly_prop("switch").also("Chromium does not announce read-only on a switch, so say it in the label or description where it matters."),
                prop("aria_label", "String")
                    .doc("Names the switch when it has no `label`."),
                prop("variant", "ChoiceVariant")
                    .default(theme.switch.variant.as_str())
                    .doc("`card` draws the switch as a bordered surface you can click anywhere. Pair it with a `description`. On the web a link inside the card keeps its own click. Natively the whole card toggles."),
            ])
            .parts("SwitchPart", vec![
                (SwitchPart::Label, "The label beside the control."),
                (SwitchPart::Required, "The required asterisk, in the label."),
                (SwitchPart::Description, "The caption between the label and the control."),
                (SwitchPart::Control, "Holds the hidden input and the track, beside the label."),
                (SwitchPart::Track, "The pill the thumb slides along."),
                (SwitchPart::Thumb, "The sliding knob."),
                (SwitchPart::Helper, "The caption under the control."),
                (SwitchPart::Status, "The validation message."),
            ])],
            accessibility: a11y()
                .key(["Space"], "Toggles the switch.")
                .key(["Enter"], "Outside a `Form`: toggles the switch. Inside one: submits the form, as on a native checkbox.")
                .handles(["A debug build warns when the switch has neither a visible label nor `aria_label`."])
                .must(["Without a visible label, set `aria_label`."])
                .example("A notifications switch in a settings `Form`, `Switch { label: \"Notifications\" }`: Space turns it on or off, Enter submits the form, and the label is its name."),
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
                controls: [vec![
                    Control::toggle("variant", ["plain", "card"])
                        .labels(["Plain", "Card"])
                        .default("plain")
                        .code(|_, values| match values.str("variant").as_str() {
                            // A card only reads as one with a description under
                            // the label, so the card shows one either way.
                            "card" if values.str("description") != "true" => vec![
                                r#"variant: "card""#.to_string(),
                                format!("description: {:?}", NotificationsCopy::DESCRIPTION),
                            ],
                            "card" => vec![r#"variant: "card""#.to_string()],
                            _ => vec![],
                        }),
                    Control::color("color"),
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("xl"),
                ], field_controls::<NotificationsCopy>(), vec![
                    // `checked` + `onchange` as a pair (the library warns on one alone); the
                    // preview writes `onchange` back into this control.
                    Control::switch("checked").default("true").code(|_, _| {
                        vec![
                            "checked: enabled()".to_string(),
                            "onchange: move |next| enabled.set(next)".to_string(),
                        ]
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<NotificationsCopy>(&values);
                    rsx! {
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
                            label: field.label,
                            aria_label: field.aria_label.map(String::from),
                            description: describes(&values).then(|| NotificationsCopy::DESCRIPTION.to_string()),
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
