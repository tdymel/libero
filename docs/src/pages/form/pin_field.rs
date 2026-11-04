use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, description_position_prop, field_controls,
    field_props, label_trailing_prop, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, Flex, PinField, Text};
use libero::theme::{Size, responsive};
use libero::use_theme;

struct CodeCopy;

impl FieldCopy for CodeCopy {
    const LABEL: &'static str = "Verification code";
    const DESCRIPTION: &'static str = "Sent to your phone.";
    const HELPER: &'static str = "It expires in ten minutes.";
    const WARNING: &'static str = "That code is about to expire.";
    const ERROR: &'static str = "That code is wrong.";
}

/// The demo's size from the `md` breakpoint up, `sm` below it.
fn responsive_on(values: &DemoValues) -> bool {
    values.str("responsive") == "true"
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
                    prop("size", "Responsive<Size>").default(theme.pin_field.size.as_str()).doc("The cell's square, its font size and the gap. A cell is as tall as a `TextField` of the same size. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`."),
                    prop("radius", "Responsive<ThemeAwareValue>").default(theme.pin_field.radius.as_str()).doc("Corner radius of each cell, independent of `size`. Or any CSS, e.g. `radius: \"0\"`. Or one per breakpoint, `radius: responsive(Size::Sm).md(Size::Xl)`."),
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
                    prop("separator", "Element").doc("Rendered between the cells, such as a dash. On a row too narrow for every cell, the separators shrink away first, so the cells keep their touch size."),
                    prop("name", "FieldName<String>").doc("What the pin posts as. A path such as `Login::FIELDS.code()` also binds the pin to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("autofocus", "bool").default("false").doc("Focuses the first cell on mount."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the cells. It names the group of cells."),
                    prop("description", "Caption").doc("Between the label and the cells. Where the code came from."),
                    label_trailing_prop(),
                    description_position_prop(),
                    prop("helper", "Caption").doc("Under the cells. How long the code lasts, how to get another."),
                    status_prop(),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label; a field without a visible `label` shows no mark. Inside a `Form`, an empty one fails the submit."),
                    prop("disabled", "bool").default("false").doc("Disables and dims every cell."),
                    readonly_prop("field"),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::LabelRow, "The row holding the label and its `label_trailing`; drawn only with a trailing slot."),
                    (FieldPart::LabelTrailing, "The `label_trailing` slot, at the end of the label row."),
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
                .key(["Backspace"], "Removes the character, so the ones after it move one cell left. In a middle cell it stays, and the next character typed goes back in there; from the last filled cell it moves back, except from the last cell. On an empty cell, only moves back.")
                .key(["Delete"], "Removes the character, so the ones after it move one cell left, and stays; the next character typed goes back in there.")
                .key(["Space"], "Moves to the next cell without typing.")
                .key(["Letter"], "Typing the character a cell already holds moves to the next cell without a change.")
                .key(["Tab"], "Moves to the next cell, and past the last one leaves the field, as in any group of inputs.")
                .handles([
                    "Each cell is a tab stop.",
                    "The pin has no holes: clearing a middle cell moves the characters after it one cell left, and a character typed past the pin lands in the first empty cell.",
                    "A separator is decoration, hidden from screen readers.",
                    "Each cell is named for its place, such as \"Character 1 of 6\", from the localization's `PinFieldLabels`, and reads the helper and the error too.",
                ])
                .must(["Give the field a `label`, which names the whole group."])
                .example("A sign-in code, `PinField { label: \"Verification code\", length: 6, .. }`: a screen reader reads the group as \"Verification code\" and each cell as \"Character 1 of 6\" and so on."),
            lead: rsx! {
                Text {
                    "A pin, one character per cell. Typing fills a cell and moves to the next, "
                    "Backspace clears and steps back, and the arrows move without changing "
                    "anything. A code pasted into any cell spreads across the rest, and "
                    "characters the field does not take are dropped, so "
                    Code { source: "\"4 2-1 3\"" }
                    " lands as "
                    Code { source: "4213" }
                    ". Extra HTML attributes land on the group, not on a cell."
                }
            },
            Demo {
                component: "PinField",
                pinned_preview: true,
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
                    Control::sizes("size").default(theme.pin_field.size.as_str())
                        .code(|control, values| match (responsive_on(values), values.str("size")) {
                            (true, size) => vec![format!(
                                "size: responsive(Size::Sm).md(Size::{:?})",
                                Size::from(size.as_str())
                            )],
                            (false, size) if size == control.default => vec![],
                            (false, size) => vec![format!("size: {size:?}")],
                        }),
                    // On, so the breakpoint switch shows without another switch: resize the window.
                    Control::switch("responsive").default("true").code(|_, _| vec![]),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"]).default(theme.pin_field.radius.as_str()),
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
                    Control::switch("readonly"),
                ]].concat(),
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
    // The length it fired at: another length is a new pin, not yet complete.
    let mut verified = use_signal(|| None::<usize>);

    let length = values.str("length").parse::<usize>().unwrap_or(4);
    let field = field_props::<CodeCopy>(&values);
    // What the cells show: a shorter length cuts the value.
    let count = value().chars().take(length).count();
    let complete = verified() == Some(length) && count == length;

    rsx! {
        Flex { direction: "column", gap: "sm", align: "center",
            PinField {
                size: match (responsive_on(&values), Size::from(values.str("size").as_str())) {
                    (true, size) => responsive(Size::Sm).md(size),
                    (false, size) => size.into(),
                },
                radius: values.str("radius"),
                length,
                kind: values.str("kind"),
                label: field.label,
                aria_label: field.aria_label,
                description: field.description,
                description_position: field.description_position,
                helper: field.helper,
                status: field.status,
                separator: (values.str("separator") == "true").then(|| rsx! { "-" }),
                mask: (values.str("mask") == "true").then_some(true),
                required: (values.str("required") == "true").then_some(true),
                disabled: (values.str("disabled") == "true").then_some(true),
                readonly: (values.str("readonly") == "true").then_some(true),
                value: value(),
                oninput: move |next| {
                    value.set(next);
                    verified.set(None);
                },
                oncomplete: move |_| verified.set(Some(length)),
            }
            // The count stays silent per key; only the completion is announced.
            if !complete {
                Text { size: "sm", "{count} of {length} entered." }
            }
            Text { size: "sm", role: "status",
                if complete { "Complete, oncomplete fired once." }
            }
        }
    }
}
