use super::dropdown_parts::{SELECT_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls, field_props, prop,
    props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::SelectPart;
use libero::use_theme;
use libero::{
    choices,
    components::{
        Choices, Code, Flex, OptionItem, OptionList, Options, Select, SelectFilterArgs,
        SelectOptionArgs, Text,
    },
    sx::sx,
};

struct FruitCopy;

impl FieldCopy for FruitCopy {
    const LABEL: &'static str = "Fruit";
    const DESCRIPTION: &'static str = "Delivered with your next box.";
    const HELPER: &'static str = "You can swap it until Friday.";
    const WARNING: &'static str = "Out of season.";
    const ERROR: &'static str = "Pick a fruit.";
}

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
    Text { component: "span", size: "xl", "aria-hidden": "true", "{o.value.emoji()}" }
    Flex {
        direction: "column",
        align: "flex-start",
        sx: sx().gap("0"),
        Text { component: "span", size: "sm", "{o.value.label()}" }
        Text { component: "span", size: "xs", sx: sx().color("muted.7"), "{o.value.note()}" }
    }
}"#;

// snippet: after FRUIT_ENUM
// snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(|| None::<Fruit>);
// snippet: in Select { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_SELECTION: &str = r#"selection: move |fruit: Fruit| rsx! { span { "aria-hidden": "true", "{fruit.emoji()} " } "{fruit.label()}" }"#;

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

/// Printed in place of `FRUIT_ENUM` while `choices` is on: an enum from another crate.
// snippet: ignore - its states hold a `Choice<Fruit>`, which the Demo's one `let` marker types as `Fruit`; `choices!` compiles in libero's doc-tests
const FOREIGN_FRUIT: &str = r#"use libero::{choices, components::Choices};

// From a domain crate, so no `Options` derive.
#[derive(Clone, Copy, PartialEq)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    Mango,
    Passion,
}

// Without a `_` arm, a new variant fails the build here.
fn fruit_label(fruit: &Fruit) -> String {
    match fruit {
        Fruit::Apple => "Apple",
        Fruit::Banana => "Banana",
        Fruit::Cherry => "Cherry",
        Fruit::Mango => "Mango",
        Fruit::Passion => "Passion fruit",
    }
    .into()
}

// In display order. The keys are what a form posts; a missing variant fails the build.
static FRUITS: Choices<Fruit> = choices!(Fruit, fruit_label, {
    Apple => "apple",
    Banana => "banana",
    Cherry => "cherry",
    Mango => "mango",
    Passion => "passion",
});

"#;

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

fn fruit_label(fruit: &Fruit) -> String {
    match fruit {
        Fruit::Apple => "Apple",
        Fruit::Banana => "Banana",
        Fruit::Cherry => "Cherry",
        Fruit::Mango => "Mango",
        Fruit::Passion => "Passion fruit",
    }
    .into()
}

static FRUITS: Choices<Fruit> = choices!(Fruit, fruit_label, {
    Apple => "apple",
    Banana => "banana",
    Cherry => "cherry",
    Mango => "mango",
    Passion => "passion",
});

/// The list as `Choice<Fruit>`s; the rows and groups below are for the derived enum only.
fn foreign(values: &DemoValues) -> bool {
    values.str("choices") == "true"
}

fn custom(values: &DemoValues) -> bool {
    values.str("custom") == "true" && !foreign(values)
}

/// Needs `searchable` too: `filter` does nothing without a search box, and its value outlives
/// the hidden control.
fn filtering(values: &DemoValues) -> bool {
    values.str("filter") == "true" && values.str("searchable") == "true" && !foreign(values)
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
        Text { component: "span", size: "xl", "aria-hidden": "true", "{o.value.emoji()}" }
        Flex {
            direction: "column",
            align: "flex-start",
            sx: sx().gap("0"),
            Text { component: "span", size: "sm", "{o.value.label()}" }
            Text {
                component: "span",
                size: "xs",
                // Shade 6 is 2.8:1 on the active row; 7 passes on it in both schemes.
                sx: sx().color("muted.7"),
                "{o.value.note()}"
            }
        }
    }
}

fn fruit_selection(fruit: Fruit) -> Element {
    rsx! {
        span { "aria-hidden": "true", "{fruit.emoji()} " }
        "{fruit.label()}"
    }
}

#[component]
pub fn SelectPage() -> Element {
    let theme = use_theme();
    let mut value = use_signal(|| Some(Fruit::Banana));
    let mut picked = use_signal(|| FRUITS.choice(&Fruit::Banana));

    rsx! {
        DocPage {
            title: "Select",
            source: "libero/src/components/form/select/select.rs",
            markdown: "/md/select.md",
            properties: vec![
                props("Select", vec![
                    prop("size", "Size").default(theme.select.size.as_str()).doc("Height, padding and font size of the field and its rows."),
                    prop("radius", "ThemeAwareValue")
                        .default(theme.select.radius.as_str())
                        .doc("Corner radius of the frame and the list. Or any CSS, e.g. `radius: \"0\"`."),
                    prop("value", "Option<T>")
                        .doc("The selected option. Pair it with `onchange`. `None`, or a value no option matches, shows `placeholder`. A bare `None` names no `T`: write `None::<T>`."),
                    prop("onchange", "EventHandler<Option<T>>")
                        .doc("Called with the option to select next, or `None` from the clear button."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("Posts the selected option's `Options::value()` under this name: for a derived enum the variant's name, never the label. A path such as `Order::FIELDS.plan()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the selection, shown once the select loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. An enum from another crate takes a `Choices` list built by `choices!`, with `T` a `Choice<Fruit>` that derefs to the `Fruit`. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. A failed fetch is an empty list, so show your own error beside the field. Order is kept, so a group label used again after another group draws its heading again."),
                    prop("option", "Callback<SelectOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. The highlight, selection and click stay the component's. Hide a decorative glyph such as an emoji with `aria-hidden`, or a screen reader reads it before the label."),
                    prop("selection", "Callback<T, Element>")
                        .default("T::label()")
                        .doc("Draws the selected value inside the trigger."),
                    prop("placeholder", "String").doc("Shown while no option matches `value`."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron while something is selected. The only way `onchange` gets `None`."),
                    prop("searchable", "bool")
                        .default("false")
                        .doc("Puts a search box at the top of the list. The query is cleared when the list closes."),
                    prop("open_on_focus", "bool")
                        .default("false")
                        .doc("Opens the list when the field takes focus, by Tab or a press. Focus coming back after a pick or Escape leaves it closed."),
                    prop("filter", "Callback<SelectFilterArgs<T>, bool>")
                        .default("contains")
                        .doc("Narrows the options while searching. Defaults to a case-insensitive `contains` over `Options::label`. It replaces that test, so a search can match a synonym or a code."),
                    prop("search_placeholder", "String")
                        .doc("What the empty search box says."),
                    prop("label", "Caption")
                        .doc("The caption above the control, and the select's name."),
                    prop("description", "Caption").doc("Between the label and the control. What to pick."),
                    prop("helper", "Caption").doc("Under the control. Constraints, or what the choice changes."),
                    status_prop(),
                    required_prop().also("Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order, dims the field and posts nothing."),
                    readonly_prop("select"),
                    prop("dropdown_parts", "Parts<DropdownPart>").doc("Styles the portaled dropdown and its inner parts."),
                ])
                .parts("SelectPart", vec![
                    (SelectPart::Label, "The label above the control."),
                    (SelectPart::Required, "The required asterisk, in the label."),
                    (SelectPart::Description, "The caption between the label and the control."),
                    (SelectPart::Frame, "The bordered box around the control."),
                    (SelectPart::Control, "The element the label names."),
                    (SelectPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (SelectPart::Value, "The picked value or the placeholder, in the trigger."),
                    (SelectPart::Helper, "The caption under the control."),
                    (SelectPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(SELECT_DROPDOWN)),
                props("Domain enums", vec![
                    prop(".choices(&X)", "Mapped<V, Choice<V>, Infallible>")
                        .doc("On a path to `Option<V>`: binds `name` to a domain enum through a `Choices<V>` list, so the form's value holds `V` and the field edits `Choice<V>`. Pass `X.list()` as `options` too. A stored value the list no longer holds shows the placeholder."),
                ]).without_base_props(),
                props("SelectOptionArgs", vec![
                    prop("value", "T").doc("The option this row draws."),
                    prop("index", "usize").doc("The row's position among the rows drawn."),
                    prop("selected", "bool").doc("Part of the selection, for a checkmark."),
                    prop("disabled", "bool").doc("The list refuses this row. The greying and `aria-disabled` are drawn anyway."),
                ])
                .without_base_props(),
                props("SelectFilterArgs", vec![
                    prop("value", "T").doc("The option under test."),
                    prop("query", "String").doc("What is typed in the search box."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Down", "Up", "Enter", "Space"], "Closed: opens the list.")
                .key(["Home", "End"], "Closed: opens the list at the first or last row. Open: jumps to the first or last row.")
                .key(["Up", "Down"], "Open: move the highlight.")
                .key(["PageUp", "PageDown"], "Open: moves the highlight 10 rows, stopping at the first or last.")
                .key(["Enter", "Space"], "Open: picks the highlighted row.")
                .key(["Tab", "Alt+Up"], "Open: pick the highlighted row and close.")
                .key(["Escape"], "Open: closes without a pick.")
                .key(["Letter"], "Jumps to a matching label. \"b\", \"e\", \"r\" typed quickly finds Berlin, and a lone \"b\" after a pause cycles the rows starting with it. On a closed trigger this changes the value in place, as on a native `<select>`.")
                .handles([
                    "Disabled options are read out but skipped.",
                    "With `searchable` the search box takes over typing and holds the focus while the list is open.",
                    "Android's Back button closes the list as Escape does, rather than the app.",
                    "From 200 rows the list draws only the rows in view. The highlighted row stays drawn for `aria-activedescendant`, each row carries `aria-posinset` and `aria-setsize` over the whole list, and a grouped row is described by its heading.",
                    "Each group is a `role=\"group\"` named by its heading.",
                    clear_name!("Fruit"),
                ])
                .must(["Without a `label`, set `aria_label`. Otherwise screen readers announce an unnamed combobox."])
                .example("A country field, `Select { label: \"Country\", .. }`: on the closed field, typing \"ger\" picks Germany in place as a native select does, and Enter opens the list for browsing."),
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
                    ". Below the "
                    Code { source: "sm" }
                    " breakpoint the list opens as a sheet at the foot of the screen, and a "
                    "touch leaves the keyboard down until the search box is tapped. For free "
                    "text, use "
                    Code { source: "Autocomplete" }
                    "."
                }
            },
            // snippet: let mut value = use_signal(|| None::<Fruit>);
            Demo {
                component: "Select",
                children_text: "",
                // `filter` and the custom rows share helpers, printed for either switch.
                wrap: Wrap(|values: &DemoValues, source: &str| {
                    match (foreign(values), custom(values) || filtering(values)) {
                        (true, _) => format!("{FOREIGN_FRUIT}{source}"),
                        (false, true) => format!("{FRUIT_ENUM}{FRUIT_IMPL}{source}"),
                        (false, false) => format!("{FRUIT_ENUM}{source}"),
                    }
                }),
                fixed: vec![
                    "sx: sx().width(\"280px\").max_width(\"100%\")".to_string(),
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(next)".to_string(),
                    "placeholder: \"Pick a fruit\"".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size").default(theme.select.size.as_str()),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"]).default(theme.select.radius.as_str()),
                ], field_controls::<FruitCopy>(), vec![
                    // Draws the rows and the trigger through `option` and
                    // `selection` - what `NativeSelect` cannot do at all.
                    Control::switch("custom").hidden_when(foreign).code(|_, values| match custom(values) {
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
                    Control::switch("open_on_focus").code(|_, values| match values.str("open_on_focus").as_str() {
                        "true" => vec!["open_on_focus: true".to_string()],
                        _ => vec![],
                    }),
                    // Off the table entirely without a search box: `filter`
                    // narrows what the box finds, so alone it does nothing.
                    Control::switch("filter")
                        .hidden_when(|values| values.str("searchable") != "true" || foreign(values))
                        .code(|_, values| match filtering(values) {
                            true => vec![CUSTOM_FILTER.to_string()],
                            false => vec![],
                        }),
                    // Named runs, drawn as `role="group"` with a heading each.
                    Control::switch("grouped").hidden_when(foreign).code(|_, values| {
                        if foreign(values) {
                            return vec![];
                        }
                        match (values.str("grouped").as_str(), values.str("unavailable").as_str()) {
                            ("true", "true") => vec![GROUPED_UNAVAILABLE.to_string()],
                            ("true", _) => vec![GROUPED.to_string()],
                            (_, "true") => vec![UNAVAILABLE.to_string()],
                            _ => vec![],
                        }
                    }),
                    // Prints through `grouped` above: the two share the one `options` prop.
                    Control::switch("unavailable").code(|_, _| vec![]),
                    // A foreign enum's list, from `choices!`; `unavailable` disables through it.
                    Control::switch("choices").code(|_, values| {
                        match (foreign(values), values.str("unavailable") == "true") {
                            (true, true) => vec!["options: FRUITS.list().disabling(|fruit| **fruit == Fruit::Cherry)".to_string()],
                            (true, false) => vec!["options: FRUITS.list()".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<FruitCopy>(&values);
                    if foreign(&values) {
                        let out_of_season = values.str("unavailable") == "true";
                        return rsx! {
                            Select {
                                sx: sx().width("280px").max_width("100%"),
                                size: values.str("size"),
                                radius: values.str("radius"),
                                label: field.label,
                                aria_label: field.aria_label,
                                description: field.description,
                                helper: field.helper,
                                status: field.status,
                                options: FRUITS.list().disabling(|fruit| out_of_season && **fruit == Fruit::Cherry),
                                searchable: (values.str("searchable") == "true").then_some(true),
                                open_on_focus: values.str("open_on_focus") == "true",
                                search_placeholder: "Search fruit",
                                clearable: (values.str("clearable") == "true").then_some(true),
                                required: (values.str("required") == "true").then_some(true),
                                disabled: (values.str("disabled") == "true").then_some(true),
                                readonly: (values.str("readonly") == "true").then_some(true),
                                placeholder: "Pick a fruit",
                                value: picked(),
                                onchange: move |next| picked.set(next),
                            }
                        };
                    }
                    rsx! {
                    Select {
                        sx: sx().width("280px").max_width("100%"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: field.label,
                        aria_label: field.aria_label,
                        description: field.description,
                        helper: field.helper,
                        status: field.status,
                        options: fruit_options(&values),
                        option: custom(&values).then(|| Callback::new(fruit_row)),
                        selection: custom(&values).then(|| Callback::new(fruit_selection)),
                        searchable: (values.str("searchable") == "true").then_some(true),
                        open_on_focus: values.str("open_on_focus") == "true",
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
                }
                },
            }
        }
    }
}
