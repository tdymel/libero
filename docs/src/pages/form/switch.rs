use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Switch, Text};

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
                    .doc("Track color when checked; a theme color name or a literal CSS color."),
                prop("size", "Size").default("md").doc("Controls track and thumb size, and the label beside them."),
                prop("radius", "Size")
                    .default("xl")
                    .doc("Track corner radius; the thumb is always a circle."),
                prop("checked", "bool")
                    .doc("Strictly controlled - pair it with `onchange`."),
                prop("onchange", "EventHandler<bool>")
                    .doc("Called with the value `checked` should take next."),
                prop("label", "Caption")
                    .doc("The caption beside the track. Names the switch through a `for`/`id` pair."),
                prop("description", "Caption")
                    .doc("Under the label: what turning it on does."),
                prop("helper", "Caption")
                    .doc("Under the description, in the label's column."),
                prop("status", "FieldStatus")
                    .default("Valid")
                    .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                prop("required", "bool")
                    .default("false")
                    .doc("Marks the switch required, adds `aria-required` and shows an asterisk in the label."),
                prop("disabled", "bool")
                    .default("false")
                    .doc("Disables interaction and dims the switch."),
                prop("aria_label", "String")
                    .doc("Names the switch when it has no `label`."),
            ])],
            lead: rsx! {
                Text {
                    "A checkbox styled as a track and thumb. A visually hidden "
                    Code { source: "input" }
                    " does the real work, so it is announced as a switch, and Space and Enter "
                    "both toggle it. It takes the same slots every field takes, with the track "
                    "where a checkbox puts its box. Strictly controlled: "
                    Code { source: "checked" }
                    " drives the look and "
                    Code { source: "onchange" }
                    " reports the value it should take next, so the track, the DOM property and "
                    "the form submission can never disagree with Rust."
                }
            },
            Demo {
                component: "Switch",
                children_text: "",
                controls: vec![
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
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"About once a month.\"".to_string()],
                            _ => vec![],
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
                        description: (values.str("description") == "true")
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
        }
    }
}
