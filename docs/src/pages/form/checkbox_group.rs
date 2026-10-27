use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls, field_props, prop,
    props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::CheckboxGroupPart;
use libero::components::{CheckboxGroup, Code, OptionList, Options, Text};
use libero::use_theme;

struct ToppingCopy;

impl FieldCopy for ToppingCopy {
    const LABEL: &'static str = "Toppings";
    const DESCRIPTION: &'static str = "Two at most, on the house.";
    const HELPER: &'static str = "More cost extra at the counter.";
    const WARNING: &'static str = "Olives run out by noon.";
    const ERROR: &'static str = "Pick a topping to continue.";
    const ARIA_LABEL: &'static str = "\"aria-label\"";
}

/// The enum is the option list, so the snippet has to show it.
const TOPPING_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Olives,
    Basil,
    #[option(label = "Chili flakes")]
    Chili,
}

"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Olives,
    Basil,
    #[option(label = "Chili flakes")]
    Chili,
}

/// What each topping adds, under its label - what makes a card worth drawing.
fn topping_description(topping: Topping) -> String {
    match topping {
        Topping::Cheese => "Aged, grated over the top.",
        Topping::Olives => "Black, pitted.",
        Topping::Basil => "Fresh leaves, added last.",
        Topping::Chili => "Hot, on the side.",
    }
    .to_string()
}

// snippet: after TOPPING_ENUM
// snippet: let mut toppings = use_signal(|| vec![Topping::Cheese, Topping::Basil]);
// snippet: in CheckboxGroup { value: toppings(), onchange: move |next| toppings.set(next), .. }
const TOPPING_DESCRIPTION: &str = r#"option_description: move |topping: Topping| match topping {
    Topping::Cheese => "Aged, grated over the top.",
    Topping::Olives => "Black, pitted.",
    Topping::Basil => "Fresh leaves, added last.",
    Topping::Chili => "Hot, on the side.",
}
.to_string()"#;

/// The flag sits on the option, inside the one `options` prop.
// snippet: after TOPPING_ENUM
// snippet: let mut toppings = use_signal(|| vec![Topping::Cheese, Topping::Basil]);
// snippet: in CheckboxGroup { value: toppings(), onchange: move |next| toppings.set(next), .. }
const DISABLED_OPTION: &str =
    r#"options: OptionList::from_options().disabling(|topping| *topping == Topping::Chili)"#;

fn topping_options(values: &DemoValues) -> OptionList<Topping> {
    let off = is_on(values, "disabled_option");
    OptionList::from_options().disabling(move |topping| off && *topping == Topping::Chili)
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

fn max_checked(values: &DemoValues) -> Option<usize> {
    values.str("max_checked").parse().ok()
}

#[component]
pub fn CheckboxGroupPage() -> Element {
    let mut toppings = use_signal(|| vec![Topping::Cheese, Topping::Basil]);
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "CheckboxGroup",
            source: "libero/src/components/form/check_group.rs",
            markdown: "/md/checkbox_group.md",
            properties: vec![
                props("CheckboxGroup", vec![
                    prop("value", "Vec<T>")
                        .doc("The checked options, in option order. Pair it with `onchange`. Empty checks nothing."),
                    prop("onchange", "EventHandler<Vec<T>>")
                        .doc("Called with the whole selection to hold next, in option order whatever the click order."),
                    prop("name", "FieldName<Vec<T>>")
                        .doc("Posts one hidden input per checked option, with `Options::value`. A path such as `Order::FIELDS.toppings()` also binds the group to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Vec<T>>")
                        .doc("Rules over the selection, shown once the group loses focus or its form is submitted."),
                    prop("max_checked", "usize")
                        .doc("At most this many checked. At the cap the unchecked options disable, and unchecking one frees them again."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList<T>` can disable single options. Named groups are drawn flat, without headings."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Renames an option. Runs during render, so it can read a locale from context."),
                    prop("option_description", "Callback<T, String>")
                        .doc("A line under each option's label. An empty string renders none."),
                    prop("variant", "ChoiceVariant")
                        .default(theme.checkbox.variant.as_str())
                        .doc("`card` draws every option as a bordered surface you can click anywhere."),
                    prop("orientation", "Orientation")
                        .default("vertical")
                        .doc("`horizontal` lays the options out in a row, for two or three short ones."),
                    prop("color", "ThemeAwareValue")
                        .default(theme.checkbox.color.as_str())
                        .doc("Box color of the checked options; `theme.checkbox.color` when unset."),
                    prop("size", "Size")
                        .default(theme.checkbox.size.as_str())
                        .doc("Size of the boxes and their labels."),
                    prop("label", "Caption")
                        .doc("The question, and the group's name."),
                    prop("description", "Caption")
                        .doc("Between the question and the options. How to choose."),
                    prop("helper", "Caption")
                        .doc("Under the options. What the choice changes."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the label. Inside a `Form`, a group with nothing checked fails the submit. No option is required on its own."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables every option and dims the group."),
                    readonly_prop("group"),
                ])
                .parts("CheckboxGroupPart", vec![
                    (CheckboxGroupPart::Label, "The label above the control."),
                    (CheckboxGroupPart::Required, "The required asterisk, in the label."),
                    (CheckboxGroupPart::Description, "The caption between the label and the control."),
                    (CheckboxGroupPart::Control, "The `group` holding the options."),
                    (CheckboxGroupPart::Box, "Each option's box."),
                    (CheckboxGroupPart::Helper, "The caption under the control."),
                    (CheckboxGroupPart::Status, "The validation message."),
                ]),
                props("Domain enums", vec![
                    prop(".choices(&X)", "Mapped<Vec<V>, Vec<Choice<V>>, Infallible>")
                        .doc("On a path to `Vec<V>`: binds `name` to a domain enum through a `Choices<V>` list, so the form's value holds `Vec<V>` and the field edits `Vec<Choice<V>>`. Pass `X.list()` as `options` too. `.rules(..)` after it run over `Vec<V>`."),
                ]).without_base_props(),
            ],
            accessibility: a11y()
                .key(["Tab"], "Moves to the next option that is not disabled: every box is its own tab stop.")
                .key(["Space"], "Checks or unchecks the focused option.")
                .handles([
                    "The group is a `group` named by its `label`, so the question is read on entering it.",
                    "Its description, helper and status make up the group's `aria-describedby`.",
                    "A required group says \"Required\" after its label, from `common.required` in the localization: `aria-required` is not allowed on a `group`.",
                    "Each option is a native checkbox with its own label, and its description is read with it.",
                    "At `max_checked`, the unchecked options are disabled and leave the tab order, and the group's `aria-describedby` says the cap, from `common.at_most` in the localization. A `readonly` group disables nothing.",
                    "A group with no name logs a warning.",
                ])
                .must([
                    "Without a visible `label`, spread `\"aria-label\"`, since the option labels do not say what the question is.",
                ])
                .example("A toppings question, `CheckboxGroup { label: \"Toppings\", max_checked: 2 }`: a screen reader says \"Toppings\" on the first box, and once two are checked the rest read as disabled."),
            lead: rsx! {
                Text {
                    "A group of checkboxes over an enum, any number of them checked. The group "
                    "is the field: it holds the question's label and captions, keeps the value as "
                    "one "
                    Code { source: "Vec<T>" }
                    " in option order and can cap how many are checked."
                }
            },
            // snippet: let mut toppings = use_signal(|| vec![Topping::Cheese, Topping::Basil]);
            Demo {
                component: "CheckboxGroup",
                children_text: "",
                fixed: vec![
                    "value: toppings()".to_string(),
                    "onchange: move |next| toppings.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::toggle("variant", ["plain", "card"])
                        .labels(["Plain", "Card"])
                        .default(theme.checkbox.variant.as_str())
                        .code(|_, values| match values.str("variant").as_str() {
                            "card" => vec![
                                r#"variant: "card""#.to_string(),
                                TOPPING_DESCRIPTION.to_string(),
                            ],
                            _ => vec![],
                        }),
                    Control::color("color").default(theme.checkbox.color.as_str()),
                    Control::sizes("size")
                        .default(theme.checkbox.size.as_str()),
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
                ], field_controls::<ToppingCopy>().into_iter().map(|control| match control.name {
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
                    let field = field_props::<ToppingCopy>(&values);
                    rsx! {
                        CheckboxGroup {
                            variant: values.str("variant"),
                            option_description: (values.str("variant") == "card")
                                .then(|| Callback::new(topping_description)),
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
                            options: topping_options(&values),
                            value: toppings(),
                            onchange: move |next| toppings.set(next),
                        }
                    }
                },
                wrap: Wrap(|_values, code| format!("{TOPPING_ENUM}{code}")),
            }
        }
    }
}
