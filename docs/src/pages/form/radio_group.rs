use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, FieldCopy, Wrap, a11y, field_controls,
    field_props, prop, props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::{Code, OptionList, Options, RadioGroup, Text};
use libero::components::{RadioGroupPart, RadioPart};

struct PlanCopy;

impl FieldCopy for PlanCopy {
    const LABEL: &'static str = "Plan";
    const DESCRIPTION: &'static str = "What your seats cost.";
    const HELPER: &'static str = "You can change it later.";
    const WARNING: &'static str = "Billing starts today.";
    const ERROR: &'static str = "Pick a plan to continue.";
    const ARIA_LABEL: &'static str = "\"aria-label\"";
}

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
                        .doc("The selected option. Pair it with `onchange`. `None` selects nothing, as for an unanswered question."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the option to select next."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("What the group posts as. A path such as `Survey::FIELDS.plan()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the selection, shown once the group loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single options. Named groups are drawn flat, without headings."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Renames an option. Runs during render, so it can read a locale from context."),
                    prop("option_description", "Callback<T, String>")
                        .doc("A line under each option's label. An empty string renders none."),
                    prop("variant", "ChoiceVariant")
                        .default("plain")
                        .doc("`card` draws every option as a bordered surface you can click anywhere. A row of cards stretches them to one height."),
                    prop("orientation", "Orientation")
                        .default("vertical")
                        .doc("`horizontal` lays the options out in a row, for two or three short ones."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Ring and dot color of the selected option."),
                    prop("size", "Size").default("md").doc("Size of the circles and their labels."),
                    prop("label", "Caption")
                        .doc("The question, and the group's name."),
                    prop("description", "Caption")
                        .doc("Between the question and the options. How to choose."),
                    prop("helper", "Caption")
                        .doc("Under the options. What the choice changes."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Sets `aria-required` on the group and marks the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every option and dims the group."),
                    readonly_prop("group").also("Chromium does not announce read-only on a group, so say it in the label or description where it matters."),
                ])
                .parts("RadioGroupPart", vec![
                    (RadioGroupPart::Label, "The label above the control."),
                    (RadioGroupPart::Required, "The required asterisk, in the label."),
                    (RadioGroupPart::Description, "The caption between the label and the control."),
                    (RadioGroupPart::Control, "The `radiogroup` holding the options."),
                    (RadioGroupPart::Circle, "Each option's ring."),
                    (RadioGroupPart::Dot, "Each option's checked mark."),
                    (RadioGroupPart::Helper, "The caption under the control."),
                    (RadioGroupPart::Status, "The validation message."),
                ]),
                props("Radio", vec![
                    prop("checked", "bool")
                        .doc("Whether it is selected. Pair it with `onselect`."),
                    prop("onselect", "EventHandler<()>")
                        .doc("Fires when this radio is picked. Never when another one is."),
                    prop("name", "String")
                        .doc("Shared by every radio in one group, which makes them exclusive. `RadioGroup` sets it."),
                    prop("tabindex", "String")
                        .doc("Which radio is the group's tab stop. `RadioGroup` sets it."),
                    prop("color", "ThemeAwareValue")
                        .default("primary")
                        .doc("Ring and dot color when selected."),
                    prop("size", "Size").default("md").doc("Size of the circle and its label."),
                    prop("label", "Caption").doc("The text beside the circle, and the radio's name."),
                    prop("description", "Caption").doc("A second line under the label."),
                    prop("helper", "Caption").doc("A caption under the radio."),
                    status_prop(),
                    required_prop().also("In a group, set it on `RadioGroup`."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Cannot be picked, dimmed and out of the tab order."),
                    prop("aria_label", "String")
                        .doc("Names the radio when it has no `label`."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Refuses the pick. ARIA has no read-only radio, so only `RadioGroup` can announce it."),
                    prop("variant", "ChoiceVariant")
                        .default("plain")
                        .doc("`card` draws the radio as a bordered surface you can click anywhere."),
                ])
                .parts("RadioPart", vec![
                    (RadioPart::Label, "The label beside the control."),
                    (RadioPart::Required, "The required asterisk, in the label."),
                    (RadioPart::Description, "The caption between the label and the control."),
                    (RadioPart::Control, "Holds the hidden input and the circle, beside the label."),
                    (RadioPart::Circle, "The drawn ring."),
                    (RadioPart::Dot, "The checked mark."),
                    (RadioPart::Helper, "The caption under the control."),
                    (RadioPart::Status, "The validation message."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Enters the group at the selected option, or the first one that is not disabled. Pressed again, leaves the group.")
                .key(["Down", "Right"], "Moves to the next option and selects it, wrapping at the end.")
                .key(["Up", "Left"], "Moves to the previous option and selects it, wrapping at the start.")
                .handles([
                    "The group is a `radiogroup` named by its `label`, so the question is read on entering it.",
                    "`required`, an error and read-only are set on the group, and its description, helper and status are its description.",
                    "Each option's description is read with its radio.",
                    "The whole group is one tab stop, and the arrows select as they move. Disabled options are skipped.",
                    "A group with no name logs a warning.",
                ])
                .must(["Without a visible `label`, spread `\"aria-label\"`, since the option labels do not say what the question is."])
                .example("A plan question, `RadioGroup { label: \"Plan\" }` with three options: Tab enters the group on the picked plan, a screen reader says \"Plan\" on the way in, and the arrows move and pick at once."),
            lead: rsx! {
                Text {
                    "A group of radios over an enum, exactly one of them selected. The group "
                    "is the field. It holds the question's label and captions, makes the "
                    "options exclusive and gives the whole set one tab stop."
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
                controls: [vec![
                    // A card only reads as one with a description, so the
                    // card brings the per-option descriptions with it.
                    Control::toggle("variant", ["plain", "card"])
                        .labels(["Plain", "Card"])
                        .default("plain")
                        .code(|_, values| match values.str("variant").as_str() {
                            "card" => vec![
                                r#"variant: "card""#.to_string(),
                                PLAN_DESCRIPTION.to_string(),
                            ],
                            _ => vec![],
                        }),
                    Control::color("color"),
                    Control::sizes("size")
                        .default("md"),
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical")
                        .code(|_, values| match values.str("orientation").as_str() {
                            "horizontal" => vec![r#"orientation: "horizontal""#.to_string()],
                            _ => vec![],
                        }),
                ], field_controls::<PlanCopy>(), vec![
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
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<PlanCopy>(&values);
                    rsx! {
                        RadioGroup {
                            variant: values.str("variant"),
                            option_description: (values.str("variant") == "card")
                                .then(|| Callback::new(plan_description)),
                            color: values.str("color"),
                            size: values.str("size"),
                            orientation: values.str("orientation"),
                            label: field.label,
                            "aria-label": field.aria_label,
                            description: field.description,
                            helper: field.helper,
                            status: field.status,
                            required: is_on(&values, "required").then_some(true),
                            disabled: is_on(&values, "disabled").then_some(true),
                            readonly: is_on(&values, "readonly").then_some(true),
                            options: plan_options(&values),
                            value: plan(),
                            onchange: move |next| plan.set(Some(next)),
                        }
                    }
                },
                wrap: Wrap(|_values, code| format!("{PLAN_ENUM}{code}")),
            }

            DocSection {
                title: "A Radio on its own",
                Text {
                    "Use a "
                    Code { source: "Radio" }
                    " on its own only to lay a group out by hand. Then the shared "
                    Code { source: "name" }
                    ", the tab order and the group's name are yours to set."
                }
            }
        }
    }
}
