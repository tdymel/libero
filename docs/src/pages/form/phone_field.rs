use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Button, Code, CodeBlock, FieldStatus, Fields, Flex, Form, PhoneField, Text},
    sx::sx,
};

const FLAG_CODE: &str = r#"PhoneField {
    label: "Mobile",
    countries: vec!["DE".into(), "FR".into(), "IT".into(), "BE".into()],
    // Drawn in the picker and in every row. The library ships none: an SVG
    // sprite of every flag is ~44 KB gzipped, and emoji flags render as two
    // letters on Windows.
    flag: move |iso: String| rsx! { Tricolour { iso } },
    value: phone(),
    oninput: move |next| phone.set(next),
}"#;

const LABEL_CODE: &str = r#"PhoneField {
    label: "Mobil",
    // Called during render, so it can read a locale out of context.
    country_label: move |iso: String| match iso.as_str() {
        "DE" => "Deutschland".to_string(),
        "FR" => "Frankreich".to_string(),
        _ => iso,
    },
    countries: vec!["DE".into(), "FR".into()],
}"#;

const FORM_CODE: &str = r#"#[derive(Clone, PartialEq, Default, Fields)]
struct Signup {
    phone: String,
}

let signup = use_store(Signup::default);

rsx! {
    Form {
        value: signup,
        onsubmit: move |event: FormEvent| {
            // One entry named "phone", and what it carries is the E.164 - the
            // visible input has no name and contributes nothing.
            posted.set(Some(event.values().iter().filter(|(name, _)| name == "phone").count()));
        },
        // No `oninput`: a path binds the number to the form's own value.
        PhoneField { label: "Mobile", country: "DE", name: Signup::FIELDS.phone() }
        Button { r#type: "submit", "Save" }
    }
}"#;

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Signup {
    pub phone: String,
}

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

/// A bound field with no handler of its own, so the form's value is the only
/// place the number lives - and the submit shows what the browser would send.
#[component]
fn SignupForm() -> Element {
    let signup = use_store(Signup::default);
    let mut posted = use_signal(|| None::<usize>);

    rsx! {
        Form {
            sx: sx().width("320px"),
            value: signup,
            onsubmit: move |event: FormEvent| {
                posted.set(Some(event.values().iter().filter(|(name, _)| name == "phone").count()));
            },
            PhoneField {
                label: "Mobile",
                country: "DE",
                placeholder: "30 123456",
                name: Signup::FIELDS.phone(),
            }
            Button { r#type: "submit", "Save" }
            if let Some(count) = posted() {
                Text { "Posted {count} entry named \"phone\", and the form's own value holds \"{signup().phone}\"." }
            }
        }
    }
}

#[component]
pub fn PhoneFieldPage() -> Element {
    let mut value = use_signal(String::new);
    let mut flagged = use_signal(String::new);

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
                        .default("true")
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
                        .default("us")
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
                    Control::switch("country_select").default("true"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    PhoneField {
                        sx: sx().width("320px"),
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
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                },
            }
            DocSection {
                title: "What posts",
                Text {
                    "The visible input holds the text being edited, so it carries no "
                    Code { source: "name" }
                    " - it would post what is on screen. The E.164 posts through one hidden "
                    "input of that name instead, the shape "
                    Code { source: "Slider" }
                    " and "
                    Code { source: "PinField" }
                    " already use. A "
                    Code { source: "FieldName" }
                    " built from a path also binds the number to the surrounding form's value, "
                    "so a bound field needs no "
                    Code { source: "oninput" }
                    " of its own."
                }
                Flex { direction: "column", gap: "md", align: "flex-start",
                    SignupForm {}
                    CodeBlock { language: "rust", source: FORM_CODE }
                }
            }
            DocSection {
                title: "Country names and flags",
                Text {
                    "The names come from the component's own table, in English. "
                    Code { source: "country_label" }
                    " overrides one during render, which is what lets it read a locale out of "
                    "context - the library bundles no translations. "
                    Code { source: "countries" }
                    " narrows the list, in the order given."
                }
                Text {
                    "No flags ship with the library. Emoji flags render as two letters on "
                    "Windows, and an inline SVG sprite of every flag costs about 44 KB "
                    "gzipped - a third of what route splitting won back in the bundle audit. "
                    Code { source: "flag" }
                    " is the hook for a caller who wants them, drawn in the picker and in "
                    "every row."
                }
                Flex { direction: "column", gap: "md", align: "flex-start",
                    PhoneField {
                        sx: sx().width("320px"),
                        label: "Mobile",
                        country: "DE",
                        countries: vec![
                            "DE".to_string(),
                            "FR".to_string(),
                            "IT".to_string(),
                            "BE".to_string(),
                        ],
                        flag: move |iso: String| rsx! { Tricolour { iso } },
                        value: flagged(),
                        oninput: move |next| flagged.set(next),
                    }
                    CodeBlock { language: "rust", source: FLAG_CODE }
                    CodeBlock { language: "rust", source: LABEL_CODE }
                }
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Two tab stops, on purpose. The "
                    Code { source: "<input type=\"tel\">" }
                    " is the labelled control, so the label names it with a plain "
                    Code { source: "for" }
                    " and the captions reach it through "
                    Code { source: "aria-describedby" }
                    ". The picker is the second: it is a real "
                    Code { source: "<button>" }
                    " and the only way to reach what it does, so a keyboard has to be able to "
                    "land on it. With "
                    Code { source: "country_select: false" }
                    " the leading slot is a plain span and there is no extra stop."
                }
                Text {
                    "Enter, Space and ArrowDown open the list; the arrows move the highlight, "
                    "Enter picks, Escape closes, and both hand the focus back to the button. "
                    "The list opens with a search box focused - 240 countries are not a list "
                    "anyone scrolls - and while it is open that box owns "
                    Code { source: "role=\"combobox\"" }
                    " and "
                    Code { source: "aria-activedescendant" }
                    ", with the button keeping only "
                    Code { source: "aria-haspopup" }
                    " and "
                    Code { source: "aria-expanded" }
                    ". Nothing in the text input is intercepted: digits, Backspace and the "
                    "arrows are all native, and the number is only regrouped once the field "
                    "loses focus, because regrouping as it is typed would move the caret."
                }
            }
        }
    }
}
