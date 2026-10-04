use super::dropdown_parts::{PHONE_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, disabled_prop, field_controls,
    field_props, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::PhoneFieldPart;
use libero::{
    components::{Code, Flex, PhoneField, Text},
    sx::sx,
};

struct MobileCopy;

impl FieldCopy for MobileCopy {
    const LABEL: &'static str = "Mobile";
    const DESCRIPTION: &'static str = "We only text you about deliveries.";
    const HELPER: &'static str = "Include the area code.";
    const WARNING: &'static str = "That looks short.";
    const ERROR: &'static str = "Enter a phone number.";
}

/// The flag emoji for an ISO 3166-1 alpha-2 code: two regional indicator symbols.
fn flag_emoji(iso: &str) -> String {
    iso.bytes()
        .filter(u8::is_ascii_alphabetic)
        .filter_map(|letter| {
            char::from_u32(0x1F1E6 + u32::from(letter.to_ascii_uppercase() - b'A'))
        })
        .collect()
}

/// A flag emoji for every country; the library ships no flags.
#[component]
fn Flag(iso: String) -> Element {
    rsx! {
        span { "aria-hidden": "true", "{flag_emoji(&iso)}" }
    }
}

#[component]
pub fn PhoneFieldPage() -> Element {
    let mut value = use_signal(String::new);

    rsx! {
        DocPage {
            title: "PhoneField",
            source: "libero/src/components/form/phone_field/field.rs",
            markdown: "/md/phone_field.md",
            properties: vec![
                props("PhoneField", vec![
                    prop("size", "Size").default("md").doc("Height, padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of `size`."),
                    prop("value", "Option<String>")
                        .doc("The number in E.164, such as `\"+12133734253\"`. Leave it out and the field keeps its own text."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires on every keystroke with the E.164 the field should hold next, or an empty string once nothing is typed."),
                    prop("country", "String")
                        .default("theme.phone_field.country")
                        .doc("The country the field starts on, ISO 3166-1 alpha-2, `US` by default. A pick wins over it until the prop changes. A `value` with another country's dial code wins over both."),
                    prop("oncountrychange", "EventHandler<String>")
                        .doc("The user picked another country. `oninput` fires at the same time with the number under the new dial code."),
                    prop("country_select", "bool")
                        .default("theme.phone_field.country_select")
                        .doc("Shows the country picker. Off pins the country and shows its dial code as plain text."),
                    prop("country_label", "Callback<String, String>")
                        .doc("Overrides the name of a country. Unset, the name comes from the localization's `phone_field.country_names`, which ships in German, else English. It runs during render, so it can read a locale from context."),
                    prop("countries", "Vec<String>")
                        .doc("Narrows the list to these ISO codes, in the order given."),
                    prop("flag", "Callback<String, Element>")
                        .doc("Draws a flag beside a country, in the picker and in the list. The library ships none. Hide it from screen readers, since the country's name is already read."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the E.164, shown once the field loses focus or its form is submitted. The field checks nothing on its own."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as, the E.164. A path such as `Signup::FIELDS.phone()` also binds the number to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the input."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. The format, or an example."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    disabled_prop("field"),
                    readonly_prop("field").also("The country button stays focusable and opens nothing."),
                    prop("dropdown_parts", "Parts<DropdownPart>")
                        .doc("Styles the portaled country list and its inner parts."),
                ])
                .parts("PhoneFieldPart", vec![
                    (PhoneFieldPart::Label, "The label above the control."),
                    (PhoneFieldPart::Required, "The required asterisk, in the label."),
                    (PhoneFieldPart::Description, "The caption between the label and the control."),
                    (PhoneFieldPart::Frame, "The bordered box around the control."),
                    (PhoneFieldPart::Leading, "The slot before the control: an icon, a prefix."),
                    (PhoneFieldPart::Control, "The element the label names."),
                    (PhoneFieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (PhoneFieldPart::Country, "The country button in the leading slot, with `country_select`."),
                    (PhoneFieldPart::Dial, "The dial code: in the country button, or alone without `country_select`."),
                    (PhoneFieldPart::Helper, "The caption under the control."),
                    (PhoneFieldPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(PHONE_DROPDOWN))
                .extends("input"),
            ],
            accessibility: a11y()
                .key(["Enter", "Space", "Down"], "On the country picker: opens the list.")
                .key(["Letter"], "Filters the open list.")
                .key(["Up", "Down"], "Move the highlight.")
                .key(["Enter"], "Picks the highlighted country and returns focus to the picker.")
                .key(["Escape"], "Closes the list and returns focus to the picker.")
                .handles([
                    "The country picker is a second tab stop.",
                    "Android's Back button closes the country list as Escape does, rather than the app.",
                ])
                .must(["Without a `label`, set `aria_label`. Otherwise screen readers announce an unnamed text field."]),
            lead: rsx! {
                Text {
                    "A country picker in front of a "
                    Code { source: "tel" }
                    " input. The picker holds the dial code, the input the national number, and "
                    "the value is one E.164 string such as "
                    Code { source: "\"+12133734253\"" }
                    ". The field regroups the digits when it loses focus, for countries with a "
                    "fixed number format. It ships the country list but no number validation, so "
                    "add a rule through "
                    Code { source: "validate" }
                    ". It does not strip a national trunk prefix: "
                    Code { source: "0171 1234567" }
                    " typed under Germany becomes "
                    Code { source: "\"+4901711234567\"" }
                    ", which is not E.164. Say so in "
                    Code { source: "helper" }
                    ", or add a rule that refuses a leading 0."
                }
            },
            // snippet: let mut phone = use_signal(String::new);
            // snippet: item #[component] fn Flag(iso: String) -> Element { rsx! {} }
            Demo {
                component: "PhoneField",
                children_text: "",
                fixed: vec![
                    "value: phone()".to_string(),
                    "oninput: move |next| phone.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                    Control::toggle("country", ["us", "de", "fr"])
                        .default("de")
                        .labels(["US", "DE", "FR"])
                        .code(|_, values| {
                            vec![format!("country: \"{}\"", values.str("country").to_uppercase())]
                        }),
                ], field_controls::<MobileCopy>(), vec![
                    Control::switch("placeholder").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"213 373 4253\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("flag").default("true").code(|_, values| {
                        match values.str("flag").as_str() {
                            "true" => vec![
                                "flag: move |iso: String| rsx! { Flag { iso } }".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::switch("country_select").default("true"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<MobileCopy>(&values);
                    let flag: Option<Callback<String, Element>> = (values.str("flag") == "true")
                        .then(|| Callback::new(move |iso: String| rsx! { Flag { iso } }));
                    let e164 = value();
                    rsx! {
                        // The column takes the width: sized by content, it outgrows a phone.
                        Flex { direction: "column", gap: "sm", align: "stretch",
                            sx: sx().width("100%").max_width("320px"),
                            PhoneField {
                                size: values.str("size"),
                                radius: values.str("radius"),
                                country: values.str("country").to_uppercase(),
                                label: field.label,
                                aria_label: field.aria_label,
                                description: field.description,
                                helper: field.helper,
                                placeholder: (values.str("placeholder") == "true")
                                    .then(|| "213 373 4253".to_string()),
                                status: field.status,
                                country_select: values.str("country_select") == "true",
                                required: (values.str("required") == "true").then_some(true),
                                disabled: (values.str("disabled") == "true").then_some(true),
                                flag,
                                value: value(),
                                oninput: move |next| value.set(next),
                            }
                            // The E.164 that posts, beside the national text on screen.
                            Text { size: "sm", "value: {e164:?}" }
                        }
                    }
                },
            }
        }
    }
}
