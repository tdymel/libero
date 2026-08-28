use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, FieldStatus, Flex, Options, Select, SelectOptionArgs, Text},
    sx::sx,
};

/// The enum is the option list, so the snippet has to show it.
const FRUIT_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    Mango,
    #[option(label = "Passion fruit")]
    Passion,
}

"#;

/// Only printed while the custom rows are on - the plain snippet never calls it.
const FRUIT_IMPL: &str = r#"impl Fruit {
    fn emoji(self) -> &'static str { /* "🍎", "🍌", ... */ }
    fn note(self) -> &'static str { /* "Crisp, keeps for weeks", ... */ }
}

"#;

const CUSTOM_OPTION: &str = r#"option: move |o: SelectOptionArgs<Fruit>| rsx! {
    Text { component: "span", size: "xl", "{o.value.emoji()}" }
    Flex {
        direction: "column",
        align: "flex-start",
        sx: sx().gap("0"),
        Text { component: "span", size: "sm", "{o.value.label()}" }
        Text { component: "span", size: "xs", sx: sx().color("grey.6"), "{o.value.note()}" }
    }
}"#;

const CUSTOM_SELECTION: &str =
    r#"selection: move |fruit: Fruit| rsx! { "{fruit.emoji()} {fruit.label()}" }"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    Mango,
    #[option(label = "Passion fruit")]
    Passion,
}

impl Fruit {
    fn emoji(self) -> &'static str {
        match self {
            Self::Apple => "🍎",
            Self::Banana => "🍌",
            Self::Cherry => "🍒",
            Self::Mango => "🥭",
            Self::Passion => "🟣",
        }
    }

    fn note(self) -> &'static str {
        match self {
            Self::Apple => "Crisp, keeps for weeks",
            Self::Banana => "Ripens on the counter",
            Self::Cherry => "In season for a fortnight",
            Self::Mango => "Ripe when it gives to a thumb",
            Self::Passion => "Wrinkled is ready",
        }
    }
}

fn custom(values: &DemoValues) -> bool {
    values.str("custom") == "true"
}

fn fruit_row(o: SelectOptionArgs<Fruit>) -> Element {
    rsx! {
        Text { component: "span", size: "xl", "{o.value.emoji()}" }
        Flex {
            direction: "column",
            align: "flex-start",
            sx: sx().gap("0"),
            Text { component: "span", size: "sm", "{o.value.label()}" }
            Text {
                component: "span",
                size: "xs",
                sx: sx().color("grey.6"),
                "{o.value.note()}"
            }
        }
    }
}

fn fruit_selection(fruit: Fruit) -> Element {
    rsx! { "{fruit.emoji()} {fruit.label()}" }
}

#[component]
pub fn SelectPage() -> Element {
    let mut value = use_signal(|| Some(Fruit::Banana));

    rsx! {
        DocPage {
            title: "Select",
            source: "libero/src/components/form/select/select.rs",
            markdown: "/md/select.md",
            properties: vec![
                props("Select", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, font size and the rows' size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list, independent of size."),
                    prop("value", "Option<T>")
                        .doc("The selected option; strictly controlled. `None` shows `placeholder`."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the option the caller should select next, or `None` when the clear button is clicked."),
                    prop("options", "Vec<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here."),
                    prop("option", "Callback<SelectOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. The row itself - highlight, `aria-selected`, click - stays the component's."),
                    prop("selection", "Callback<T, Element>")
                        .default("T::label()")
                        .doc("Draws the selected value inside the trigger."),
                    prop("placeholder", "String").doc("Shown while `value` is `None`."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron while something is selected. The only way `onchange` fires `None`."),
                    prop("searchable", "bool")
                        .default("false")
                        .doc("Puts a search box at the top of the list. It takes focus while the list is open, and the query is cleared when it closes."),
                    prop("filter", "Callback<SelectFilterArgs<T>, bool>")
                        .default("contains")
                        .doc("Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`."),
                    prop("search_placeholder", "String")
                        .doc("What the search box says while empty."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the trigger through `aria-labelledby`."),
                    prop("description", "Caption").doc("Between the label and the control: what to pick."),
                    prop("helper", "Caption").doc("Under the control: constraints, or what the choice affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A listbox over an enum, in the same field frame as every other input. Unlike "
                    Code { source: "NativeSelect" }
                    ", the rows are libero's own, so "
                    Code { source: "option" }
                    " can draw them with anything. Strictly controlled: "
                    Code { source: "value" }
                    " drives it, and "
                    Code { source: "clearable" }
                    " is what lets the user go back to "
                    Code { source: "None" }
                    "."
                }
            },
            Demo {
                component: "Select",
                children_text: "",
                // The helpers the custom rows call only exist in the snippet
                // while those rows are on.
                wrap: Wrap(|values: &DemoValues, source: &str| match custom(values) {
                    true => format!("{FRUIT_ENUM}{FRUIT_IMPL}{source}"),
                    false => format!("{FRUIT_ENUM}{source}"),
                }),
                fixed: vec![
                    "sx: sx().width(\"280px\")".to_string(),
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(next)".to_string(),
                    "placeholder: \"Pick a fruit\"".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"Out of season.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Pick a fruit.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Fruit\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Delivered with your next box.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"You can swap it until Friday.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    // Draws the rows and the trigger through `option` and
                    // `selection` - what `NativeSelect` cannot do at all.
                    Control::switch("custom").code(|_, values| match custom(values) {
                        true => vec![CUSTOM_OPTION.to_string(), CUSTOM_SELECTION.to_string()],
                        false => vec![],
                    }),
                    // The search box lives at the top of the list, so the
                    // trigger is unchanged and only an open list shows it.
                    Control::switch("searchable").code(|_, values| {
                        match values.str("searchable").as_str() {
                            "true" => vec![
                                "searchable: true".to_string(),
                                "search_placeholder: \"Search fruit\"".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Select {
                        sx: sx().width("280px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Fruit".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "Delivered with your next box.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "You can swap it until Friday.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Out of season.".to_string()),
                            "error" => FieldStatus::Error("Pick a fruit.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        option: custom(&values).then(|| Callback::new(fruit_row)),
                        selection: custom(&values).then(|| Callback::new(fruit_selection)),
                        searchable: (values.str("searchable") == "true").then_some(true),
                        search_placeholder: "Search fruit",
                        clearable: (values.str("clearable") == "true").then_some(true),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        placeholder: "Pick a fruit",
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
