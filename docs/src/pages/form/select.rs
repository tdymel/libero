use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Code, FieldStatus, Flex, OptionItem, OptionList, Options, Select, SelectFilterArgs,
        SelectOptionArgs, Text,
    },
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
// snippet: after FRUIT_ENUM
const FRUIT_IMPL: &str = r#"impl Fruit {
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

"#;

// snippet: after FRUIT_ENUM
// snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_OPTION: &str = r#"option: move |o: SelectOptionArgs<Fruit>| rsx! {
    Text { component: "span", size: "xl", "{o.value.emoji()}" }
    Flex {
        direction: "column",
        align: "flex-start",
        sx: sx().gap("0"),
        Text { component: "span", size: "sm", "{o.value.label()}" }
        Text { component: "span", size: "xs", sx: sx().color("muted.6"), "{o.value.note()}" }
    }
}"#;

// snippet: after FRUIT_ENUM
// snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_SELECTION: &str =
    r#"selection: move |fruit: Fruit| rsx! { "{fruit.emoji()} {fruit.label()}" }"#;

/// A filter can test anything the caller knows: this one searches the note too.
// snippet: after FRUIT_ENUM
// snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_FILTER: &str = r#"filter: move |f: SelectFilterArgs<Fruit>| {
    let query = f.query.to_lowercase();
    f.value.label().to_lowercase().contains(&query)
        || f.value.note().to_lowercase().contains(&query)
}"#;

/// Named runs, built explicitly: the caller is the only one who knows both
/// the order the groups go in and what each is called.
// snippet: after FRUIT_ENUM
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const GROUPED: &str = r#"options: OptionList::grouped()
    .group("Orchard", [Fruit::Apple, Fruit::Cherry])
    .group("Tropical", [Fruit::Banana, Fruit::Mango, Fruit::Passion])"#;

/// The flag sits on the option, in the same builder as the group - a row the
/// list refuses, rather than a value the type refuses everywhere.
// snippet: after FRUIT_ENUM
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const UNAVAILABLE: &str = r#"options: OptionList::new([
    Fruit::Apple.into(),
    Fruit::Banana.into(),
    OptionItem::new(Fruit::Cherry).disabled(true),
    Fruit::Mango.into(),
    Fruit::Passion.into(),
])"#;

/// Both at once, since the two switches share the one `options` prop.
// snippet: after FRUIT_ENUM
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const GROUPED_UNAVAILABLE: &str = r#"options: OptionList::grouped()
    .group("Orchard", [
        Fruit::Apple.into(),
        OptionItem::new(Fruit::Cherry).disabled(true),
    ])
    .group("Tropical", [Fruit::Banana, Fruit::Mango, Fruit::Passion])"#;

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

/// Needs `searchable` too: `filter` does nothing without a search box, and its value outlives
/// the hidden control.
fn filtering(values: &DemoValues) -> bool {
    values.str("filter") == "true" && values.str("searchable") == "true"
}

/// Matches the note as well as the label: "thumb" finds Mango.
fn fruit_filter(f: SelectFilterArgs<Fruit>) -> bool {
    let query = f.query.to_lowercase();
    f.value.label().to_lowercase().contains(&query)
        || f.value.note().to_lowercase().contains(&query)
}

/// The two switches drive one prop, so the list is built once from both.
/// Cherry is the out-of-season row, which is why `status: warning` says so.
fn fruit_options(values: &DemoValues) -> OptionList<Fruit> {
    let out_of_season = values.str("unavailable") == "true";
    let cherry = OptionItem::new(Fruit::Cherry).disabled(out_of_season);
    match values.str("grouped") == "true" {
        true => OptionList::grouped()
            .group("Orchard", [Fruit::Apple.into(), cherry])
            .group(
                "Tropical",
                [Fruit::Banana, Fruit::Mango, Fruit::Passion].map(OptionItem::new),
            ),
        false => OptionList::new([
            Fruit::Apple.into(),
            Fruit::Banana.into(),
            cherry,
            Fruit::Mango.into(),
            Fruit::Passion.into(),
        ]),
    }
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
                sx: sx().color("muted.6"),
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
                    prop("size", "Size").default("md").doc("Height, padding and font size of the field and its rows."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list."),
                    prop("value", "Option<T>")
                        .doc("The selected option. Pair it with `onchange`. `None` shows `placeholder`."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the option to select next, or `None` from the clear button."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("Posts the selected option's `Options::value()` under this name. A path such as `Order::FIELDS.plan()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the selection, shown once the select loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. A failed fetch is an empty list, so show your own error beside the field."),
                    prop("option", "Callback<SelectOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. The highlight, selection and click stay the component's."),
                    prop("selection", "Callback<T, Element>")
                        .default("T::label()")
                        .doc("Draws the selected value inside the trigger."),
                    prop("placeholder", "String").doc("Shown while `value` is `None`."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron while something is selected. The only way `onchange` gets `None`."),
                    prop("searchable", "bool")
                        .default("false")
                        .doc("Puts a search box at the top of the list. The query is cleared when the list closes."),
                    prop("filter", "Callback<SelectFilterArgs<T>, bool>")
                        .default("contains")
                        .doc("Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`."),
                    prop("search_placeholder", "String")
                        .doc("What the empty search box says."),
                    prop("label", "Caption")
                        .doc("The caption above the control, and the select's name."),
                    prop("description", "Caption").doc("Between the label and the control. What to pick."),
                    prop("helper", "Caption").doc("Under the control. Constraints, or what the choice changes."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Sets `aria-required` and marks the label."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` drops the select from the tab order and the post instead."),
                ]),
            ],
            accessibility: a11y()
                .key(["Down", "Up", "Enter", "Space"], "Closed: opens the list.")
                .key(["Home", "End"], "Closed: opens the list at the first or last row. Open: jumps to the first or last row.")
                .key(["Up", "Down"], "Open: move the highlight.")
                .key(["Enter", "Space"], "Open: picks the highlighted row.")
                .key(["Tab", "Alt+Up"], "Open: pick the highlighted row and close.")
                .key(["Escape"], "Open: closes without a pick.")
                .key(["Letter"], "Jumps to a matching label. \"b\", \"e\", \"r\" typed quickly finds Berlin, and a lone \"b\" after a pause cycles the rows starting with it. On a closed trigger this changes the value in place, as on a native `<select>`.")
                .handles([
                    "Disabled options are read out but skipped.",
                    "With `searchable` the search box takes over typing and holds the focus while the list is open.",
                ]),
            lead: rsx! {
                Text {
                    "A listbox over an enum, in the same frame as every other field. Unlike "
                    Code { source: "NativeSelect" }
                    ", the rows are libero's own, so "
                    Code { source: "option" }
                    " can draw them with anything. In exchange there is no OS picker on "
                    "phones. Pass "
                    Code { source: "value" }
                    " with "
                    Code { source: "onchange" }
                    ". "
                    Code { source: "clearable" }
                    " lets the user go back to "
                    Code { source: "None" }
                    "."
                }
            },
            // snippet: let mut value = use_signal(|| None::<Fruit>);
            Demo {
                component: "Select",
                children_text: "",
                // `filter` and the custom rows share helpers, printed for either switch.
                wrap: Wrap(|values: &DemoValues, source: &str| match custom(values)
                    || filtering(values)
                {
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
                        .labels(["Valid", "Warning", "Error"])
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
                            _ => vec!["aria_label: \"Fruit\"".to_string()],
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
                    // Off the table entirely without a search box: `filter`
                    // narrows what the box finds, so alone it does nothing.
                    Control::switch("filter")
                        .hidden_when(|values| values.str("searchable") != "true")
                        .code(|_, values| match filtering(values) {
                            true => vec![CUSTOM_FILTER.to_string()],
                            false => vec![],
                        }),
                    // Named runs, drawn as `role="group"` with a heading each.
                    Control::switch("grouped").code(|_, values| {
                        match (values.str("grouped").as_str(), values.str("unavailable").as_str()) {
                            ("true", "true") => vec![GROUPED_UNAVAILABLE.to_string()],
                            ("true", _) => vec![GROUPED.to_string()],
                            (_, "true") => vec![UNAVAILABLE.to_string()],
                            _ => vec![],
                        }
                    }),
                    // Prints through `grouped` above: the two share the one `options` prop.
                    Control::switch("unavailable").code(|_, _| vec![]),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                ],
                render: move |values: DemoValues| rsx! {
                    Select {
                        sx: sx().width("280px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Fruit".to_string()),
                        aria_label: (values.str("label") != "true").then_some("Fruit"),
                        description: (values.str("description") == "true")
                            .then(|| "Delivered with your next box.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "You can swap it until Friday.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Out of season.".to_string()),
                            "error" => FieldStatus::Error("Pick a fruit.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        options: fruit_options(&values),
                        option: custom(&values).then(|| Callback::new(fruit_row)),
                        selection: custom(&values).then(|| Callback::new(fruit_selection)),
                        searchable: (values.str("searchable") == "true").then_some(true),
                        search_placeholder: "Search fruit",
                        filter: filtering(&values).then(|| Callback::new(fruit_filter)),
                        clearable: (values.str("clearable") == "true").then_some(true),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        readonly: (values.str("readonly") == "true").then_some(true),
                        placeholder: "Pick a fruit",
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
