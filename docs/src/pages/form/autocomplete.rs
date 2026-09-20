use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
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
        Text { component: "span", size: "xs", sx: sx().color("muted.6"), "{country(&o.value)}" }
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
                sx: sx().color("muted.6"),
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
                    prop("size", "Size").default("md").doc("Height, padding and font size of the field and its rows."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list."),
                    prop("value", "String")
                        .default("\"\"")
                        .doc("The text. Pair it with `oninput`. Picking a suggestion inserts its label."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per keystroke, and again with the label when a suggestion is picked or the field is cleared."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.city()` also binds it to the surrounding `Form`'s value when it has no `oninput`."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the text, shown once the field loses focus or its form is submitted."),
                    prop("options", "Vec<T>")
                        .default("[]")
                        .doc("The suggestions to offer. `T` is inferred from it."),
                    prop("option", "Callback<AutocompleteOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. The highlight and click stay the component's."),
                    prop("onpick", "EventHandler<T>")
                        .doc("A suggestion was accepted, with the whole value behind the text. Fires after `oninput`."),
                    prop("filter", "Callback<AutocompleteFilterArgs<T>, bool>")
                        .default("contains")
                        .doc("Narrows `options`. Defaults to a case-insensitive `contains` over the label."),
                    prop("prefiltered", "bool")
                        .default("false")
                        .doc("`options` arrives already narrowed, such as a list fetched per keystroke. Skips filtering, so `filter` never runs."),
                    prop("placeholder", "String").doc("Shown while the field is empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x at the end of the frame that empties the field."),
                    prop("empty", "Element")
                        .doc("Shown in place of the list when nothing matches. Screen readers hear the localization's `combobox.nothing_found` either way, so change that string to match."),
                    prop("leading", "Element").doc("Inside the frame, before the control, such as a search icon."),
                    prop("trailing", "Element").doc("Inside the frame, after the control, before the clear x."),
                    prop("describe_leading", "bool")
                        .default("false")
                        .doc("`leading` is text that describes the value, such as a unit, so screen readers read it with the input."),
                    prop("describe_trailing", "bool")
                        .default("false")
                        .doc("The same for `trailing`."),
                    prop("label", "Caption").doc("The caption above the control, and the field's name."),
                    prop("description", "Caption").doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption").doc("Under the control. Formatting rules, or what the entry changes."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Sets `aria-required` and marks the label."),
                    prop("disabled", "bool").default("false").doc("Takes the input out of the tab order and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` drops the field from the tab order and the post instead."),
                ]),
            ],
            accessibility: a11y()
                .key(["Down"], "Opens the list. Typing opens it too.")
                .key(["Up", "Down", "Home", "End"], "Move the highlight.")
                .key(["Enter"], "Picks the highlighted row.")
                .key(["Escape", "Tab"], "Close the list.")
                .handles([
                    "Nothing is highlighted until you arrow onto a row, so Enter on text that matches nothing still submits the form.",
                ]),
            lead: rsx! {
                Text {
                    "A "
                    Code { source: "TextField" }
                    " that offers completions. The value stays a "
                    Code { source: "String" }
                    ", and picking a suggestion inserts its label. To choose from a fixed set, "
                    "use "
                    Code { source: "Select" }
                    "."
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
                        .labels(["Valid", "Warning", "Error"])
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
                            _ => vec!["aria_label: \"City\"".to_string()],
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
                        aria_label: (values.str("label") != "true").then_some("City"),
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
