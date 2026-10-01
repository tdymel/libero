use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::components::FieldPart;
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
                    prop("size", "Size").default("md").doc("The cell's square, its font size and the gap. A cell is as tall as a `TextField` of the same size."),
                    prop("radius", "Size").default("sm").doc("Corner radius of each cell, independent of `size`."),
                    prop("length", "usize").default("4").doc("How many cells."),
                    prop("kind", "PinKind").default("numeric").doc("`numeric` or `alphanumeric`. Any other character is ignored as it is typed."),
                    prop("value", "Option<String>")
                        .doc("The pin so far, one character per filled cell. Leave it out and the field keeps its own pin."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires for every accepted character with the pin the field should hold next."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the pin, shown once the field loses focus or its form is submitted."),
                    prop("oncomplete", "EventHandler<String>")
                        .doc("Fires once when the last empty cell fills. Clearing a cell arms it again."),
                    prop("mask", "bool").default("false").doc("Hides the characters as in a password field. The value is unaffected."),
                    prop("one_time_code", "bool")
                        .default("true")
                        .doc("Lets a phone offer the code it just received."),
                    prop("separator", "Element").doc("Rendered between the cells, such as a dash."),
                    prop("name", "FieldName<String>").doc("What the pin posts as. A path such as `Login::FIELDS.code()` also binds the pin to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("autofocus", "bool").default("false").doc("Focuses the first cell on mount."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the cells. It names the group of cells."),
                    prop("description", "Caption").doc("Between the label and the cells. Where the code came from."),
                    prop("helper", "Caption").doc("Under the cells. How long the code lasts, how to get another."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error, an empty one `Valid`."),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables and dims every cell."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "Each cell's box."),
                    (FieldPart::Control, "Each cell's input."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]),
            ],
            accessibility: a11y()
                .key(["Left", "Right"], "Moves to the previous or next cell.")
                .key(["Home", "End"], "Moves to the first or last cell.")
                .key(["Backspace"], "Clears the cell and moves back, except from the last cell; on an empty cell, only moves back.")
                .key(["Delete"], "Clears the cell and stays.")
                .key(["Space"], "Moves to the next cell without typing.")
                .key(["Tab"], "Moves to the next cell, and past the last one leaves the field, as in any group of inputs.")
                .handles([
                    "Each cell is a tab stop.",
                    "The pin has no holes: clearing a middle cell moves the characters after it one cell left, and a character typed past the pin lands in the first empty cell.",
                    "A separator is decoration, hidden from screen readers.",
                    "Each cell is named for its place, such as \"Character 1 of 6\", from the localization's `PinFieldLabels`, and reads the helper and the error too.",
                ])
                .must(["Give the field a `label`, which names the whole group."]),
            lead: rsx! {
                Text {
                    "A pin, one character per cell. Typing fills a cell and moves to the next, "
                    "Backspace clears and steps back, and the arrows move without changing "
                    "anything. A code pasted into any cell spreads across the rest, and "
                    "characters the field does not take are dropped, so "
                    Code { source: "\"4 2-1 3\"" }
                    " lands as "
                    Code { source: "4213" }
                    ". "
                    Code { source: "oncomplete" }
                    " fires the moment the last cell fills, which is usually where you submit "
                    "the code. Extra HTML attributes land on the group, not on a cell."
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
                        .labels(["Valid", "Warning", "Error"])
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

/// Its own component, so the pin survives a control change and shows
/// `oncomplete` firing once per fill.
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
            // The count stays silent per key; only the completion is announced.
            if !verified() {
                Text { size: "sm", "{value().chars().count()} of {length} entered." }
            }
            Text { size: "sm", role: "status",
                if verified() { "Complete, oncomplete fired once." }
            }
        }
    }
}
