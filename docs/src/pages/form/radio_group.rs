use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, OptionList, Options, RadioGroup, Text};

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

/// What each plan buys, under its label - what makes a card worth drawing.
fn plan_description(plan: Plan) -> String {
    match plan {
        Plan::Free => "Three projects, community support.",
        Plan::Pro => "Unlimited projects, email support.",
        Plan::Team => "Five seats and shared billing.",
    }
    .to_string()
}

// snippet: after PLAN_ENUM
// snippet: let mut plan = use_signal(|| None::<Plan>);
// snippet: in RadioGroup { value: plan(), onchange: move |next| plan.set(Some(next)), .. }
const PLAN_DESCRIPTION: &str = r#"option_description: move |plan: Plan| match plan {
    Plan::Free => "Three projects, community support.",
    Plan::Pro => "Unlimited projects, email support.",
    Plan::Team => "Five seats and shared billing.",
}
.to_string()"#;

/// The flag sits on the option, inside the one `options` prop - a row this
/// group refuses, rather than a plan the type refuses everywhere.
// snippet: after PLAN_ENUM
// snippet: let mut plan = use_signal(|| None::<Plan>);
// snippet: in RadioGroup { value: plan(), onchange: move |next| plan.set(Some(next)), .. }
const DISABLED_OPTION: &str =
    r#"options: OptionList::from_options().disabling(|plan| *plan == Plan::Team)"#;

/// Every plan, with `Team` flagged when the switch is on.
fn plan_options(values: &DemoValues) -> OptionList<Plan> {
    let off = is_on(values, "disabled_option");
    OptionList::from_options().disabling(move |plan| off && *plan == Plan::Team)
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
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here, since only an enum lists its own. A `Vec<T>` converts; an `OptionList<T>` adds per-option `disabled`. Named groups are accepted and drawn flattened - the group is the field, so it draws no headings inside itself."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Overrides what the derive named an option. Runs during render, so it can read a locale from context."),
                    prop("option_description", "Callback<T, String>")
                        .doc("A line under each option's label; an empty string renders none. What makes a card option worth its surface."),
                    prop("variant", "ChoiceVariant")
                        .default("plain")
                        .doc("`card` draws every option as a bordered surface that is its own hit area. A row of cards stretches them to one height."),
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
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. Sets `aria-readonly` on the group, which Chromium does not announce: say it in the label or description where it matters."),
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
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Refuses the pick. ARIA has no read-only radio: `RadioGroup` says it on the group, a lone radio says nothing."),
                    prop("variant", "ChoiceVariant")
                        .default("plain")
                        .doc("`card` draws the radio as a bordered surface and makes all of it the hit area."),
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
            // snippet: let mut plan = use_signal(|| Some(Plan::Pro));
            Demo {
                component: "RadioGroup",
                children_text: "",
                fixed: vec![
                    "value: plan()".to_string(),
                    "onchange: move |next| plan.set(Some(next))".to_string(),
                ],
                controls: vec![
                    // A card only reads as one with a description, so the
                    // card brings the per-option descriptions with it.
                    Control::toggle("variant", ["plain", "card"])
                        .default("plain")
                        .code(|_, values| match values.str("variant").as_str() {
                            "card" => vec![
                                r#"variant: "card""#.to_string(),
                                PLAN_DESCRIPTION.to_string(),
                            ],
                            _ => vec![],
                        }),
                    Control::color("color"),
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
                            // Unlabelled, it still needs a name.
                            _ => vec![r#""aria-label": "Plan""#.to_string()],
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
                    Control::switch("readonly"),
                    // The flag lives inside `options`, so the switch stands
                    // for one named option rather than for a prop of its own.
                    Control::switch("disabled_option").code(|_, values| {
                        match is_on(values, "disabled_option") {
                            true => vec![DISABLED_OPTION.to_string()],
                            false => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    RadioGroup {
                        variant: values.str("variant"),
                        option_description: (values.str("variant") == "card")
                            .then(|| Callback::new(plan_description)),
                        color: values.str("color"),
                        size: values.str("size"),
                        orientation: values.str("orientation"),
                        label: is_on(&values, "label").then(|| "Plan".to_string()),
                        "aria-label": (!is_on(&values, "label")).then_some("Plan"),
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
                        readonly: is_on(&values, "readonly").then_some(true),
                        options: plan_options(&values),
                        value: plan(),
                        onchange: move |next| plan.set(Some(next)),
                    }
                },
                wrap: Wrap(|_values, code| format!("{PLAN_ENUM}{code}")),
            }
        }
    }
}
