use crate::components::{
    Control, Demo, DemoValues, DocPage, DocSection, FieldCopy, a11y, field_controls, field_props,
    prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, Flex, PinField, Text};
use libero::use_theme;

struct CodeCopy;

impl FieldCopy for CodeCopy {
    const LABEL: &'static str = "Verification code";
    const DESCRIPTION: &'static str = "Sent to your phone.";
    const HELPER: &'static str = "It expires in ten minutes.";
    const WARNING: &'static str = "That code is about to expire.";
    const ERROR: &'static str = "That code is wrong.";
}

#[component]
pub fn PinFieldPage() -> Element {
    let theme = use_theme();
    rsx! {
        DocPage {
            title: "PinField",
            source: "libero/src/components/form/pin_field.rs",
            markdown: "/md/pin_field.md",
            properties: vec![
                props("PinField", vec![
                    prop("size", "Size").default(theme.pin_field.size.as_str()).doc("The cell's square, its font size and the gap. A cell is as tall as a `TextField` of the same size."),
                    prop("radius", "Size").default(theme.pin_field.radius.as_str()).doc("Corner radius of each cell, independent of `size`."),
                    prop("length", "usize").default(theme.pin_field.length.to_string()).doc("How many cells, clamped to 1 through 32."),
                    prop("kind", "PinKind").default(theme.pin_field.kind.as_str()).doc("`numeric` takes the digits 0-9, `alphanumeric` the ASCII letters and digits. Any other character, an accented letter too, is ignored as it is typed."),
                    prop("value", "Option<String>")
                        .doc("The pin so far, one character per filled cell. Leave it out and the field keeps its own pin."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires for every change of the pin, a typed character, a clear or a paste, with the pin the field should hold next."),
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
                    status_prop(),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables and dims every cell."),
                    readonly_prop("field"),
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
                .key(["Left", "Right"], "Moves to the cell on the left or right. The cells run left to right in a right-to-left locale too, so Left is always the previous cell.")
                .key(["Home", "End"], "Moves to the first or last cell.")
                .key(["Backspace"], "Clears the cell and moves back, except from the last cell; on an empty cell, only moves back.")
                .key(["Delete"], "Clears the cell and stays.")
                .key(["Space"], "Moves to the next cell without typing.")
                .key(["Letter"], "Typing the character a cell already holds moves to the next cell without a change.")
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
                    "."
                }
            },
            Demo {
                component: "PinField",
                children_text: "",
                controls: [vec![
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
                    Control::sizes("size").default("md"),
                    Control::sizes("radius").default("sm"),
                ], field_controls::<CodeCopy>(), vec![
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
                ]].concat(),
                render: move |values: DemoValues| rsx! {
                    PinFieldDemo { values }
                },
            }

            DocSection {
                title: "Completion and attributes",
                Text {
                    Code { source: "oncomplete" }
                    " fires the moment the last cell fills, which is usually where you submit "
                    "the code. Extra HTML attributes land on the group, not on a cell."
                }
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
    let field = field_props::<CodeCopy>(&values);

    rsx! {
        Flex { direction: "column", gap: "sm", align: "center",
            PinField {
                size: values.str("size"),
                radius: values.str("radius"),
                length,
                kind: values.str("kind"),
                label: field.label,
                aria_label: field.aria_label,
                description: field.description,
                helper: field.helper,
                status: field.status,
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
