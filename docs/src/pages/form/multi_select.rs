use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Code, FieldStatus, MultiSelect, Options, Text},
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

#[derive(Clone, Copy, PartialEq, Options)]
enum Topping {
    Cheese,
    Mushrooms,
    Olives,
    Onions,
    Peppers,
    Pineapple,
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
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{TOPPING_ENUM}{source}")),
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
                        status: match values.str("status").as_str() {
                            "warning" => FieldStatus::Warning("Pineapple divides the table.".to_string()),
                            "error" => FieldStatus::Error("Pick at least one.".to_string()),
                            _ => FieldStatus::Valid,
                        },
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
