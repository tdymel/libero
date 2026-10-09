use super::CLEAR_NAME;
use super::dropdown_parts::{SELECT_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Control, Demo, DemoFile, DemoValues, DocPage, FieldCopy, Wrap, a11y, field_controls,
    field_props, indent, prop, props, readonly_prop, required_prop, status_prop,
};
use dioxus::prelude::*;

struct ToppingsCopy;

impl FieldCopy for ToppingsCopy {
    const LABEL: &'static str = "Toppings";
    const DESCRIPTION: &'static str = "Each one is at no extra cost.";
    const HELPER: &'static str = "Picked in the order they go on.";
    const WARNING: &'static str = "Pineapple divides the table.";
    const ERROR: &'static str = "Pick no more than one topping.";
}
use libero::components::SelectPart;
use libero::use_theme;
use libero::{
    components::{
        ActionIcon, Chip, MultiSelect, OptionItem, OptionList, Options, SelectFilterArgs,
        SelectOptionArgs, SelectionArgs, Text,
    },
    hooks::use_localization,
    localization::{Localization, fill},
    sx::sx,
};

/// The page's own source: the printed parts are cut from its live demo.
const FILE: DemoFile = DemoFile(include_str!("multi_select.rs"));

// demo-code: topping start
#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
    Onions,
    Peppers,
    Pineapple,
}
// demo-code: topping end

/// The two switches drive one prop, so the list is built once from both. Pineapple sells out,
/// matching `status: warning`.
fn topping_options(values: &DemoValues) -> OptionList<Topping> {
    match (
        values.str("grouped") == "true",
        values.str("sold_out") == "true",
    ) {
        (true, true) => {
            // demo-code: grouped_sold_out start
            OptionList::grouped()
                .group("Dairy", [Topping::Cheese])
                .group(
                    "Vegetables",
                    [
                        Topping::Mushrooms,
                        Topping::Olives,
                        Topping::Onions,
                        Topping::Peppers,
                    ],
                )
                .group(
                    "Fruit",
                    [OptionItem::new(Topping::Pineapple).disabled(true)],
                )
            // demo-code: grouped_sold_out end
        }
        (true, false) => {
            // Named runs, built explicitly: only the caller knows their order and names.
            // demo-code: grouped start
            OptionList::grouped()
                .group("Dairy", [Topping::Cheese])
                .group(
                    "Vegetables",
                    [
                        Topping::Mushrooms,
                        Topping::Olives,
                        Topping::Onions,
                        Topping::Peppers,
                    ],
                )
                .group("Fruit", [Topping::Pineapple])
            // demo-code: grouped end
        }
        (false, true) => {
            // The flag sits on the option: a row the list refuses, not a value the type refuses.
            // demo-code: sold_out start
            OptionList::new([
                OptionItem::new(Topping::Cheese),
                OptionItem::new(Topping::Mushrooms),
                OptionItem::new(Topping::Olives),
                OptionItem::new(Topping::Onions),
                OptionItem::new(Topping::Peppers),
                OptionItem::new(Topping::Pineapple).disabled(true),
            ])
            // demo-code: sold_out end
        }
        (false, false) => OptionList::from_options(),
    }
}

// demo-code: impl start
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
// demo-code: impl end

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
    // demo-code: filter start
    let query = f.query.to_lowercase();
    f.value.label().to_lowercase().contains(&query)
        || f.value.note().to_lowercase().contains(&query)
    // demo-code: filter end
}

/// A checkmark as well as the tint: `selected` is on the args for exactly this.
fn topping_row(o: SelectOptionArgs<Topping>) -> Element {
    // demo-code: row start
    rsx! {
        Text { component: "span", size: "lg", "aria-hidden": "true", "{o.value.emoji()}" }
        Text { component: "span", sx: sx().flex("1 1 auto"), "{o.value.label()}" }
        if o.selected {
            Text { component: "span", "aria-hidden": "true", "✓" }
        }
    }
    // demo-code: row end
}

/// The chip's inside is the caller's, the remove control with it - `remove` on
/// the args is the wiring. The keyboard stays `MultiSelect`'s either way.
fn topping_selection(words: &'static Localization, s: SelectionArgs<Topping>) -> Element {
    // demo-code: selection start
    rsx! {
        // Unclipped, so the x's 24px hit area reaches past the pill.
        Chip { size: "xs", variant: "outlined", sx: sx().overflow("visible"),
            // The chip is the caller's, remove control included. The keys stay
            // the control's either way.
            trailing: rsx! {
                // A disabled or read-only field's chip has no remove control.
                if !s.disabled && !s.readonly {
                    // Keeps focus in the field, and the click off the field.
                    span { onmousedown: move |event| event.prevent_default(),
                        onclick: move |event| event.stop_propagation(),
                        ActionIcon {
                            // `words` is `use_localization()`, read in the component.
                            aria_label: fill(words.common.remove, &[("label", &s.value.label())]),
                            size: "16px",
                            // A `<button>` inherits no colour of its own.
                            sx: sx().color("inherit"),
                            tabindex: "-1",
                            onclick: move |_| s.remove.call(()),
                            "x"
                        }
                    }
                }
            },
            span { "aria-hidden": "true", "{s.value.emoji()} " }
            "{s.value.label()}"
        }
    }
    // demo-code: selection end
}

/// The printed twin of the page's `value` signal below.
const VALUE_SIGNAL: &str = "let mut value = use_signal(|| vec![Topping::Cheese, Topping::Olives]);";

#[component]
pub fn MultiSelectPage() -> Element {
    let theme = use_theme();
    let mut value = use_signal(|| vec![Topping::Cheese, Topping::Olives]);
    let words = use_localization();

    rsx! {
        DocPage {
            title: "MultiSelect",
            source: "libero/src/components/form/select/multi_select.rs",
            markdown: "/md/multi_select.md",
            properties: vec![
                props("MultiSelect", vec![
                    prop("size", "Size").default(theme.multi_select.size.as_str()).doc("Height, padding and font size of the field and its rows."),
                    prop("radius", "Size")
                        .default(theme.multi_select.radius.as_str())
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
                        .doc("Draws one row's content. `selected` on the args is there for a checkmark. Hide a decorative glyph such as an emoji or the checkmark with `aria-hidden`, or a screen reader reads it with the label."),
                    prop("selection", "Callback<SelectionArgs<T>, Element>")
                        .default("Chip with an x")
                        .doc("Draws one selected value in the trigger, remove control included. `remove` on the args drops that value; `disabled` and `readonly` say the field refuses it, so leave the remove control out then."),
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
                    status_prop(),
                    required_prop().also("Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool").default("false").doc("Takes the trigger out of the tab order and dims the field."),
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
                    (SelectPart::Value, "The placeholder, in the trigger, while nothing is picked."),
                    (SelectPart::Chip, "A picked value's chip."),
                    (SelectPart::Helper, "The caption under the control."),
                    (SelectPart::Status, "The validation message."),
                ])
                .dropdown_parts("DropdownPart", list_dropdown_parts(SELECT_DROPDOWN)),
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
                props("SelectionArgs", vec![
                    prop("value", "T").doc("The value this chip draws."),
                    prop("remove", "Callback<()>").doc("Drops this value, the same edit as picking its row again."),
                    prop("disabled", "bool").doc("The field is disabled: `remove` does nothing, so draw no remove control."),
                    prop("readonly", "bool").doc("The field is read-only: `remove` does nothing, so draw no remove control."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Down", "Up", "Enter", "Space"], "Closed: opens the list.")
                .key(["Home", "End"], "Closed: opens the list at the first or last row.")
                .key(["Up", "Down"], "Open: move the highlight.")
                .key(["PageUp", "PageDown"], "Open: moves the highlight 10 rows, stopping at the first or last.")
                .key(["Enter", "Space"], "Open: toggles the row and keeps the list open.")
                .key(["Escape", "Tab", "Alt+Up"], "Open: close the list.")
                .key(["Left", "Right"], "Move over the chips.")
                .key(["Backspace", "Delete"], "Removes the chip you are on, or the last one. Inside the search box `Backspace` only edits the query.")
                .key(["Letter"], "Jumps to a matching label and opens the list there. Unlike on `Select`, it never changes the value in place, since a pick here toggles.")
                .handles([
                    "Disabled options are read out but skipped.",
                    "With `searchable` the search box takes over typing and holds the focus while the list is open.",
                    "Android's Back button closes the list as Escape does, rather than the app.",
                    "From 200 rows the list draws only the rows in view. The highlighted row stays drawn for `aria-activedescendant`, each row carries `aria-posinset` and `aria-setsize`, and a grouped row is described by its heading.",
                    CLEAR_NAME,
                ])
                .must(["Without a `label`, set `aria_label`. Otherwise screen readers announce an unnamed combobox."])
                .example("A tags field, `MultiSelect { label: \"Tags\", .. }`: Enter opens the list, Space ticks rows and keeps it open, and after closing it Left and Right move over the chips and Delete removes one."),
            lead: rsx! {
                Text {
                    "A listbox over an enum that holds any number of its options, drawn as chips "
                    "in the trigger. A pick toggles the row and the list stays open. Escape, a "
                    "click elsewhere or the trigger close it."
                }
            },
            Demo {
                component: "MultiSelect",
                children_text: "",
                // `filter` and the custom rows share one helper, printed for either switch.
                wrap: Wrap(|values: &DemoValues, source: &str| {
                    let state = match custom(values) {
                        true => format!("{VALUE_SIGNAL}\nlet words = use_localization();"),
                        false => VALUE_SIGNAL.to_string(),
                    };
                    match custom(values) || filtering(values) {
                        true => format!("{}\n\n{}\n\n{state}\n\n{source}", FILE.section("topping"), FILE.section("impl")),
                        false => format!("{}\n\n{state}\n\n{source}", FILE.section("topping")),
                    }
                }),
                fixed: vec![
                    "sx: sx().width(\"280px\")".to_string(),
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(next)".to_string(),
                    "placeholder: \"Pick toppings\"".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size").default(theme.multi_select.size.as_str()),
                    Control::sizes("radius").default(theme.multi_select.radius.as_str()),
                ], field_controls::<ToppingsCopy>(), vec![
                    // Draws the rows and the chips through `option` and
                    // `selection`.
                    Control::switch("custom").code(|_, values| match custom(values) {
                        true => vec![
                            format!("option: move |o: SelectOptionArgs<Topping>| {}", FILE.section("row")),
                            format!("selection: move |s: SelectionArgs<Topping>| {}", FILE.section("selection")),
                        ],
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
                            true => vec![format!(
                                "filter: move |f: SelectFilterArgs<Topping>| {{\n{}}}",
                                indent(&FILE.section("filter"))
                            )],
                            false => vec![],
                        }),
                    // Named runs, drawn as `role="group"` with a heading each.
                    Control::switch("grouped").code(|_, values| {
                        match (values.str("grouped").as_str(), values.str("sold_out").as_str()) {
                            ("true", "true") => vec!["grouped_sold_out"],
                            ("true", _) => vec!["grouped"],
                            (_, "true") => vec!["sold_out"],
                            _ => vec![],
                        }
                        .into_iter()
                        .map(|name| format!("options: {}", FILE.section(name)))
                        .collect()
                    }),
                    // Prints through `grouped` above: the two share the one `options` prop.
                    Control::switch("sold_out").code(|_, _| vec![]),
                    Control::switch("clearable"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<ToppingsCopy>(&values);
                    rsx! {
                    MultiSelect {
                        sx: sx().width("280px"),
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: field.label,
                        aria_label: field.aria_label,
                        description: field.description,
                        helper: field.helper,
                        status: field.status,
                        options: topping_options(&values),
                        option: custom(&values).then(|| Callback::new(topping_row)),
                        selection: custom(&values)
                            .then(|| Callback::new(move |s| topping_selection(words, s))),
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
                    }
                },
            }
        }
    }
}
