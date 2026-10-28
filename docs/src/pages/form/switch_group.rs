use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls, field_props, prop,
    props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::SwitchGroupPart;
use libero::components::{Code, OptionList, Options, SwitchGroup, Text};
use libero::use_theme;

struct AlertCopy;

impl FieldCopy for AlertCopy {
    const LABEL: &'static str = "Alerts";
    const DESCRIPTION: &'static str = "Two at most, the rest stay off.";
    const HELPER: &'static str = "Text messages cost extra with your carrier.";
    const WARNING: &'static str = "Push needs notifications allowed.";
    const ERROR: &'static str = "Pick a channel to continue.";
    const ARIA_LABEL: &'static str = "\"aria-label\"";
}

/// The enum is the option list, so the snippet has to show it.
const ALERT_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Alert {
    Email,
    Push,
    #[option(label = "Text message")]
    Sms,
    Digest,
}

"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Alert {
    Email,
    Push,
    #[option(label = "Text message")]
    Sms,
    Digest,
}

/// What each channel sends, under its label - what makes a card worth drawing.
fn alert_description(alert: Alert) -> String {
    match alert {
        Alert::Email => "Each message to your inbox, as it lands.",
        Alert::Push => "A banner on your phone or desktop.",
        Alert::Sms => "Short texts, for the urgent ones only.",
        Alert::Digest => "One summary each morning.",
    }
    .to_string()
}

// snippet: after ALERT_ENUM
// snippet: let mut alerts = use_signal(|| vec![Alert::Email, Alert::Digest]);
// snippet: in SwitchGroup { value: alerts(), onchange: move |next| alerts.set(next), .. }
const ALERT_DESCRIPTION: &str = r#"option_description: move |alert: Alert| match alert {
    Alert::Email => "Each message to your inbox, as it lands.",
    Alert::Push => "A banner on your phone or desktop.",
    Alert::Sms => "Short texts, for the urgent ones only.",
    Alert::Digest => "One summary each morning.",
}
.to_string()"#;

/// The flag sits on the option, inside the one `options` prop.
// snippet: after ALERT_ENUM
// snippet: let mut alerts = use_signal(|| vec![Alert::Email, Alert::Digest]);
// snippet: in SwitchGroup { value: alerts(), onchange: move |next| alerts.set(next), .. }
const DISABLED_OPTION: &str =
    r#"options: OptionList::from_options().disabling(|alert| *alert == Alert::Sms)"#;

fn alert_options(values: &DemoValues) -> OptionList<Alert> {
    let off = is_on(values, "disabled_option");
    OptionList::from_options().disabling(move |alert| off && *alert == Alert::Sms)
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

fn max_checked(values: &DemoValues) -> Option<usize> {
    values.str("max_checked").parse().ok()
}

#[component]
pub fn SwitchGroupPage() -> Element {
    let mut alerts = use_signal(|| vec![Alert::Email, Alert::Digest]);
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "SwitchGroup",
            source: "libero/src/components/form/check_group.rs",
            markdown: "/md/switch_group.md",
            properties: vec![
                props("SwitchGroup", vec![
                    prop("value", "Vec<T>")
                        .doc("The switches that are on, in option order. Pair it with `onchange`. Empty turns none on."),
                    prop("onchange", "EventHandler<Vec<T>>")
                        .doc("Called with the whole selection to hold next, in option order whatever the click order."),
                    prop("name", "FieldName<Vec<T>>")
                        .doc("Posts one hidden input per switch that is on, with `Options::value`. A path such as `Settings::FIELDS.alerts()` also binds the group to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Vec<T>>")
                        .doc("Rules over the selection, shown once the group loses focus or its form is submitted."),
                    prop("max_checked", "usize")
                        .doc("At most this many on. At the cap the off switches disable, and switching one off frees them again."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single options. Named groups are drawn flat, without headings."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Renames an option. Runs during render, so it can read a locale from context."),
                    prop("option_description", "Callback<T, String>")
                        .doc("A line under each option's label. An empty string renders none."),
                    prop("variant", "ChoiceVariant")
                        .default(theme.switch.variant.as_str())
                        .doc("`card` draws every option as a bordered surface you can click anywhere."),
                    prop("orientation", "Orientation")
                        .default("vertical")
                        .doc("`horizontal` lays the options out in a row, for two or three short ones."),
                    prop("color", "ThemeAwareValue")
                        .default(theme.switch.color.as_str())
                        .doc("Track color of the switches that are on; `theme.switch.color` when unset."),
                    prop("size", "Size")
                        .default(theme.switch.size.as_str())
                        .doc("Size of the tracks, the thumbs and their labels."),
                    prop("label", "Caption")
                        .doc("The question, and the group's name."),
                    prop("description", "Caption")
                        .doc("Between the question and the options. How to choose."),
                    prop("helper", "Caption")
                        .doc("Under the options. What the choice changes."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the label. Inside a `Form`, a group with nothing on fails the submit. No option is required on its own."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every option and dims the group."),
                    readonly_prop("group"),
                ])
                .parts("SwitchGroupPart", vec![
                    (SwitchGroupPart::Label, "The label above the control."),
                    (SwitchGroupPart::Required, "The required asterisk, in the label."),
                    (SwitchGroupPart::Description, "The caption between the label and the control."),
                    (SwitchGroupPart::Control, "The `group` holding the options."),
                    (SwitchGroupPart::Track, "Each option's track."),
                    (SwitchGroupPart::Thumb, "Each option's thumb."),
                    (SwitchGroupPart::Helper, "The caption under the control."),
                    (SwitchGroupPart::Status, "The validation message."),
                ]),
            ],
            accessibility: a11y()
                .key(["Tab"], "Moves to the next option that is not disabled: every switch is its own tab stop.")
                .key(["Space"], "Toggles the focused switch.")
                .key(["Enter"], "Outside a `Form`: toggles the focused switch. Inside one: submits the form, as on a native checkbox.")
                .handles([
                    "The group is a `group` named by its `label`, so the question is read on entering it.",
                    "Its description, helper and status make up the group's `aria-describedby`.",
                    "A required group says \"Required\" after its label, from `common.required` in the localization: `aria-required` is not allowed on a `group`.",
                    "Each switch is a native checkbox announced as a switch, with its own label, and its description is read with it.",
                    "At `max_checked`, the off switches are disabled and leave the tab order, and a line under the switches says the cap, from `common.at_most` in the localization. The group's `aria-describedby` holds that line, and the switch that reaches the cap announces it. A `readonly` group disables nothing.",
                    "A group with no name logs a warning.",
                ])
                .must([
                    "Without a visible `label`, spread `\"aria-label\"`, since the option labels do not say what the question is.",
                ])
                .example("An alerts question, `SwitchGroup { label: \"Alerts\", max_checked: 2 }`: a screen reader says \"Alerts\" on the first switch, and once two are on the rest read as disabled."),
            lead: rsx! {
                Text {
                    "A group of switches over an enum, any number of them on. The group "
                    "is the field: it holds the question's label and captions, keeps the value as "
                    "one "
                    Code { source: "Vec<T>" }
                    " in option order and can cap how many are on."
                }
            },
            // snippet: let mut alerts = use_signal(|| vec![Alert::Email, Alert::Digest]);
            Demo {
                component: "SwitchGroup",
                children_text: "",
                fixed: vec![
                    "value: alerts()".to_string(),
                    "onchange: move |next| alerts.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::toggle("variant", ["plain", "card"])
                        .labels(["Plain", "Card"])
                        .default(theme.switch.variant.as_str())
                        .code(|_, values| match values.str("variant").as_str() {
                            "card" => vec![
                                r#"variant: "card""#.to_string(),
                                ALERT_DESCRIPTION.to_string(),
                            ],
                            _ => vec![],
                        }),
                    Control::color("color").default(theme.switch.color.as_str()),
                    Control::sizes("size")
                        .default(theme.switch.size.as_str()),
                    Control::toggle("orientation", ["horizontal", "vertical"])
                        .labels(["Horizontal", "Vertical"])
                        .default("vertical")
                        .code(|_, values| match values.str("orientation").as_str() {
                            "horizontal" => vec![r#"orientation: "horizontal""#.to_string()],
                            _ => vec![],
                        }),
                    Control::toggle("max_checked", ["none", "2"])
                        .labels(["None", "2"])
                        .default("2")
                        .code(|_, values| match max_checked(values) {
                            Some(max) => vec![format!("max_checked: {max}")],
                            None => vec![],
                        }),
                ], field_controls::<AlertCopy>().into_iter().map(|control| match control.name {
                    // The description states the cap the demo sets by default.
                    "description" => control.default("true"),
                    _ => control,
                }).collect(), vec![
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                    Control::switch("disabled_option").code(|_, values| {
                        match is_on(values, "disabled_option") {
                            true => vec![DISABLED_OPTION.to_string()],
                            false => vec![],
                        }
                    }),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<AlertCopy>(&values);
                    rsx! {
                        SwitchGroup {
                            variant: values.str("variant"),
                            option_description: (values.str("variant") == "card")
                                .then(|| Callback::new(alert_description)),
                            color: values.str("color"),
                            size: values.str("size"),
                            orientation: values.str("orientation"),
                            max_checked: max_checked(&values),
                            label: field.label,
                            "aria-label": field.aria_label,
                            description: field.description,
                            helper: field.helper,
                            status: field.status,
                            required: is_on(&values, "required").then_some(true),
                            disabled: is_on(&values, "disabled").then_some(true),
                            readonly: is_on(&values, "readonly").then_some(true),
                            options: alert_options(&values),
                            value: alerts(),
                            onchange: move |next| alerts.set(next),
                        }
                    }
                },
                wrap: Wrap(|_values, code| format!("{ALERT_ENUM}{code}")),
            }
        }
    }
}
