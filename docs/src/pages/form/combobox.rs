use std::time::Duration;

use super::dropdown_parts::{COMBOBOX_DROPDOWN, list_dropdown_parts};
use crate::components::{
    Child, Control, Demo, DemoFile, DemoValues, DocPage, Wrap, a11y, indent, prop, props,
};
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

mod demo;
use demo::{FetchingDemo, SelectDemo, SuggestionsDemo};

/// The live demos, printed from their text: the printed parts are cut from them.
const FILE: DemoFile = DemoFile(include_str!("combobox/demo.rs"));

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
        code.push("select_only: true".to_string());
    }
    code
}

fn option_code(_: &Control, values: &DemoValues) -> Vec<String> {
    let wiring = match suggesting(values) {
        true => FILE.section("suggestion_wiring"),
        false => FILE.section("select_wiring"),
    };
    let row = match values.str("option").as_str() {
        "true" => FILE.section("rich_row"),
        _ => FILE.section("plain_row"),
    };
    let (wiring, row) = (indent(&indent(&wiring)), indent(&indent(&row)));
    let (wiring, row) = (wiring.trim_end(), row.trim_end());
    vec![format!(
        "option: move |o: ComboboxOptionArgs<Fruit>| rsx! {{\n    \
         ComboboxOption {{\n{wiring}\n{row}\n    }}\n}}"
    )]
}

/// The trigger, disabled along with the combobox as the `disabled` row asks.
fn trigger_code(values: &DemoValues) -> String {
    let trigger = if fetching(values) {
        FILE.section("fetching_trigger")
    } else if suggesting(values) {
        FILE.section("suggestions_trigger")
    } else {
        FILE.section("select_trigger")
    };
    let disabled = match values.str("disabled") == "true" {
        true => "\n    disabled: true,\n",
        false => "\n",
    };
    trigger.replacen("\n    disabled,\n", disabled, 1)
}

fn preamble(values: &DemoValues, source: &str) -> String {
    let (search, state) = match values.str("mode").as_str() {
        "fetching" => (Some("search"), "fetching_state"),
        "suggestions" => (None, "suggestions_state"),
        _ => (None, "select_state"),
    };
    let search = search.map_or(String::new(), |search| FILE.section(search) + "\n\n");
    let (fruit, state) = (FILE.section("fruit"), FILE.section(state));
    format!("{fruit}\n\n{search}{state}\n\n{source}")
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
                    prop("state", "ComboboxState").default("required")
                        .doc("From `use_combobox()`. The open state, the highlighted row and the id the aria wiring uses. One state drives one combobox."),
                    prop("options", "OptionSource<T>").default("required")
                        .doc("The options to list, already filtered. A `Vec<T>` converts, an `OptionList<T>` adds named groups and disabled options, and a `Resource<Vec<T>>` adds the loader while it fetches. `None::<OptionList<T>>` is pending, for a fetch you drive yourself. Every option renders, so cap the list here. A failed fetch is an empty list, so show your own error beside the field."),
                    prop("option", "Callback<ComboboxOptionArgs<T>, Element>").default("required")
                        .doc("Draws one row, usually a `ComboboxOption`."),
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
                    prop("select_only", "bool")
                        .default("false")
                        .doc("For a trigger with no text to type in, such as a button: Space picks the highlighted row like Enter, and Tab or Alt+Up pick it before closing, as a `Select` does."),
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
                        .doc("Called on a click, and by Enter while the row is active. The only way to pick."),
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
                .key(["PageUp", "PageDown"], "Moves the highlight 10 rows, stopping at the first or last.")
                .key(["Enter"], "Open: picks the highlighted row. Every open starts on the first row, so in a suggestion list Enter replaces the typed text; press `Escape` first to keep it.")
                .key(["Escape"], "Open: closes the list and keeps the typed text. Enter then goes to the field, so a form submits.")
                .key(["Space"], "With `select_only`, open: picks the highlighted row, as Enter does.")
                .key(["Tab"], "Closes the list. With `select_only`, picks the highlighted row first.")
                .handles([
                    "Focus stays on your trigger, so typing keeps working.",
                    "An open list with no options says `empty_label`, so an empty search is heard, not only seen.",
                    "Android's Back button calls `onopened(false)` while the list is open, rather than closing the app.",
                    "From 200 options the list draws only the rows in view. The highlighted row stays drawn for `aria-activedescendant`, a `ComboboxOption` row carries `aria-posinset` and `aria-setsize`, and a grouped row is described by its heading.",
                ])
                .must([
                    "Spread `state.a11y_attributes()` on your trigger, or screen readers cannot tie the list to it.",
                    "Close the list on your trigger's blur, or an enclosing `Modal` stops hearing Escape while the list stays open.",
                    "Name the trigger: it becomes a `combobox`, which takes no name from its content. Point a button trigger's `aria-labelledby` at a visible label, and give a text field a `label`.",
                    "Pass the same label's id as `labelled_by`, so the list has a name too.",
                    "From 200 options, keep every row the theme's `row_height` tall, as a plain `ComboboxOption` is: the list places the rows it draws by that height. A debug build warns when they differ.",
                ])
                .example("A search field as your own trigger, with `state.a11y_attributes()` spread on it and a visible label whose id is `labelled_by`: a screen reader reads a combobox named by the label, Down moves into the list, and focus never leaves the field."),
            lead: rsx! {
                Text {
                    "A listbox that hangs off whatever control you put in it, with the placement, "
                    "the arrow keys and the row styling. It holds no state: "
                    Code { source: "use_combobox()" }
                    " keeps the open state in your scope, and the selection and closing on an "
                    "outside click are yours."
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
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
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
