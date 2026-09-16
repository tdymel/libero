use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Flex, PinField, Text};

#[component]
pub fn PinFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "PinField",
            source: "libero/src/components/form/pin_field.rs",
            markdown: "/md/pin_field.md",
            properties: vec![
                props("PinField", vec![
                    prop("size", "Size").default("md").doc("Controls the cell's square, its font size and the gap. A cell is as tall as a `TextField` at the same size."),
                    prop("radius", "Size").default("sm").doc("Corner radius of each cell, independent of size."),
                    prop("length", "usize").default("4").doc("How many cells."),
                    prop("kind", "PinKind").default("numeric").doc("`numeric` or `alphanumeric`. Anything else is dropped at the key, so a rejected character never appears."),
                    prop("value", "Option<String>")
                        .doc("The pin so far, one character per filled cell. `None` leaves the cells to the field's own buffer."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires per accepted character with the pin the field should hold next."),
                    prop("oncomplete", "EventHandler<String>")
                        .doc("Fires once when the last empty cell fills. Clearing a cell arms it again."),
                    prop("mask", "bool").default("false").doc("Renders the cells as password inputs. The value is unaffected."),
                    prop("one_time_code", "bool")
                        .default("true")
                        .doc("Puts `autocomplete=\"one-time-code\"` on the first cell, so a phone offers the code it just received."),
                    prop("separator", "Element").doc("Rendered between the cells - a dash, a wider gap."),
                    prop("name", "String").doc("Emits a hidden input of that name, so the pin posts with a form. The cells cannot carry it - there are several of them."),
                    prop("autofocus", "bool").default("false").doc("Focuses the first cell on mount."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the cells. Names the group through `aria-labelledby` - a `div` is not labelable."),
                    prop("description", "Caption").doc("Between the label and the cells: where the code came from."),
                    prop("helper", "Caption").doc("Under the cells: how long it lasts, how to get another."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` to every cell and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables every cell and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A pin, one character per cell. Typing fills a cell and moves to the next, "
                    "Backspace clears and steps back, and the arrows move without changing "
                    "anything. Pasting a whole code into any cell spreads it across the rest - "
                    "separators and spaces included, so "
                    Code { source: "\"4 2-1 3\"" }
                    " lands as "
                    Code { source: "4213" }
                    ". "
                    Code { source: "oncomplete" }
                    " fires the moment the last cell fills, which is usually where the code "
                    "gets submitted."
                }
            },
            Demo {
                component: "PinField",
                children_text: "",
                controls: vec![
                    // `length` is a `usize`, so it must not print quoted the
                    // way a bare slider's value would.
                    Control::slider("length", ["4", "5", "6", "8"])
                        .default("4")
                        .code(|control, values| match values.str("length") == control.default {
                            true => vec![],
                            false => vec![format!("length: {}", values.str("length"))],
                        }),
                    Control::toggle("kind", ["numeric", "alphanumeric"])
                        .labels(["Numeric", "Alphanumeric"])
                        .default("numeric")
                        .code(|_, values| match values.str("kind").as_str() {
                            "alphanumeric" => vec!["kind: \"alphanumeric\"".to_string()],
                            _ => vec![],
                        }),
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"]).default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"That code is about to expire.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"That code is wrong.\"".to_string()],
                            _ => vec![],
                        }),
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Verification code\"".to_string()],
                            _ => vec!["aria_label: \"Verification code\"".to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Sent to your phone.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"It expires in ten minutes.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    // The separator is an `Element`, so the control switches a
                    // whole `rsx!` in rather than a value.
                    Control::switch("separator").code(|_, values| {
                        match values.str("separator").as_str() {
                            "true" => vec!["separator: rsx! { \"-\" }".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("mask"),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    PinFieldDemo { values }
                },
            }
        }
    }
}

/// Its own component so the pin and the completion count are state the demo
/// keeps across a control change - which is what shows `oncomplete` firing
/// once per fill rather than once per keystroke.
#[component]
fn PinFieldDemo(values: DemoValues) -> Element {
    let mut value = use_signal(String::new);
    let mut verified = use_signal(|| false);

    let length = values.str("length").parse::<usize>().unwrap_or(4);

    rsx! {
        Flex { direction: "column", gap: "sm", align: "center",
            PinField {
                size: values.str("size"),
                radius: values.str("radius"),
                length,
                kind: values.str("kind"),
                label: (values.str("label") == "true").then(|| "Verification code".to_string()),
                aria_label: (values.str("label") != "true").then_some("Verification code"),
                description: (values.str("description") == "true")
                    .then(|| "Sent to your phone.".to_string()),
                helper: (values.str("helper") == "true")
                    .then(|| "It expires in ten minutes.".to_string()),
                status: match values.str("status").as_str() {
                    "warning" => FieldStatus::Warning("That code is about to expire.".to_string()),
                    "error" => FieldStatus::Error("That code is wrong.".to_string()),
                    _ => FieldStatus::Valid,
                },
                separator: (values.str("separator") == "true").then(|| rsx! { "-" }),
                mask: (values.str("mask") == "true").then_some(true),
                required: (values.str("required") == "true").then_some(true),
                disabled: (values.str("disabled") == "true").then_some(true),
                value: value(),
                oninput: move |next| {
                    value.set(next);
                    verified.set(false);
                },
                oncomplete: move |_| verified.set(true),
            }
            Text {
                size: "sm",
                match verified() {
                    true => "Complete - oncomplete fired once.".to_string(),
                    false => format!("{} of {length} entered.", value().chars().count()),
                }
            }
        }
    }
}
