use std::time::Duration;

use crate::components::{Child, Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{
        Button, Code, Combobox, ComboboxOption, ComboboxOptionArgs, Flex, Options, Text, TextField,
        use_combobox,
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

/// A fetch per keystroke. `loading` is what keeps `empty` from flashing
/// between the keystroke and the answer.
// snippet: after FRUIT_ENUM
const FETCHING_STATE: &str = r#"let suggestions = use_combobox();
let mut text = use_signal(String::new);
let mut results = use_signal(Vec::<Fruit>::new);
let mut loading = use_signal(|| false);
// The answer still on its way. Replacing it drops the older one, so a slow
// answer never overwrites a newer query's.
let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
use_drop(move || pending.set(None));

"#;

// snippet: after FRUIT_ENUM, FETCHING_SEARCH, FETCHING_STATE
const FETCHING_TRIGGER: &str = r#"TextField {
    sx: sx().width("280px"),
    placeholder: "Type a fruit",
    value: text(),
    attributes: suggestions.a11y_attributes(),
    oninput: move |next: String| {
        text.set(next.clone());
        suggestions.open();
        loading.set(true);
        let answer = timer().map(|timer| {
            timer.after(
                LATENCY,
                Box::new(move || {
                    results.set(matching(&next));
                    loading.set(false);
                }),
            )
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
const SELECT_TRIGGER: &str = r#"Button {
    variant: "outlined",
    sx: sx().width("280px"),
    attributes: fruit.a11y_attributes(),
    onclick: move |_| fruit.toggle(),
    match picked() {
        Some(fruit) => rsx! { "{fruit.label()}" },
        None => rsx! { "Pick a fruit" },
    }
}"#;

// snippet: after FRUIT_ENUM, SUGGESTIONS_STATE
const SUGGESTIONS_TRIGGER: &str = r#"TextField {
    sx: sx().width("280px"),
    placeholder: "Type a fruit",
    value: text(),
    attributes: suggestions.a11y_attributes(),
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
const RICH_ROW: &str = r#"        Text { component: "span", size: "xl", "{o.value.emoji()}" }
        Flex {
            direction: "column",
            justify: "center",
            align: "flex-start",
            sx: sx().gap("0"),
            Text { component: "span", size: "sm", "{o.value.label()}" }
            Text {
                component: "span",
                size: "xs",
                sx: sx().color("grey.6"),
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

/// The mode drives the state the two examples keep, so it prints their props
/// too - `options` is a whole filtered `Vec` in one and the enum's own list in
/// the other.
fn mode_code(_: &Control, values: &DemoValues) -> Vec<String> {
    if fetching(values) {
        return [
            "state: suggestions",
            "options: results()",
            "loading: loading()",
            r#"loading_label: "Searching fruit""#,
            r#"empty: rsx! { Text { size: "sm", sx: sx().padding("xs"), "No fruit matches" } }"#,
        ]
        .map(String::from)
        .to_vec();
    }
    let (state, options) = match suggesting(values) {
        true => ("state: suggestions", "options: matches"),
        false => ("state: fruit", "options: Fruit::options().to_vec()"),
    };
    vec![state.to_string(), options.to_string()]
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

fn trigger_code(values: &DemoValues) -> String {
    if fetching(values) {
        return FETCHING_TRIGGER.to_string();
    }
    match suggesting(values) {
        true => SUGGESTIONS_TRIGGER.to_string(),
        false => SELECT_TRIGGER.to_string(),
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
            Text { component: "span", size: "xl", "{fruit.emoji()}" }
            Flex {
                direction: "column",
                justify: "center",
                align: "flex-start",
                sx: sx().gap("0"),
                Text { component: "span", size: "sm", "{fruit.label()}" }
                Text {
                    component: "span",
                    size: "xs",
                    sx: sx().color("grey.6"),
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
                Button {
                    variant: "outlined",
                    sx: sx().width("280px"),
                    disabled,
                    attributes: fruit.a11y_attributes(),
                    onclick: move |_| fruit.toggle(),
                    match picked() {
                        Some(fruit) => rsx! { "{fruit.label()}" },
                        None => rsx! { "Pick a fruit" },
                    }
                }
            }
            Text {
                size: "sm",
                sx: sx().color("grey.6"),
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
    let mut results = use_signal(Vec::<Fruit>::new);
    let mut loading = use_signal(|| false);
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));

    rsx! {
        Combobox {
            size: values.str("size"),
            radius: values.str("radius"),
            disabled: disabled.then_some(true),
            state: suggestions,
            options: results(),
            loading: loading(),
            loading_label: "Searching fruit",
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
                placeholder: "Type a fruit",
                value: text(),
                disabled,
                attributes: suggestions.a11y_attributes(),
                oninput: move |next: String| {
                    text.set(next.clone());
                    suggestions.open();
                    loading.set(true);
                    let answer = timer().map(|timer| {
                        timer.after(
                            LATENCY,
                            Box::new(move || {
                                results.set(matching(&next));
                                loading.set(false);
                            }),
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
                placeholder: "Type a fruit",
                value: text(),
                disabled,
                attributes: suggestions.a11y_attributes(),
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
    rsx! {
        DocPage {
            title: "Combobox",
            source: "libero/src/components/form/combobox",
            markdown: "/md/combobox.md",
            properties: vec![
                props("Combobox", vec![
                    prop("state", "ComboboxState")
                        .doc("From `use_combobox()`: the open state, the arrow-key highlight, and the id the aria wiring is built from. Required."),
                    prop("options", "Vec<T>")
                        .doc("The options to list, already filtered. Required."),
                    prop("option", "Callback<ComboboxOptionArgs<T>, Element>")
                        .doc("Draws one row - typically a `ComboboxOption`. Required."),
                    prop("children", "Element")
                        .doc("The trigger, and anything else that belongs with it - a hidden input, say."),
                    prop("empty", "Element")
                        .doc("Shown in place of the list when `options` is empty."),
                    prop("loading", "bool")
                        .default("false")
                        .doc("The options are being fetched. Replaces the rows and `empty` with a `Loader`, marks the dropdown `aria-busy`, and puts `loading_label` in a status region beside the trigger. It wins over `empty`, so an async list does not flash \"no results\" on every keystroke."),
                    prop("loading_label", "String")
                        .default("theme")
                        .doc("What the status region says while `loading`. Unset, `theme.combobox.labels.loading` - \"Loading\" in `ComboboxLabels::ENGLISH`."),
                    prop("size", "Size")
                        .default("md")
                        .doc("A row's height and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("The dropdown's corner radius."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Blocks the arrow keys. Disable the trigger too, which draws its own dimmed state."),
                ]),
                props("ComboboxOption", vec![
                    prop("selected", "bool")
                        .doc("The current selection - `aria-selected` and a tint. Leave it unset for a suggestion list."),
                    prop("active", "bool")
                        .default("from the Combobox")
                        .doc("Overrides the keyboard highlight, which otherwise comes from the `Combobox` drawing the row."),
                    prop("onpick", "EventHandler<()>")
                        .doc("Called on a click, and by Enter while the row is active."),
                    prop("size", "Size")
                        .doc("Row height and font size. Defaults to the `Combobox`'s own `size`."),
                    prop("radius", "Size")
                        .doc("Corner radius, tightened by the dropdown's padding so the row nests inside it. Defaults to the `Combobox`'s own."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A listbox that hangs off whatever control you put in it. It holds "
                    "no state of its own: "
                    Code { source: "use_combobox()" }
                    " keeps it in your scope, the selection is yours entirely, the rows "
                    "are drawn by "
                    Code { source: "option" }
                    ", and the trigger is just "
                    Code { source: "children" }
                    ". All it adds is the placement, the arrow keys, and the row theming."
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
