use std::time::Duration;

use super::dropdown_parts::{COMBOBOX_DROPDOWN, list_dropdown_parts};
use crate::components::{Child, Control, Demo, DemoValues, DocPage, Wrap, a11y, prop, props};
use dioxus::prelude::*;
use libero::use_theme;
use libero::{
    components::{
        Button, Code, Combobox, ComboboxOption, ComboboxOptionArgs, Flex, OptionList, Options,
        Text, TextField, use_combobox,
    },
    platform::{TimerSubscription, timer},
    sx::sx,
};

/// The fake server the preview asks, printed so the snippet calls nothing it
/// does not show.
// snippet: after FRUIT_ENUM
const FETCHING_SEARCH: &str = r#"/// How long the fake search takes.
const LATENCY: Duration = Duration::from_millis(700);

/// The fruit whose label contains `query`, case-insensitively.
fn matching(query: &str) -> Vec<Fruit> {
    let query = query.to_lowercase();
    Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| fruit.label().to_lowercase().contains(&query))
        .collect()
}

"#;

/// A fetch per keystroke. `None` is the search in flight, and it is what keeps
/// `empty` from flashing between the keystroke and the answer.
// snippet: after FRUIT_ENUM
const FETCHING_STATE: &str = r#"let suggestions = use_combobox();
let mut text = use_signal(String::new);
// One signal, not a list plus a `loading` flag: `None` *is* the search in
// flight, so the two can never disagree.
let mut results = use_signal(|| Some(Vec::<Fruit>::new()));
// The answer still on its way. Replacing it drops the older one, so a slow
// answer never overwrites a newer query's.
let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
use_drop(move || pending.set(None));

"#;

// snippet: after FRUIT_ENUM, FETCHING_SEARCH, FETCHING_STATE
const FETCHING_TRIGGER: &str = r#"TextField {
    sx: sx().width("280px"),
    label: "Fruit",
    placeholder: "Type a fruit",
    value: text(),
    attributes: suggestions.a11y_attributes(),
    onblur: move |_| suggestions.close(),
    oninput: move |next: String| {
        text.set(next.clone());
        suggestions.open();
        results.set(None);
        let answer = timer().map(|timer| {
            timer.after(LATENCY, Box::new(move || results.set(Some(matching(&next)))))
        });
        pending.set(answer);
    },
}"#;

/// How long the fake search takes - long enough to see, short enough to type
/// through.
const LATENCY: Duration = Duration::from_millis(700);

/// The enum is the option list, so every snippet has to show it.
const FRUIT_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    #[option(label = "Dragon fruit")]
    Dragon,
    Elderberry,
    Mango,
    Grape,
}

"#;

// snippet: after FRUIT_ENUM
const SELECT_STATE: &str = r#"let fruit = use_combobox();
let mut picked = use_signal(|| None::<Fruit>);
// A `combobox` role takes no name from its content, so the label names it.
let label = format!("{}-label", fruit.id());

"#;

// snippet: after FRUIT_ENUM
const SUGGESTIONS_STATE: &str = r#"let suggestions = use_combobox();
let mut text = use_signal(String::new);

let matches: Vec<Fruit> = Fruit::options()
    .iter()
    .copied()
    .filter(|fruit| fruit.label().to_lowercase().contains(&text().to_lowercase()))
    .collect();

"#;

// snippet: after FRUIT_ENUM, SELECT_STATE
const SELECT_TRIGGER: &str = r#"Text { id: "{label}", size: "sm", "Fruit" }
Button {
    variant: "outlined",
    sx: sx().width("280px"),
    attributes: [
        fruit.a11y_attributes(),
        vec![Attribute::new("aria-labelledby", label, None, false)],
    ]
    .concat(),
    onclick: move |_| fruit.toggle(),
    onblur: move |_| fruit.close(),
    match picked() {
        Some(fruit) => rsx! { "{fruit.label()}" },
        None => rsx! { "Pick a fruit" },
    }
}"#;

// snippet: after FRUIT_ENUM, SUGGESTIONS_STATE
const SUGGESTIONS_TRIGGER: &str = r#"TextField {
    sx: sx().width("280px"),
    label: "Fruit",
    placeholder: "Type a fruit",
    value: text(),
    attributes: suggestions.a11y_attributes(),
    onblur: move |_| suggestions.close(),
    oninput: move |next| {
        text.set(next);
        suggestions.open();
    },
}
input { r#type: "hidden", name: "fruit", value: "{text()}" }"#;

/// A row is four combinations of two choices, so the snippet is composed
/// rather than written out four times.
// snippet: after FRUIT_ENUM, SELECT_STATE
// snippet: in Combobox { state: fruit, options: Fruit::options().to_vec(), option: move |o: ComboboxOptionArgs<Fruit>| rsx! { ComboboxOption { .. "{o.value.label()}" } } }
const SELECT_WIRING: &str = r#"        selected: picked() == Some(o.value),
        onpick: move |_| {
            picked.set(Some(o.value));
            fruit.close();
        },"#;

/// No `selected`: a suggestion is not a selection.
// snippet: after FRUIT_ENUM, SUGGESTIONS_STATE
// snippet: in Combobox { state: suggestions, options: matches, option: move |o: ComboboxOptionArgs<Fruit>| rsx! { ComboboxOption { .. "{o.value.label()}" } } }
const SUGGESTION_WIRING: &str = r#"        onpick: move |_| {
            text.set(o.value.label());
            suggestions.close();
        },"#;

// snippet: after FRUIT_ENUM
// snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let fruit = use_combobox();
// snippet: in Combobox { state: fruit, options: Fruit::options().to_vec(), option: move |o: ComboboxOptionArgs<Fruit>| rsx! { ComboboxOption { .. } } }
const PLAIN_ROW: &str = r#"        "{o.value.label()}""#;

// snippet: after FRUIT_ENUM
// snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
// snippet: let fruit = use_combobox();
// snippet: in Combobox { state: fruit, options: Fruit::options().to_vec(), option: move |o: ComboboxOptionArgs<Fruit>| rsx! { ComboboxOption { .. } } }
const RICH_ROW: &str = r#"        Text { component: "span", size: "xl", "aria-hidden": "true", "{o.value.emoji()}" }
        Flex {
            direction: "column",
            justify: "center",
            align: "flex-start",
            sx: sx().gap("0"),
            Text { component: "span", size: "sm", "{o.value.label()}" }
            Text {
                component: "span",
                size: "xs",
                sx: sx().color("text-dimmed"),
                "{o.value.note()}"
            }
        }"#;

/// Suggestions and fetching wire their rows the same way: picking one fills
/// the text.
fn suggesting(values: &DemoValues) -> bool {
    matches!(values.str("mode").as_str(), "suggestions" | "fetching")
}

fn fetching(values: &DemoValues) -> bool {
    values.str("mode") == "fetching"
}

/// The mode also prints its example's props: a filtered `Vec` or the enum's own list.
fn mode_code(_: &Control, values: &DemoValues) -> Vec<String> {
    if fetching(values) {
        return [
            "state: suggestions",
            "options: results().map(OptionList::from)",
            r#"loading_label: "Searching fruit""#,
            r#"empty_label: "No fruit matches""#,
            r#"empty: rsx! { Text { size: "sm", sx: sx().padding("xs"), "No fruit matches" } }"#,
        ]
        .map(String::from)
        .to_vec();
    }
    let (state, options) = match suggesting(values) {
        true => ("state: suggestions", "options: matches"),
        false => ("state: fruit", "options: Fruit::options().to_vec()"),
    };
    let mut code = vec![state.to_string(), options.to_string()];
    if !suggesting(values) {
        code.push("labelled_by: label.clone()".to_string());
    }
    code
}

fn option_code(_: &Control, values: &DemoValues) -> Vec<String> {
    let wiring = match suggesting(values) {
        true => SUGGESTION_WIRING,
        false => SELECT_WIRING,
    };
    let row = match values.str("option").as_str() {
        "true" => RICH_ROW,
        _ => PLAIN_ROW,
    };
    vec![format!(
        "option: move |o: ComboboxOptionArgs<Fruit>| rsx! {{\n    \
         ComboboxOption {{\n{wiring}\n{row}\n    }}\n}}"
    )]
}

/// The trigger, disabled along with the combobox as the `disabled` row asks.
fn trigger_code(values: &DemoValues) -> String {
    let trigger = if fetching(values) {
        FETCHING_TRIGGER
    } else if suggesting(values) {
        SUGGESTIONS_TRIGGER
    } else {
        SELECT_TRIGGER
    };
    let width = "    sx: sx().width(\"280px\"),\n";
    match values.str("disabled") == "true" {
        true => trigger.replacen(width, &format!("{width}    disabled: true,\n"), 1),
        false => trigger.to_string(),
    }
}

fn preamble(values: &DemoValues, source: &str) -> String {
    let (search, state) = match values.str("mode").as_str() {
        "fetching" => (FETCHING_SEARCH, FETCHING_STATE),
        "suggestions" => ("", SUGGESTIONS_STATE),
        _ => ("", SELECT_STATE),
    };
    format!("{FRUIT_ENUM}{search}{state}{source}")
}

#[derive(Clone, Copy, PartialEq, Options)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
    #[option(label = "Dragon fruit")]
    Dragon,
    Elderberry,
    Mango,
    Grape,
}

impl Fruit {
    fn emoji(self) -> &'static str {
        match self {
            Self::Apple => "🍎",
            Self::Banana => "🍌",
            Self::Cherry => "🍒",
            Self::Dragon => "🐉",
            Self::Elderberry => "🫐",
            Self::Mango => "🥭",
            Self::Grape => "🍇",
        }
    }

    /// The second line of a rich option row.
    fn note(self) -> &'static str {
        match self {
            Self::Apple => "Crisp, keeps for weeks",
            Self::Banana => "Ripens on the counter",
            Self::Cherry => "In season for a fortnight",
            Self::Dragon => "Mild, mostly texture",
            Self::Elderberry => "Cook it, never raw",
            Self::Mango => "Ripe when it gives to a thumb",
            Self::Grape => "Sweetest straight off the vine",
        }
    }
}

/// What a row draws, plain or rich - the same either side of the mode toggle.
#[component]
fn RowContent(fruit: Fruit, rich: bool) -> Element {
    rsx! {
        if rich {
            Text { component: "span", size: "xl", "aria-hidden": "true", "{fruit.emoji()}" }
            Flex {
                direction: "column",
                justify: "center",
                align: "flex-start",
                sx: sx().gap("0"),
                Text { component: "span", size: "sm", "{fruit.label()}" }
                Text {
                    component: "span",
                    size: "xs",
                    sx: sx().color("text-dimmed"),
                    "{fruit.note()}"
                }
            }
        } else {
            "{fruit.label()}"
        }
    }
}

/// A button opens the list, a pick closes it, and the pick is shown below.
#[component]
fn SelectDemo(values: DemoValues) -> Element {
    let fruit = use_combobox();
    let mut picked = use_signal(|| None::<Fruit>);
    let rich = values.str("option") == "true";
    let disabled = values.str("disabled") == "true";

    // A `combobox` role takes no name from its content, so the label names it.
    let label = format!("{}-label", fruit.id());
    rsx! {
        Flex {
            direction: "column",
            gap: "sm",
            align: "flex-start",
            Combobox {
                size: values.str("size"),
                radius: values.str("radius"),
                disabled: disabled.then_some(true),
                state: fruit,
                options: Fruit::options().to_vec(),
                labelled_by: label.clone(),
                option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                    ComboboxOption {
                        selected: picked() == Some(o.value),
                        onpick: move |_| {
                            picked.set(Some(o.value));
                            fruit.close();
                        },
                        RowContent { fruit: o.value, rich }
                    }
                },
                Text { id: "{label}", size: "sm", "Fruit" }
                Button {
                    variant: "outlined",
                    sx: sx().width("280px"),
                    disabled,
                    attributes: [
                        fruit.a11y_attributes(),
                        vec![Attribute::new("aria-labelledby", label, None, false)],
                    ]
                    .concat(),
                    onclick: move |_| fruit.toggle(),
                    onblur: move |_| fruit.close(),
                    match picked() {
                        Some(fruit) => rsx! { "{fruit.label()}" },
                        None => rsx! { "Pick a fruit" },
                    }
                }
            }
            Text {
                size: "sm",
                sx: sx().color("text-dimmed"),
                match picked() {
                    Some(fruit) => rsx! { "Picked: {fruit.label()}" },
                    None => rsx! { "Nothing picked yet" },
                }
            }
        }
    }
}

/// The fruit whose label contains `query`, case-insensitively.
fn matching(query: &str) -> Vec<Fruit> {
    let query = query.to_lowercase();
    Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| fruit.label().to_lowercase().contains(&query))
        .collect()
}

/// Every keystroke starts a fake search that answers after [`LATENCY`], and a
/// newer keystroke cancels the older one by dropping its timer.
#[component]
fn FetchingDemo(values: DemoValues) -> Element {
    let suggestions = use_combobox();
    let rich = values.str("option") == "true";
    let disabled = values.str("disabled") == "true";
    let mut text = use_signal(String::new);
    // One signal, not a list plus a `loading` flag: `None` *is* the search in
    // flight, so the two can never disagree.
    let mut results = use_signal(|| Some(Vec::<Fruit>::new()));
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));

    rsx! {
        Combobox {
            size: values.str("size"),
            radius: values.str("radius"),
            disabled: disabled.then_some(true),
            state: suggestions,
            options: results().map(OptionList::from),
            loading_label: "Searching fruit",
            empty_label: "No fruit matches",
            empty: rsx! { Text { size: "sm", sx: sx().padding("xs"), "No fruit matches" } },
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        text.set(o.value.label());
                        suggestions.close();
                    },
                    RowContent { fruit: o.value, rich }
                }
            },
            TextField {
                sx: sx().width("280px"),
                label: "Fruit",
                placeholder: "Type a fruit",
                value: text(),
                disabled,
                attributes: suggestions.a11y_attributes(),
                onblur: move |_| suggestions.close(),
                oninput: move |next: String| {
                    text.set(next.clone());
                    suggestions.open();
                    results.set(None);
                    let answer = timer().map(|timer| {
                        timer.after(
                            LATENCY,
                            Box::new(move || results.set(Some(matching(&next)))),
                        )
                    });
                    pending.set(answer);
                },
            }
        }
    }
}

/// Typing filters the list; picking a suggestion fills the field.
#[component]
fn SuggestionsDemo(values: DemoValues) -> Element {
    let suggestions = use_combobox();
    let mut text = use_signal(String::new);
    let rich = values.str("option") == "true";
    let disabled = values.str("disabled") == "true";

    let matches: Vec<Fruit> = Fruit::options()
        .iter()
        .copied()
        .filter(|fruit| {
            fruit
                .label()
                .to_lowercase()
                .contains(&text().to_lowercase())
        })
        .collect();

    rsx! {
        Combobox {
            size: values.str("size"),
            radius: values.str("radius"),
            disabled: disabled.then_some(true),
            state: suggestions,
            options: matches,
            option: move |o: ComboboxOptionArgs<Fruit>| rsx! {
                ComboboxOption {
                    onpick: move |_| {
                        text.set(o.value.label());
                        suggestions.close();
                    },
                    RowContent { fruit: o.value, rich }
                }
            },
            TextField {
                sx: sx().width("280px"),
                label: "Fruit",
                placeholder: "Type a fruit",
                value: text(),
                disabled,
                attributes: suggestions.a11y_attributes(),
                onblur: move |_| suggestions.close(),
                oninput: move |next| {
                    text.set(next);
                    suggestions.open();
                },
            }
            input { r#type: "hidden", name: "fruit", value: "{text()}" }
        }
    }
}

#[component]
pub fn ComboboxPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "Combobox",
            source: "libero/src/components/form/combobox",
            markdown: "/md/combobox.md",
            properties: vec![
                props("Combobox", vec![
                    prop("state", "ComboboxState")
                        .doc("From `use_combobox()`. The open state, the highlighted row and the id the aria wiring uses. Required. One state drives one combobox."),
                    prop("options", "OptionSource<T>")
                        .doc("The options to list, already filtered. Required. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. `None::<OptionList<T>>` is pending, for a fetch you drive yourself. Every option renders, so cap the list here. A failed fetch is an empty list, so show your own error beside the field."),
                    prop("option", "Callback<ComboboxOptionArgs<T>, Element>")
                        .doc("Draws one row, usually a `ComboboxOption`. Required."),
                    prop("children", "Element")
                        .doc("The trigger, and anything that belongs with it, such as a hidden input."),
                    prop("empty", "Element")
                        .doc("Shown in place of the list when `options` is empty."),
                    prop("empty_label", "String")
                        .default("combobox.nothing_found")
                        .doc("What screen readers hear when the open list has no options, and what it shows there without `empty`."),
                    prop("loading_label", "String")
                        .default("common.loading")
                        .doc("What screen readers hear while `options` is pending. A pending list shows a `Loader` instead of the rows or `empty`."),
                    prop("labelled_by", "String")
                        .doc("The id of the element that names the list, usually the trigger's label. Screen readers read it with the list."),
                    prop("size", "Size")
                        .default(theme.combobox.size.as_str())
                        .doc("A row's height and font size."),
                    prop("radius", "Size")
                        .default(theme.combobox.radius.as_str())
                        .doc("The dropdown's corner radius."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Draws no list and ignores the keys, and the trigger reads as closed. Disable the trigger too."),
                ])
                .parts("DropdownPart", list_dropdown_parts(COMBOBOX_DROPDOWN)),
                props("ComboboxOption", vec![
                    prop("selected", "bool")
                        .doc("Marks the current selection with `aria-selected` and a tint. Leave it unset in a suggestion list."),
                    prop("active", "bool")
                        .default("from the Combobox")
                        .doc("Overrides the keyboard highlight."),
                    prop("disabled", "bool")
                        .default("from the Combobox")
                        .doc("Overrides whether the row is refused. A refused row is greyed and ignores the click and Enter."),
                    prop("onpick", "EventHandler<()>")
                        .doc("Called on a click, and by Enter while the row is active."),
                    prop("size", "Size")
                        .doc("Row height and font size. Defaults to the `Combobox`'s `size`."),
                    prop("radius", "Size")
                        .doc("Corner radius. Defaults to the `Combobox`'s, reduced so the row nests inside the dropdown."),
                    prop("children", "Element")
                        .doc("The row's content. Put a long label in `span { \"data-slot\": \"label\" }` to end it in an ellipsis."),
                ]),
                props("ComboboxOptionArgs", vec![
                    prop("value", "T").doc("The option this row draws."),
                    prop("index", "usize").doc("The row's position in `options`."),
                    prop("active", "bool").doc("The arrow keys are on this row. For a row drawn without `ComboboxOption`."),
                    prop("disabled", "bool").doc("The list refuses this row. A row drawn without `ComboboxOption` owes the greying and `aria-disabled`."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Down"], "Opens the list, and moves the highlight down.")
                .key(["Up"], "Moves the highlight up.")
                .key(["Home", "End"], "Jumps to the first or last row.")
                .key(["Enter"], "Picks the highlighted row.")
                .key(["Escape", "Tab"], "Close the list.")
                .handles([
                    "Focus stays on your trigger, so typing keeps working.",
                    "An open list with no options says `empty_label`, so an empty search is heard, not only seen.",
                    "Android's Back button calls `onopened(false)` while the list is open, rather than closing the app.",
                ])
                .must([
                    "Spread `state.a11y_attributes()` on your trigger, or screen readers cannot tie the list to it.",
                    "Close the list on your trigger's blur, or an enclosing `Modal` stops hearing Escape while the list stays open.",
                    "Name the trigger: it becomes a `combobox`, which takes no name from its content. Point a button trigger's `aria-labelledby` at a visible label, and give a text field a `label`.",
                    "Pass the same label's id as `labelled_by`, so the list has a name too.",
                ]),
            lead: rsx! {
                Text {
                    "A listbox that hangs off whatever control you put in it. It holds no "
                    "state of its own. "
                    Code { source: "use_combobox()" }
                    " keeps the open state in your scope, the selection is yours, "
                    Code { source: "option" }
                    " draws the rows and "
                    Code { source: "children" }
                    " is the trigger. The combobox adds the placement, the arrow keys and the "
                    "row styling. Closing on an outside click is yours, and "
                    Code { source: "onpick" }
                    " is the only way to pick."
                }
            },
            // snippet: item impl Fruit { fn emoji(self) -> &'static str { "" } fn note(self) -> &'static str { "" } }
            Demo {
                component: "Combobox",
                children_text: "",
                code_child: Child(trigger_code),
                wrap: Wrap(preamble),
                controls: vec![
                    Control::toggle("mode", ["select", "suggestions", "fetching"])
                        .labels(["Select", "Suggestions", "Fetching"])
                        .code(mode_code),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::switch("option").code(option_code),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| match values.str("mode").as_str() {
                    "suggestions" => rsx! { SuggestionsDemo { values } },
                    // Every keystroke answers after 700ms, so the loader shows.
                    "fetching" => rsx! { FetchingDemo { values } },
                    _ => rsx! { SelectDemo { values } },
                },
            }
        }
    }
}
