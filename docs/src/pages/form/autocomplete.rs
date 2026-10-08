use super::CLEAR_NAME;
use super::dropdown_parts::{SUGGESTION_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls,
    field_props, prop, props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::{
    components::{Autocomplete, AutocompleteFilterArgs, AutocompleteOptionArgs, Code, Flex, Text},
    sx::sx,
    use_theme,
};

struct CityCopy;

impl FieldCopy for CityCopy {
    const LABEL: &'static str = "City";
    const DESCRIPTION: &'static str = "Where the parcel goes.";
    const HELPER: &'static str = "Anything you type is kept, list or not.";
    const WARNING: &'static str = "We don't deliver there yet.";
    const ERROR: &'static str = "Pick a city from the list.";
}

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("autocomplete.rs"));

/// The suggestion list is runtime data, so every snippet prints it.
// demo-code: cities start
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
// demo-code: cities end

fn custom(values: &DemoValues) -> bool {
    values.str("custom") == "true"
}

fn city_row(o: AutocompleteOptionArgs<String>) -> Element {
    // demo-code: row start
    rsx! {
        Flex {
            direction: "column",
            align: "flex-start",
            sx: sx().gap("0"),
            Text { component: "span", size: "sm", "{o.value}" }
            Text {
                component: "span",
                size: "xs",
                // Shade 6 is 2.8:1 on the active row; 7 passes on it in both schemes.
                sx: sx().color("muted.7"),
                "{country(&o.value)}"
            }
        }
    }
    // demo-code: row end
}

/// Prefix rather than the default `contains`: the same list, narrowed by a different rule.
fn starts_with(f: AutocompleteFilterArgs<String>) -> bool {
    // demo-code: filter start
    f.value.to_lowercase().starts_with(&f.query.to_lowercase())
    // demo-code: filter end
}

#[component]
pub fn AutocompletePage() -> Element {
    let mut value = use_signal(String::new);
    let mut picked = use_signal(|| None::<&'static str>);
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Autocomplete",
            source: "libero/src/components/form/autocomplete.rs",
            markdown: "/md/autocomplete.md",
            properties: vec![
                props("Autocomplete", vec![
                    prop("size", "Size")
                        .default(theme.autocomplete.size.as_str())
                        .doc("Height, padding and font size of the field and its rows."),
                    prop("radius", "Size")
                        .default(theme.autocomplete.radius.as_str())
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
                    prop("options", "OptionSource<T>")
                        .default("[]")
                        .doc("The suggestions to offer. `T` is inferred from it. A `Vec<T>` converts, and so does a `Resource<Vec<T>>` or an `Option<OptionList<T>>` that is `None` while you fetch: a pending list shows the loader and says `loading_label`, never \"No results\" for an answer that has not arrived. Groups and disabled options of an `OptionList` are not drawn."),
                    prop("option", "Callback<AutocompleteOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. The highlight and click stay the component's."),
                    prop("onpick", "EventHandler<T>")
                        .doc("A suggestion was accepted, with the whole value behind the text. Fires after `oninput`."),
                    prop("filter", "Callback<AutocompleteFilterArgs<T>, bool>")
                        .default("contains")
                        .doc("Narrows `options`. Defaults to a case-insensitive `contains` over the label, ignoring spaces around the text."),
                    prop("prefiltered", "bool")
                        .default("false")
                        .doc("`options` arrives already narrowed, such as a list fetched per keystroke. Skips filtering, so `filter` never runs."),
                    prop("placeholder", "String").doc("Shown while the field is empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x at the end of the frame that empties the field."),
                    prop("empty", "Element")
                        .doc("Shown in place of the list when typed text matches nothing. Screen readers hear the localization's `combobox.nothing_found` either way, so change that string to match."),
                    prop("loading_label", "String")
                        .default("common.loading")
                        .doc("What screen readers hear while `options` is pending."),
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
                    status_prop(),
                    required_prop().also("Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool").default("false").doc("Takes the input out of the tab order and dims the field."),
                    readonly_prop("field"),
                    prop("dropdown_parts", "Parts<DropdownPart>").doc("Styles the portaled dropdown and its inner parts."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Leading, "The slot before the control: an icon, a prefix."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(SUGGESTION_DROPDOWN)),
                props("AutocompleteOptionArgs", vec![
                    prop("value", "T").doc("The suggestion this row draws."),
                    prop("index", "usize").doc("The row's position in the narrowed list."),
                ])
                .without_base_props(),
                props("AutocompleteFilterArgs", vec![
                    prop("value", "T").doc("The suggestion under test."),
                    prop("query", "String").doc("The text in the field."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Down"], "Opens the list. Typing opens it too.")
                .key(["Alt+Down"], "Opens the list without moving the highlight.")
                .key(["Up", "Down"], "Move the highlight.")
                .key(["Home", "End"], "Once a row is highlighted: move the highlight to the first or last row. Before that, they move the text caret.")
                .key(["PageUp", "PageDown"], "Open: moves the highlight 10 rows, stopping at the first or last.")
                .key(["Enter"], "Picks the highlighted row.")
                .key(["Escape", "Tab", "Alt+Up"], "Close the list.")
                .handles([
                    "Nothing is highlighted until you arrow onto a row, so Enter on text that matches nothing still submits the form.",
                    "Android's Back button closes the list as Escape does, rather than the app.",
                    "A polite status region says how many options the typed text left (`ComboboxLabels::results`, \"2 results\"), or \"No results\".",
                    CLEAR_NAME,
                ])
                .must(["Without a `label`, set `aria_label`. Otherwise screen readers announce an unnamed combobox."])
                .example("A city field, `Autocomplete { label: \"City\", .. }`: typing \"ber\" opens the list and the status says \"2 results\", Down highlights Berlin, and Enter picks it."),
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
                Text {
                    "The field turns the browser's saved entries off, so they do not cover the "
                    "list. You can pass your own "
                    Code { source: "autocomplete" }
                    " token, such as "
                    Code { source: "address-level2" }
                    " for a city, and the saved entries then cover the list."
                }
            },
            // snippet: let mut value = use_signal(String::new);
            // snippet: let mut picked = use_signal(|| None::<&'static str>);
            Demo {
                component: "Autocomplete",
                children_text: "",
                // `country` only exists in the snippet while the rows or the
                // pick handler call it.
                wrap: Wrap(|_values: &DemoValues, source: &str| {
                    format!("{}\n\n{source}", FILE.section("cities"))
                }),
                fixed: vec![
                    "sx: sx().width(\"280px\")".to_string(),
                    "options: cities()".to_string(),
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                    "placeholder: \"Start typing\"".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size").default(theme.autocomplete.size.as_str()),
                    Control::sizes("radius").default(theme.autocomplete.radius.as_str()),
                ], field_controls::<CityCopy>(), vec![
                    // A row drawn with `option`, and the `onpick` that reaches
                    // the record behind the text.
                    Control::switch("custom").code(|_, values| match custom(values) {
                        true => vec![
                            format!(
                                "option: move |o: AutocompleteOptionArgs<String>| {}",
                                FILE.section("row")
                            ),
                            format!("onpick: {}", FILE.section("onpick")),
                        ],
                        false => vec![],
                    }),
                    // Prefix instead of the default `contains`.
                    Control::switch("filter").code(|_, values| {
                        match values.str("filter").as_str() {
                            "true" => vec![format!(
                                "filter: move |f: AutocompleteFilterArgs<String>| {{\n    {}\n}}",
                                FILE.section("filter")
                            )],
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
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<CityCopy>(&values);
                    rsx! {
                    Autocomplete {
                        sx: sx().width("280px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: field.label,
                        aria_label: field.aria_label,
                        description: field.description,
                        helper: field
                            .helper
                            .or_else(|| picked().map(|country| format!("Picked a city in {country}."))),
                        status: field.status,
                        option: custom(&values).then(|| Callback::new(city_row)),
                        onpick: custom(&values).then(|| {
                            Callback::new(
                                // demo-code: onpick start
                                move |city: String| picked.set(Some(country(&city)))
                                // demo-code: onpick end
                            )
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
                }
                },
            }
        }
    }
}
