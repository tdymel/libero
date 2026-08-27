use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Options, RadioGroup, Text};

/// The enum is the option list, so the snippet has to show it.
const PLAN_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    #[option(label = "Team of 5")]
    Team,
}

"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Plan {
    Free,
    Pro,
    #[option(label = "Team of 5")]
    Team,
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

#[component]
pub fn RadioGroupPage() -> Element {
    let mut plan = use_signal(|| Some(Plan::Pro));

    rsx! {
        DocPage {
            title: "RadioGroup",
            source: "libero/src/components/form/radio_group.rs",
            markdown: "/md/radio_group.md",
            properties: vec![
                props("RadioGroup", vec![
                    prop("value", "Option<T>")
                        .doc("The selected option; strictly controlled. `None` selects nothing, which is what an unanswered question looks like."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the option the caller should select next."),
                    prop("options", "Vec<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here, since only an enum lists its own."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Overrides what the derive named an option. Runs during render, so it can read a locale from context."),
                    prop("orientation", "Orientation")
                        .default("vertical")
                        .doc("Lays the options out in a row instead of a column. A form stacks; a row is for two or three short options."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("The ring and dot color of the selected option."),
                    prop("size", "Size").default("md").doc("The size of every circle, and of the labels beside them."),
                    prop("label", "Caption")
                        .doc("The question. Names the group through `aria-labelledby`, since `for` cannot name a `role=\"radiogroup\"`."),
                    prop("description", "Caption")
                        .doc("Between the question and the options: how to choose."),
                    prop("helper", "Caption")
                        .doc("Under the options. Consequences of the choice."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Adds `aria-required` to the group and marks the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every option and dims the group."),
                ]),
                props("Radio", vec![
                    prop("checked", "bool")
                        .doc("Strictly controlled - pair it with `onselect`."),
                    prop("onselect", "EventHandler<()>")
                        .doc("Fires when this radio is picked. Never fires to unpick one: a radio is turned off by another in its group being turned on."),
                    prop("name", "String")
                        .doc("Shared by every radio in one group, which is what makes the native control exclusive. `RadioGroup` sets it."),
                    prop("tabindex", "String")
                        .doc("`RadioGroup` makes exactly one radio the group's tab stop and takes the rest out of the tab order."),
                    prop("aria_label", "String")
                        .doc("Names the radio when it has no `label`."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A group of radios over an enum, exactly one of them selected. The group is "
                    "the field: it owns the question's label and captions, the "
                    Code { source: "name" }
                    " that makes the set exclusive, and the single tab stop the ARIA pattern "
                    "asks for. Arrow keys move through the options and select as they go, "
                    "wrapping at the ends."
                }
                Text {
                    "Reach for a "
                    Code { source: "Radio" }
                    " on its own only to lay a group out by hand - alone it owns none of those "
                    "three things."
                }
            },
            Demo {
                component: "RadioGroup",
                children_text: "",
                fixed: vec![
                    "value: plan()".to_string(),
                    "onchange: move |next| plan.set(Some(next))".to_string(),
                ],
                controls: vec![
                    Control::color(
                        "color",
                        ["primary", "secondary", "success", "error", "warning", "info"],
                    ),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::toggle("orientation", ["vertical", "horizontal"])
                        .default("vertical")
                        .code(|_, values| match values.str("orientation").as_str() {
                            "horizontal" => vec![r#"orientation: "horizontal""#.to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("Billing starts today.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "Pick a plan to continue.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec![r#"label: "Plan""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec![r#"description: "What your seats cost.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec![r#"helper: "You can change it later.""#.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    RadioGroup {
                        color: values.str("color"),
                        size: values.str("size"),
                        orientation: values.str("orientation"),
                        label: is_on(&values, "label").then(|| "Plan".to_string()),
                        description: is_on(&values, "description")
                            .then(|| "What your seats cost.".to_string()),
                        helper: is_on(&values, "helper")
                            .then(|| "You can change it later.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Billing starts today.".to_string()),
                            "error" => FieldStatus::Error("Pick a plan to continue.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        required: is_on(&values, "required").then_some(true),
                        disabled: is_on(&values, "disabled").then_some(true),
                        value: plan(),
                        onchange: move |next| plan.set(Some(next)),
                    }
                },
                wrap: Wrap(|_values, code| format!("{PLAN_ENUM}{code}")),
            }
        }
    }
}
