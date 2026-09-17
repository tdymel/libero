use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, FieldStatus, Flex, PhoneField, Text},
    sx::sx,
};

/// Three bands, every flag this example needs. The library ships none.
#[component]
fn Tricolour(iso: String) -> Element {
    let (bands, vertical) = match iso.as_str() {
        "DE" => (["#000000", "#dd0000", "#ffce00"], false),
        "FR" => (["#002395", "#ffffff", "#ed2939"], true),
        "IT" => (["#008c45", "#f4f5f0", "#cd212a"], true),
        "BE" => (["#000000", "#fae042", "#ed2939"], true),
        _ => (["#cccccc", "#eeeeee", "#cccccc"], false),
    };

    rsx! {
        svg {
            width: "16",
            height: "12",
            view_box: "0 0 3 3",
            "aria-hidden": "true",
            for (index , band) in bands.iter().enumerate() {
                rect {
                    key: "{index}",
                    x: if vertical { index.to_string() } else { "0".to_string() },
                    y: if vertical { "0".to_string() } else { index.to_string() },
                    width: if vertical { "1" } else { "3" },
                    height: if vertical { "3" } else { "1" },
                    fill: "{band}",
                }
            }
        }
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
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. The country button stays focusable and opens nothing."),
                ]).extends("input"),
            ],
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
                    "."
                }
            },
            // snippet: let mut phone = use_signal(String::new);
            // snippet: item #[component] fn Tricolour(iso: String) -> Element { rsx! {} }
            Demo {
                component: "PhoneField",
                children_text: "",
                fixed: vec![
                    "value: phone()".to_string(),
                    "oninput: move |next| phone.set(next)".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("country", ["us", "de", "fr"])
                        .default("de")
                        .labels(["US", "DE", "FR"])
                        .code(|_, values| {
                            vec![format!("country: \"{}\"", values.str("country").to_uppercase())]
                        }),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That looks short.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Enter a phone number.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Mobile\"".to_string()],
                            _ => vec!["aria_label: \"Mobile\"".to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"We only text you about deliveries.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("placeholder").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"213 373 4253\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("flag").default("true").code(|_, values| {
                        match values.str("flag").as_str() {
                            "true" => vec![
                                "flag: move |iso: String| rsx! { Tricolour { iso } }".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::switch("country_select").default("true"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| {
                    let flag: Option<Callback<String, Element>> = (values.str("flag") == "true")
                        .then(|| Callback::new(move |iso: String| rsx! { Tricolour { iso } }));
                    let e164 = value();
                    rsx! {
                        // The column, not the field, takes the width: a column
                        // sized by its content would be as wide as the input's
                        // natural width, which is more than a phone has.
                        Flex { direction: "column", gap: "sm", align: "stretch",
                            sx: sx().width("100%").max_width("320px"),
                            PhoneField {
                                size: values.str("size"),
                                radius: values.str("radius"),
                                country: values.str("country").to_uppercase(),
                                label: (values.str("label") == "true").then(|| "Mobile".to_string()),
                                aria_label: (values.str("label") != "true").then_some("Mobile"),
                                description: (values.str("description") == "true")
                                    .then(|| "We only text you about deliveries.".to_string()),
                                placeholder: (values.str("placeholder") == "true")
                                    .then(|| "213 373 4253".to_string()),
                                status: match values.str("status").as_str() {
                                    "warning" => FieldStatus::Warning("That looks short.".to_string()),
                                    "error" => FieldStatus::Error("Enter a phone number.".to_string()),
                                    _ => FieldStatus::Valid,
                                },
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
            DocSection { title: "Accessibility",
                Text {
                    "The country picker is a second tab stop. Enter, Space and Arrow Down open "
                    "the list, typing filters it, the arrows move the highlight, Enter picks and "
                    "Escape closes. Both return focus to the picker."
                }
            }
        }
    }
}
