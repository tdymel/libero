use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, description_position_prop, disabled_prop,
    field_controls, field_props, label_trailing_prop, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Button, Code, Text, TextField};
use libero::theme::{Size, responsive};
use libero::{hooks::use_element, platform::ElementApi, sx::sx, use_theme};

struct UsernameCopy;

impl FieldCopy for UsernameCopy {
    const LABEL: &'static str = "Username";
    const DESCRIPTION: &'static str = "How other people see you.";
    const HELPER: &'static str = "Letters, numbers and underscores.";
    const WARNING: &'static str = "That name is close to another one.";
    const ERROR: &'static str = "That name is taken.";
}

fn no_trailing(values: &DemoValues) -> bool {
    values.str("trailing") != "true"
}

fn label_trailing_on(values: &DemoValues) -> bool {
    values.str("label") == "true" && values.str("label_trailing") == "true"
}

// A disabled or read-only field takes no suggestion.
fn locked(values: &DemoValues) -> bool {
    values.str("disabled") == "true" || values.str("readonly") == "true"
}

/// The demo's size from the `md` breakpoint up, `sm` below it.
fn responsive_on(values: &DemoValues) -> bool {
    values.str("responsive") == "true"
}

#[component]
pub fn TextFieldPage() -> Element {
    let mut value = use_signal(String::new);
    let field_element = use_element();
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "TextField",
            source: "libero/src/components/form/text_field.rs",
            markdown: "/md/text_field.md",
            properties: vec![
                props("TextField", vec![
                    prop("size", "Responsive<Size>")
                        .default(theme.text_field.size.as_str())
                        .doc("Height, padding and font size. Or one per breakpoint, `size: responsive(Size::Sm).md(Size::Lg)`. Breakpoints follow the window, not the parent; the slot buttons and the clear button keep the base size."),
                    prop("radius", "Responsive<ThemeAwareValue>")
                        .default(theme.text_field.radius.as_str())
                        .doc("Corner radius, independent of `size`. Or any CSS, e.g. `radius: \"0\"`. Or one per breakpoint, `radius: responsive(Size::Sm).md(Size::Xl)`."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. Leave it out and the input keeps its own text."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires on every keystroke with the text the field should hold next."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the text, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.email()` also binds the text to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("leading", "Element")
                        .doc("Inside the frame, before the control, such as a search icon or a currency sign."),
                    prop("trailing", "Element")
                        .doc("Inside the frame, after the control, such as a clear button or a unit."),
                    prop("describe_leading", "bool")
                        .default("false")
                        .doc("Set it when `leading` is text that belongs to the value, such as `@`, so a screen reader reads it with the input. Not for an icon or a button."),
                    prop("describe_trailing", "bool")
                        .default("false")
                        .doc("The same for `trailing`, such as `kg` or a `12/20` counter."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the field. Takes a string or an `Element`. The label's element id is `{id}-label`, so a `Combobox`'s `labelled_by` can name its list from it."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    label_trailing_prop(),
                    description_position_prop(),
                    prop("helper", "Caption")
                        .doc("Under the control. Formatting rules, limits or a counter."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label; a field without a visible `label` shows no mark. Inside a `Form`, an empty one fails the submit."),
                    disabled_prop("field"),
                    readonly_prop("field"),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::LabelRow, "The row holding the label and its `label_trailing`; drawn only with a trailing slot."),
                    (FieldPart::LabelTrailing, "The `label_trailing` slot, at the end of the label row."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Leading, "The slot before the control: an icon, a prefix."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]).extends("input"),
            ],
            accessibility: a11y()
                .handles([
                    "A string `description` or `helper` is read with the input.",
                    "An error status marks the input invalid, a warning does not.",
                ])
                .must([
                    "Leave `label` unset only when something else names the field, such as an `aria_label`.",
                    "Markup in `description` or `helper` is shown but not read, so its accessibility is yours.",
                    "A `leading` or `trailing` slot is not read with the input. When it is text that belongs to the value, such as a unit or a counter, set `describe_leading` or `describe_trailing`.",
                ])
                .example("An email field, `TextField { label: \"Email\", helper: \"We never share it.\" }`: a screen reader reads the label, then the helper. An error status marks the input invalid and is read the same way."),
            lead: rsx! {
                Text {
                    "A single-line text field. Like every field, it stacks a label, a description, "
                    "the control, helper text and a validation message. Pass "
                    Code { source: "value" }
                    " and "
                    Code { source: "oninput" }
                    " to control it, or leave "
                    Code { source: "value" }
                    " out and the input keeps its own text. "
                    Code { source: "leading" }
                    " and "
                    Code { source: "trailing" }
                    " put content inside the border, beside the control. The "
                    Code { source: "Pending" }
                    " status sets "
                    Code { source: "aria-busy" }
                    "; its loader comes from a surrounding "
                    Code { source: "Form" }
                    "'s "
                    Code { source: "pending_loader" }
                    "."
                    " Fields side by side in a "
                    Code { source: "Flex" }
                    " row line up when the row sets "
                    Code { source: "align: \"flex-start\"" }
                    " (a row centres by default) and each field puts its description under the control with "
                    Code { source: "description_position: \"below\"" }
                    ", so a description on one does not push its control down."
                }
            },
            // snippet: let mut value = use_signal(String::new);
            Demo {
                component: "TextField",
                pinned_preview: true,
                children_text: "",
                fixed: vec![
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size")
                        .default(theme.text_field.size.as_str())
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
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.text_field.radius.as_str()),
                ], field_controls::<UsernameCopy>(), vec![
                    // It shares the label's row, so it goes with the label.
                    Control::switch("label_trailing")
                        .default("true")
                        .hidden_when(|values| values.str("label") != "true")
                        .code(|_, values| match label_trailing_on(values) {
                            true => {
                                let disabled = if locked(values) { ", disabled: true" } else { "" };
                                vec![format!(
                                    r#"label_trailing: rsx! {{ Button {{ variant: "text", size: "xs", aria_label: "Suggest a username"{disabled}, onclick: move |_| value.set("ada_lovelace".to_string()), "Suggest" }} }}"#
                                )]
                            }
                            false => vec![],
                        }),
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"ada\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("leading").code(|_, values| {
                        match values.str("leading").as_str() {
                            "true" => vec!["leading: rsx! { \"@\" }".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("trailing").code(|_, values| {
                        match values.str("trailing").as_str() {
                            "true" => vec![
                                "trailing: rsx! { \"{value().chars().count()}/20\" }".to_string(),
                                "maxlength: 20".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                    // The counter is text a screen reader should hear with the input.
                    Control::switch("describe_trailing").hidden_when(no_trailing),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("readonly"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<UsernameCopy>(&values);
                    let locked = locked(&values);
                    rsx! {
                    // `display: contents` keeps the field the flex item; the div finds its input.
                    div {
                        style: "display: contents",
                        onmounted: field_element.mount(),
                    TextField {
                        // Sized by content, it is narrower than the card on a phone.
                        sx: sx().width("100%").max_width("320px"),
                        size: match (responsive_on(&values), Size::from(values.str("size").as_str())) {
                            (true, size) => responsive(Size::Sm).md(size),
                            (false, size) => size.into(),
                        },
                        radius: values.str("radius"),
                        label: field.label,
                        label_trailing: label_trailing_on(&values).then(|| rsx! {
                            Button {
                                variant: "text",
                                size: "xs",
                                aria_label: "Suggest a username",
                                disabled: locked.then_some(true),
                                onclick: move |_| {
                                    value.set("ada_lovelace".to_string());
                                    let _ = field_element
                                        .query_selector("input")
                                        .and_then(|input| input.focus());
                                },
                                "Suggest"
                            }
                        }),
                        aria_label: field.aria_label,
                        description: field.description,
                        description_position: field.description_position,
                        helper: field.helper,
                        status: field.status,
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "ada".to_string()),
                        // Plain text: `Icon` colours and sizes an svg, not a prefix.
                        leading: (values.str("leading") == "true").then(|| rsx! { "@" }),
                        trailing: (values.str("trailing") == "true")
                            .then(|| rsx! { "{value().chars().count()}/20" }),
                        maxlength: (values.str("trailing") == "true").then_some(20),
                        describe_trailing: !no_trailing(&values)
                            && values.str("describe_trailing") == "true",
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        readonly: (values.str("readonly") == "true").then_some(true),
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                    }
                }
                },
            }
        }
    }
}
