use super::dropdown_parts::{SELECT_DROPDOWN, list_dropdown_parts};
use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::SelectPart;
use libero::{
    components::{
        ActionIcon, Chip, FieldStatus, MultiSelect, OptionItem, OptionList, Options,
        SelectFilterArgs, SelectOptionArgs, SelectionArgs, Text,
    },
    sx::sx,
};

/// The enum is the option list, so the snippet has to show it.
const TOPPING_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
    Onions,
    Peppers,
    Pineapple,
}

"#;

/// Only printed while the custom rows are on - the plain snippet never calls it.
// snippet: after TOPPING_ENUM
const TOPPING_IMPL: &str = r#"impl Topping {
    fn emoji(self) -> &'static str {
        match self {
            Self::Cheese => "🧀",
            Self::Mushrooms => "🍄",
            Self::Olives => "🫒",
            Self::Onions => "🧅",
            Self::Peppers => "🫑",
            Self::Pineapple => "🍍",
        }
    }

    fn note(self) -> &'static str {
        match self {
            Self::Cheese => "Melts over everything",
            Self::Mushrooms => "Earthy, browns well",
            Self::Olives => "Salty, cures the dough",
            Self::Onions => "Sharp raw, sweet cooked",
            Self::Peppers => "Crisp, mild heat",
            Self::Pineapple => "Divides the table",
        }
    }
}

"#;

// snippet: after TOPPING_ENUM
// snippet: item impl Topping { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(Vec::<Topping>::new);
// snippet: in MultiSelect { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_OPTION: &str = r#"option: move |o: SelectOptionArgs<Topping>| rsx! {
    Text { component: "span", size: "lg", "{o.value.emoji()}" }
    Text { component: "span", sx: sx().flex("1 1 auto"), "{o.value.label()}" }
    if o.selected {
        Text { component: "span", "✓" }
    }
}"#;

// snippet: after TOPPING_ENUM
// snippet: item impl Topping { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(Vec::<Topping>::new);
// snippet: in MultiSelect { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_SELECTION: &str = r#"selection: move |s: SelectionArgs<Topping>| rsx! {
    Chip { size: "xs", variant: "outlined",
        // The chip is the caller's, remove control included. The keys stay
        // the control's either way.
        trailing: rsx! {
            span { onmousedown: move |event| event.prevent_default(),
                onclick: move |event| event.stop_propagation(),
                ActionIcon {
                    aria_label: "Remove {s.value.label()}",
                    size: "16px",
                    // A `<button>` inherits no colour of its own.
                    sx: sx().color("inherit"),
                    tabindex: "-1",
                    onclick: move |_| s.remove.call(()),
                    "x"
                }
            }
        },
        "{s.value.emoji()} {s.value.label()}"
    }
}"#;

/// A filter can test anything the caller knows: this one searches the note too.
// snippet: after TOPPING_ENUM
// snippet: item impl Topping { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let mut value = use_signal(Vec::<Topping>::new);
// snippet: in MultiSelect { value: value(), onchange: move |next| value.set(next), .. }
const CUSTOM_FILTER: &str = r#"filter: move |f: SelectFilterArgs<Topping>| {
    let query = f.query.to_lowercase();
    f.value.label().to_lowercase().contains(&query)
        || f.value.note().to_lowercase().contains(&query)
}"#;

/// Named runs, built explicitly: the caller is the only one who knows both
/// the order the groups go in and what each is called.
// snippet: after TOPPING_ENUM
// snippet: let mut value = use_signal(Vec::<Topping>::new);
// snippet: in MultiSelect { value: value(), onchange: move |next| value.set(next), .. }
const GROUPED: &str = r#"options: OptionList::grouped()
    .group("Dairy", [Topping::Cheese])
    .group("Vegetables", [Topping::Mushrooms, Topping::Olives, Topping::Onions, Topping::Peppers])
    .group("Fruit", [Topping::Pineapple])"#;

/// The flag sits on the option, in the same builder as the group - a row the
/// list refuses, rather than a value the type refuses everywhere.
// snippet: after TOPPING_ENUM
// snippet: let mut value = use_signal(Vec::<Topping>::new);
// snippet: in MultiSelect { value: value(), onchange: move |next| value.set(next), .. }
const SOLD_OUT: &str = r#"options: OptionList::new([
    Topping::Cheese.into(),
    Topping::Mushrooms.into(),
    Topping::Olives.into(),
    Topping::Onions.into(),
    Topping::Peppers.into(),
    OptionItem::new(Topping::Pineapple).disabled(true),
])"#;

/// Both at once, since the two switches share the one `options` prop.
// snippet: after TOPPING_ENUM
// snippet: let mut value = use_signal(Vec::<Topping>::new);
// snippet: in MultiSelect { value: value(), onchange: move |next| value.set(next), .. }
const GROUPED_SOLD_OUT: &str = r#"options: OptionList::grouped()
    .group("Dairy", [Topping::Cheese])
    .group("Vegetables", [Topping::Mushrooms, Topping::Olives, Topping::Onions, Topping::Peppers])
    .group("Fruit", [OptionItem::new(Topping::Pineapple).disabled(true)])"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
    Onions,
    Peppers,
    Pineapple,
}

/// The two switches drive one prop, so the list is built once from both. Pineapple sells out,
/// matching `status: warning`.
fn topping_options(values: &DemoValues) -> OptionList<Topping> {
    let pineapple = OptionItem::new(Topping::Pineapple).disabled(values.str("sold_out") == "true");
    let vegetables = [
        Topping::Mushrooms,
        Topping::Olives,
        Topping::Onions,
        Topping::Peppers,
    ];
    match values.str("grouped") == "true" {
        true => OptionList::grouped()
            .group("Dairy", [Topping::Cheese])
            .group("Vegetables", vegetables)
            .group("Fruit", [pineapple]),
        false => OptionList::new(
            [Topping::Cheese]
                .into_iter()
                .chain(vegetables)
                .map(OptionItem::new)
                .chain([pineapple]),
        ),
    }
}

impl Topping {
    fn emoji(self) -> &'static str {
        match self {
            Self::Cheese => "🧀",
            Self::Mushrooms => "🍄",
            Self::Olives => "🫒",
            Self::Onions => "🧅",
            Self::Peppers => "🫑",
            Self::Pineapple => "🍍",
        }
    }

    /// Not drawn on the row - it exists so the `filter` switch has something to
    /// match that the row itself never shows.
    fn note(self) -> &'static str {
        match self {
            Self::Cheese => "Melts over everything",
            Self::Mushrooms => "Earthy, browns well",
            Self::Olives => "Salty, cures the dough",
            Self::Onions => "Sharp raw, sweet cooked",
            Self::Peppers => "Crisp, mild heat",
            Self::Pineapple => "Divides the table",
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

/// Matches the note as well as the label: "earthy" finds Mushrooms.
fn topping_filter(f: SelectFilterArgs<Topping>) -> bool {
    let query = f.query.to_lowercase();
    f.value.label().to_lowercase().contains(&query)
        || f.value.note().to_lowercase().contains(&query)
}

/// A checkmark as well as the tint: `selected` is on the args for exactly this.
fn topping_row(o: SelectOptionArgs<Topping>) -> Element {
    rsx! {
        Text { component: "span", size: "lg", "{o.value.emoji()}" }
        Text { component: "span", sx: sx().flex("1 1 auto"), "{o.value.label()}" }
        if o.selected {
            Text { component: "span", "✓" }
        }
    }
}

/// The chip's inside is the caller's, the remove control with it - `remove` on
/// the args is the wiring. The keyboard stays `MultiSelect`'s either way.
fn topping_selection(s: SelectionArgs<Topping>) -> Element {
    let label = s.value.label();
    rsx! {
        Chip { size: "xs", variant: "outlined",
            trailing: rsx! {
                span {
                    onmousedown: move |event: MouseEvent| event.prevent_default(),
                    onclick: move |event: MouseEvent| event.stop_propagation(),
                    ActionIcon {
                        aria_label: "Remove {label}",
                        size: "16px",
                        sx: sx().color("inherit"),
                        tabindex: "-1",
                        onclick: move |_| s.remove.call(()),
                        "x"
                    }
                }
            },
            "{s.value.emoji()} {label}"
        }
    }
}

#[component]
pub fn MultiSelectPage() -> Element {
    let mut value = use_signal(|| vec![Topping::Cheese, Topping::Olives]);

    rsx! {
        DocPage {
            title: "MultiSelect",
            source: "libero/src/components/form/select/multi_select.rs",
            markdown: "/md/multi_select.md",
            properties: vec![
                props("MultiSelect", vec![
                    prop("size", "Size").default("md").doc("Height, padding and font size of the field and its rows."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list."),
                    prop("value", "Vec<T>")
                        .doc("The selection, in the order it was picked. Pair it with `onchange`. Empty shows `placeholder`."),
                    prop("onchange", "EventHandler<Vec<T>>")
                        .doc("Called with the whole next selection."),
                    prop("name", "FieldName<Vec<T>>")
                        .doc("Posts each selected option's `Options::value()` under this name. A path such as `Order::FIELDS.toppings()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Vec<T>>")
                        .doc("Rules over the selection, shown once the select loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set goes here. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches."),
                    prop("option", "Callback<SelectOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. `selected` on the args is there for a checkmark."),
                    prop("selection", "Callback<SelectionArgs<T>, Element>")
                        .default("Chip with an x")
                        .doc("Draws one selected value in the trigger, remove control included. `remove` on the args drops that value."),
                    prop("placeholder", "String").doc("Shown while `value` is empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron that empties the selection."),
                    prop("searchable", "bool")
                        .default("false")
                        .doc("Puts a search box at the top of the list. The query survives a pick and is cleared when the list closes."),
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
                    prop("dropdown_parts", "Parts<DropdownPart>").doc("Styles the portaled dropdown and its inner parts."),
                ])
                .parts("SelectPart", vec![
                    (SelectPart::Label, "The label above the control."),
                    (SelectPart::Required, "The required asterisk, in the label."),
                    (SelectPart::Description, "The caption between the label and the control."),
                    (SelectPart::Frame, "The bordered box around the control."),
                    (SelectPart::Control, "The element the label names."),
                    (SelectPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (SelectPart::Value, "The placeholder, in the trigger, while nothing is picked."),
                    (SelectPart::Chip, "A picked value's chip."),
                    (SelectPart::Helper, "The caption under the control."),
                    (SelectPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(SELECT_DROPDOWN)),
            ],
            accessibility: a11y()
                .key(["Down", "Up", "Enter", "Space"], "Closed: opens the list.")
                .key(["Home", "End"], "Closed: opens the list at the first or last row.")
                .key(["Up", "Down"], "Open: move the highlight.")
                .key(["Enter", "Space"], "Open: toggles the row and keeps the list open.")
                .key(["Escape", "Tab", "Alt+Up"], "Open: close the list.")
                .key(["Left", "Right"], "Move over the chips.")
                .key(["Backspace", "Delete"], "Removes the chip you are on, or the last one. Inside the search box `Backspace` only edits the query.")
                .key(["Letter"], "Jumps to a matching label and opens the list there. Unlike on `Select`, it never changes the value in place, since a pick here toggles.")
                .handles([
                    "Disabled options are read out but skipped.",
                    "With `searchable` the search box takes over typing and holds the focus while the list is open.",
                ]),
            lead: rsx! {
                Text {
                    "A listbox over an enum that holds any number of its options, drawn as chips "
                    "in the trigger. A pick toggles the row and the list stays open. Escape, a "
                    "click elsewhere or the trigger close it."
                }
            },
            // snippet: let mut value = use_signal(Vec::<Topping>::new);
            Demo {
                component: "MultiSelect",
                children_text: "",
                // `filter` and the custom rows share one helper, printed for either switch.
                wrap: Wrap(|values: &DemoValues, source: &str| match custom(values)
                    || filtering(values)
                {
                    true => format!("{TOPPING_ENUM}{TOPPING_IMPL}{source}"),
                    false => format!("{TOPPING_ENUM}{source}"),
                }),
                fixed: vec![
                    "sx: sx().width(\"280px\")".to_string(),
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(next)".to_string(),
                    "placeholder: \"Pick toppings\"".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .labels(["Valid", "Warning", "Error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"Pineapple divides the table.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Pick at least one.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Toppings\"".to_string()],
                            _ => vec!["aria_label: \"Toppings\"".to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Up to five, at no extra cost.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"Picked in the order they go on.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    // Draws the rows and the chips through `option` and
                    // `selection`.
                    Control::switch("custom").code(|_, values| match custom(values) {
                        true => vec![CUSTOM_OPTION.to_string(), CUSTOM_SELECTION.to_string()],
                        false => vec![],
                    }),
                    // The query survives a pick here, so several matches of one
                    // search can be ticked without retyping it.
                    Control::switch("searchable").code(|_, values| {
                        match values.str("searchable").as_str() {
                            "true" => vec![
                                "searchable: true".to_string(),
                                "search_placeholder: \"Search toppings\"".to_string(),
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
                        match (values.str("grouped").as_str(), values.str("sold_out").as_str()) {
                            ("true", "true") => vec![GROUPED_SOLD_OUT.to_string()],
                            ("true", _) => vec![GROUPED.to_string()],
                            (_, "true") => vec![SOLD_OUT.to_string()],
                            _ => vec![],
                        }
                    }),
                    // Prints through `grouped` above: the two share the one `options` prop.
                    Control::switch("sold_out").code(|_, _| vec![]),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    MultiSelect {
                        sx: sx().width("280px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Toppings".to_string()),
                        aria_label: (values.str("label") != "true").then_some("Toppings"),
                        description: (values.str("description") == "true")
                            .then(|| "Up to five, at no extra cost.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "Picked in the order they go on.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Pineapple divides the table.".to_string()),
                            "error" => FieldStatus::Error("Pick at least one.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        options: topping_options(&values),
                        option: custom(&values).then(|| Callback::new(topping_row)),
                        selection: custom(&values).then(|| Callback::new(topping_selection)),
                        searchable: (values.str("searchable") == "true").then_some(true),
                        search_placeholder: "Search toppings",
                        filter: filtering(&values).then(|| Callback::new(topping_filter)),
                        clearable: (values.str("clearable") == "true").then_some(true),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        placeholder: "Pick toppings",
                        value: value(),
                        onchange: move |next| value.set(next),
                    }
                },
            }
        }
    }
}
