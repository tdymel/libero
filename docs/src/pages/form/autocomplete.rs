use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Autocomplete, AutocompleteFilterArgs, AutocompleteOptionArgs, Code, FieldStatus, Flex, Text,
    },
    sx::sx,
};

/// The suggestion list is runtime data, so every snippet has to show it.
const CITIES_CONST: &str = r#"const CITIES: [(&str, &str); 6] = [
    ("Amsterdam", "Netherlands"),
    ("Antwerp", "Belgium"),
    ("Berlin", "Germany"),
    ("Bern", "Switzerland"),
    ("Copenhagen", "Denmark"),
    ("Cologne", "Germany"),
];

fn cities() -> Vec<String> {
    CITIES.iter().map(|(city, _)| city.to_string()).collect()
}

fn country(city: &str) -> &'static str {
    CITIES
        .iter()
        .find(|(name, _)| *name == city)
        .map(|(_, country)| *country)
        .unwrap_or("")
}

"#;

// snippet: after CITIES_CONST
// snippet: let mut value = use_signal(String::new);
// snippet: in Autocomplete { options: cities(), value: value(), oninput: move |next| value.set(next), .. }
const CUSTOM_OPTION: &str = r#"option: move |o: AutocompleteOptionArgs<String>| rsx! {
    Flex {
        direction: "column",
        align: "flex-start",
        sx: sx().gap("0"),
        Text { component: "span", size: "sm", "{o.value}" }
        Text { component: "span", size: "xs", sx: sx().color("grey.6"), "{country(&o.value)}" }
    }
}"#;

/// Prefix rather than the default `contains` - the same list, narrowed by a
/// different rule.
// snippet: after CITIES_CONST
// snippet: let mut value = use_signal(String::new);
// snippet: in Autocomplete { options: cities(), value: value(), oninput: move |next| value.set(next), .. }
const CUSTOM_FILTER: &str = r#"filter: move |f: AutocompleteFilterArgs<String>| {
    f.value.to_lowercase().starts_with(&f.query.to_lowercase())
}"#;

// snippet: after CITIES_CONST
// snippet: let mut value = use_signal(String::new);
// snippet: let mut picked = use_signal(|| None::<&'static str>);
// snippet: in Autocomplete { options: cities(), value: value(), oninput: move |next| value.set(next), .. }
const ONPICK: &str = r#"onpick: move |city: String| picked.set(Some(country(&city)))"#;

const CITIES: [(&str, &str); 6] = [
    ("Amsterdam", "Netherlands"),
    ("Antwerp", "Belgium"),
    ("Berlin", "Germany"),
    ("Bern", "Switzerland"),
    ("Copenhagen", "Denmark"),
    ("Cologne", "Germany"),
];

fn cities() -> Vec<String> {
    CITIES.iter().map(|(city, _)| city.to_string()).collect()
}

fn country(city: &str) -> &'static str {
    CITIES
        .iter()
        .find(|(name, _)| *name == city)
        .map(|(_, country)| *country)
        .unwrap_or("")
}

fn custom(values: &DemoValues) -> bool {
    values.str("custom") == "true"
}

fn city_row(o: AutocompleteOptionArgs<String>) -> Element {
    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            sx: sx().gap("0"),
            Text { component: "span", size: "sm", "{o.value}" }
            Text {
                component: "span",
                size: "xs",
                sx: sx().color("grey.6"),
                "{country(&o.value)}"
            }
        }
    }
}

fn starts_with(f: AutocompleteFilterArgs<String>) -> bool {
    f.value.to_lowercase().starts_with(&f.query.to_lowercase())
}

#[component]
pub fn AutocompletePage() -> Element {
    let mut value = use_signal(String::new);
    let mut picked = use_signal(|| None::<&'static str>);

    rsx! {
        DocPage {
            title: "Autocomplete",
            source: "libero/src/components/form/autocomplete.rs",
            markdown: "/md/autocomplete.md",
            properties: vec![
                props("Autocomplete", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, font size and the rows' size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list, independent of size."),
                    prop("value", "String")
                        .default("\"\"")
                        .doc("The text; strictly controlled. Picking a suggestion inserts its label - the value is never a `T`."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke, and again with the label when a suggestion is picked or the field is cleared."),
                    prop("options", "Vec<T>")
                        .default("[]")
                        .doc("The suggestions to offer. `T` is inferred from it, so no call site annotates one."),
                    prop("option", "Callback<AutocompleteOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. The row itself - highlight, click - stays the component's. No `selected`: a suggestion is not a selection."),
                    prop("onpick", "EventHandler<T>")
                        .doc("A suggestion was accepted, with the whole value behind the text. Fires after `oninput`."),
                    prop("filter", "Callback<AutocompleteFilterArgs<T>, bool>")
                        .default("contains")
                        .doc("Narrows `options`. Defaults to a case-insensitive `contains` over the label."),
                    prop("prefiltered", "bool")
                        .default("false")
                        .doc("`options` arrives already narrowed - a list fetched per keystroke. Skips filtering, so `filter` never runs."),
                    prop("placeholder", "String").doc("Shown while the field is empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x that empties the field, at the end of the frame."),
                    prop("empty", "Element")
                        .doc("Shown in place of the list when nothing matches. Without it a list with no rows draws nothing."),
                    prop("leading", "Element").doc("Inside the frame, before the control - a search icon."),
                    prop("trailing", "Element").doc("Inside the frame, after the control, before the clear x."),
                    prop("label", "Caption").doc("The field's caption, above the control."),
                    prop("description", "Caption").doc("Between the label and the control: what to enter."),
                    prop("helper", "Caption").doc("Under the control: formatting rules, or what the entry affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Takes the input out of the tab order and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "TextField" }
                    " that offers completions. The value is a "
                    Code { source: "String" }
                    " at all times - picking a suggestion inserts its label, it does not make the field hold a "
                    Code { source: "T" }
                    ". To choose out of a fixed set instead, reach for "
                    Code { source: "Select" }
                    ". Nothing is highlighted until you press ArrowDown, so Enter on text that matches nothing still submits a form."
                }
            },
            // snippet: let mut value = use_signal(String::new);
            // snippet: let mut picked = use_signal(|| None::<&'static str>);
            Demo {
                component: "Autocomplete",
                children_text: "",
                // `country` only exists in the snippet while the rows or the
                // pick handler call it.
                wrap: Wrap(|_values: &DemoValues, source: &str| format!("{CITIES_CONST}{source}")),
                fixed: vec![
                    "sx: sx().width(\"280px\")".to_string(),
                    "options: cities()".to_string(),
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                    "placeholder: \"Start typing\"".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"We don't deliver there yet.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Pick a city from the list.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"City\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Where the parcel goes.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Anything you type is kept, list or not.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    // A row drawn with `option`, and the `onpick` that reaches
                    // the record behind the text.
                    Control::switch("custom").code(|_, values| match custom(values) {
                        true => vec![CUSTOM_OPTION.to_string(), ONPICK.to_string()],
                        false => vec![],
                    }),
                    // Prefix instead of the default `contains`.
                    Control::switch("filter").code(|_, values| {
                        match values.str("filter").as_str() {
                            "true" => vec![CUSTOM_FILTER.to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("empty").code(|_, values| match values.str("empty").as_str() {
                        "true" => vec![
                            "empty: rsx! { Text { \"No city by that name.\" } }".to_string(),
                        ],
                        _ => vec![],
                    }),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Autocomplete {
                        sx: sx().width("280px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "City".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "Where the parcel goes.".to_string()),
                        helper: match (values.str("helper").as_str(), picked()) {
                            ("true", _) => Some("Anything you type is kept, list or not.".to_string()),
                            (_, Some(country)) => Some(format!("Picked a city in {country}.")),
                            _ => None,
                        },
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("We don't deliver there yet.".to_string()),
                            "error" => FieldStatus::Error("Pick a city from the list.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        option: custom(&values).then(|| Callback::new(city_row)),
                        onpick: custom(&values).then(|| {
                            Callback::new(move |city: String| picked.set(Some(country(&city))))
                        }),
                        filter: (values.str("filter") == "true")
                            .then(|| Callback::new(starts_with)),
                        empty: (values.str("empty") == "true")
                            .then(|| rsx! { Text { "No city by that name." } }),
                        clearable: (values.str("clearable") == "true").then_some(true),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        placeholder: "Start typing",
                        options: cities(),
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
