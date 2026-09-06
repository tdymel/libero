use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, FieldStatus, Flex, PhoneField, Text},
    sx::sx,
};

/// Three bands, which is every flag this example needs and no more - the
/// library ships none, so a caller draws their own.
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
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<String>")
                        .doc("The number, in E.164 - `\"+12133734253\"`. The text on screen is a rendering of it. `None` leaves the field uncontrolled."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke with the E.164 the field should hold next, or an empty string once nothing is typed."),
                    prop("country", "String")
                        .default("us")
                        .doc("The country the field starts on, ISO 3166-1 alpha-2. The picker wins over it until the prop changes, which moves the field and the number with it. A `value` whose dial code belongs to another country wins over both. Themed."),
                    prop("oncountrychange", "EventHandler<String>")
                        .doc("The user picked another country. `oninput` fires at the same time, with the value under the new dial code."),
                    prop("country_select", "bool")
                        .default("theme.phone_field.country_select")
                        .doc("Offers the picker at all. Off pins the country, draws a static `+49` in its place, and is one tab stop fewer."),
                    prop("country_label", "Callback<String, String>")
                        .doc("Overrides the English name a country is offered under, during render - so it can read a locale out of context."),
                    prop("countries", "Vec<String>")
                        .doc("Narrows the list to these ISO codes, in the order given."),
                    prop("flag", "Callback<String, Element>")
                        .doc("Draws a flag beside a country, in the picker and in the list. The library ships none."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as - the E.164, through a hidden input. A path also binds it to the surrounding form's value."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the E.164, shown once the field loses focus or its form is submitted. Nothing here validates a number on its own."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the `<input>` through a `for`/`id` pair."),
                    prop("description", "Caption")
                        .doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control: the format, or an example."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required, adds `aria-required` and shows an asterisk in the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the field."),
                    prop("readonly", "bool")
                        .default("false")
                        .doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A country picker in front of a "
                    Code { source: "tel" }
                    " input. The value is E.164 and only E.164 - "
                    Code { source: "\"+12133734253\"" }
                    " - and the text on screen is a rendering of it, so the picker carries the "
                    "dial code and what is typed is the national number. The library ships the "
                    "country list and no validation at all: a rule over the number is an "
                    "ordinary "
                    Code { source: "validate" }
                    " line."
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
                            _ => vec![],
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
        }
    }
}
