use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Chip, Code, FieldStatus, MultiSelect, Options, SelectOptionArgs, Text},
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
const TOPPING_IMPL: &str = r#"impl Topping {
    fn emoji(self) -> &'static str { /* "🧀", "🍄", ... */ }
}

"#;

const CUSTOM_OPTION: &str = r#"option: move |o: SelectOptionArgs<Topping>| rsx! {
    Text { component: "span", size: "lg", "{o.value.emoji()}" }
    Text { component: "span", sx: sx().flex("1 1 auto"), "{o.value.label()}" }
    if o.selected {
        Text { component: "span", "✓" }
    }
}"#;

const CUSTOM_SELECTION: &str = r#"selection: move |topping: Topping| rsx! {
    Chip { size: "xs", variant: "outlined", "{topping.emoji()} {topping.label()}" }
}"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
    Onions,
    Peppers,
    Pineapple,
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
}

fn custom(values: &DemoValues) -> bool {
    values.str("custom") == "true"
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

fn topping_selection(topping: Topping) -> Element {
    rsx! {
        Chip { size: "xs", variant: "outlined", "{topping.emoji()} {topping.label()}" }
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
                    prop("size", "Size").default("md").doc("Controls height, padding, font size and the rows' size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius of the frame and the list, independent of size."),
                    prop("value", "Vec<T>")
                        .doc("The selection, in the order it was picked; strictly controlled. Empty shows `placeholder`."),
                    prop("onchange", "EventHandler<Vec<T>>")
                        .doc("Called with the whole selection the caller should hold next."),
                    prop("options", "Vec<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set passes it here."),
                    prop("option", "Callback<SelectOptionArgs<T>, Element>")
                        .default("T::label()")
                        .doc("Draws one row's content. `selected` on the args is there for a checkmark."),
                    prop("selection", "Callback<T, Element>")
                        .default("Chip")
                        .doc("Draws one selected value inside the trigger. Replaces the chip entirely."),
                    prop("placeholder", "String").doc("Shown while `value` is empty."),
                    prop("clearable", "bool")
                        .default("false")
                        .doc("Shows an x in place of the chevron that empties the selection."),
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
                    "A listbox over an enum that holds any number of its options. The list stays "
                    "open on a pick and a pick toggles the row; Escape, clicking elsewhere and the "
                    "trigger close it. The selection is drawn in the trigger as chips, unless "
                    Code { source: "selection" }
                    " draws it some other way."
                }
            },
            Demo {
                component: "MultiSelect",
                children_text: "",
                // The helper the custom rows call only exists in the snippet
                // while those rows are on.
                wrap: Wrap(|values: &DemoValues, source: &str| match custom(values) {
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
                            _ => vec![],
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
                        description: (values.str("description") == "true")
                            .then(|| "Up to five, at no extra cost.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "Picked in the order they go on.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Pineapple divides the table.".to_string()),
                            "error" => FieldStatus::Error("Pick at least one.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        option: custom(&values).then(|| Callback::new(topping_row)),
                        selection: custom(&values).then(|| Callback::new(topping_selection)),
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
