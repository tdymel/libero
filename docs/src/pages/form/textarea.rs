use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, a11y, disabled_prop, field_controls,
    field_props, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::TextareaPart;
use libero::components::{Code, Text, Textarea};
use libero::use_theme;

struct NotesCopy;

impl FieldCopy for NotesCopy {
    const LABEL: &'static str = "Notes";
    const DESCRIPTION: &'static str = "Anything the team should know.";
    const HELPER: &'static str = "Markdown is not rendered.";
    const WARNING: &'static str = "That is getting long.";
    const ERROR: &'static str = "Say something.";
}

#[component]
pub fn TextareaPage() -> Element {
    let mut value = use_signal(String::new);
    let theme = use_theme();

    rsx! {
        DocPage {
            title: "Textarea",
            source: "libero/src/components/form/textarea.rs",
            markdown: "/md/textarea.md",
            properties: vec![
                props("Textarea", vec![
                    prop("size", "Size")
                        .default(theme.textarea.size.as_str())
                        .doc("Padding and font size."),
                    prop("radius", "ThemeAwareValue")
                        .default(theme.textarea.radius.as_str())
                        .doc("Corner radius, independent of `size`. Or any CSS, e.g. `radius: \"0\"`."),
                    prop("rows", "u32")
                        .default("3")
                        .doc("Visible lines, which set the starting height. The user can still drag it taller."),
                    prop("value", "Option<String>")
                        .doc("The text in the field. Leave it out and the textarea keeps its own text."),
                    prop("oninput", "EventHandler<String>")
                        .doc("Fires on every keystroke with the text the field should hold next."),
                    prop("validate", "Validators<String>")
                        .doc("Rules over the text, shown once the field loses focus or its form is submitted."),
                    prop("name", "FieldName<String>")
                        .doc("What the field posts as. A path such as `Signup::FIELDS.bio()` also binds the text to the surrounding `Form`'s value when the field has no `oninput`."),
                    prop("placeholder", "String")
                        .doc("Shown while the field is empty."),
                    prop("counter", "bool")
                        .default("false")
                        .doc("Shows `12/200` in the frame's bottom corner while a `maxlength` attribute is set. It counts as `maxlength` does, so an emoji counts two."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. It names the field."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to enter."),
                    prop("helper", "Caption")
                        .doc("Under the control. Formatting rules or limits."),
                    status_prop(),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required and adds an asterisk to the label. Inside a `Form`, an empty one fails the submit."),
                    disabled_prop("field"),
                    readonly_prop("field"),
                ])
                .parts("TextareaPart", vec![
                    (TextareaPart::Label, "The label above the control."),
                    (TextareaPart::Required, "The required asterisk, in the label."),
                    (TextareaPart::Description, "The caption between the label and the control."),
                    (TextareaPart::Frame, "The bordered box around the control."),
                    (TextareaPart::Control, "The element the label names."),
                    (TextareaPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (TextareaPart::Counter, "The `12/200` badge in the frame's bottom corner, with `counter`."),
                    (TextareaPart::Helper, "The caption under the control."),
                    (TextareaPart::Status, "The validation message."),
                ]).extends("textarea"),
            ],
            accessibility: a11y()
                .handles([
                    "The visible counter is hidden from screen readers. Instead, a polite status says how many characters are left once a tenth of the limit remains, one second after typing pauses. Its words come from the localization's `textarea.characters_left`.",
                    "A controlled `value` longer than `maxlength` makes the status say by how many, \"2 characters too many\", from `textarea.characters_over`, after the same pause.",
                ])
                .must(["Leave `label` unset only when something else names the field."])
                .example("A bio, `Textarea { label: \"Bio\", maxlength: 200, counter: true }`: the counter is silent while you type, and from 20 characters left a polite status says how many remain once you pause for a second."),
            lead: rsx! {
                Text {
                    "A multi-line text field with the same slots as "
                    Code { source: "TextField" }
                    ". "
                    Code { source: "rows" }
                    " sets the starting height, and the user can drag it taller. With server "
                    "rendering, the box stays empty until the app hydrates."
                }
            },
            // snippet: let mut value = use_signal(String::new);
            Demo {
                component: "Textarea",
                children_text: "",
                fixed: vec![
                    "value: value()".to_string(),
                    "oninput: move |next| value.set(next)".to_string(),
                ],
                controls: [vec![
                    Control::sizes("size")
                        .default(theme.textarea.size.as_str()),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"])
                        .default(theme.textarea.radius.as_str()),
                    // A `u32`, so it prints unquoted.
                    Control::slider("rows", ["2", "3", "5", "8"]).default("3").code(
                        |control, values| match values.str("rows") {
                            rows if rows == control.default => vec![],
                            rows => vec![format!("rows: {rows}")],
                        },
                    ),
                ], field_controls::<NotesCopy>(), vec![
                    Control::switch("placeholder").default("true").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Start typing\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("counter").code(|_, values| {
                        match values.str("counter").as_str() {
                            "true" => vec!["counter: true".to_string(), "maxlength: 200".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<NotesCopy>(&values);
                    rsx! {
                    Textarea {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        rows: values.str("rows").parse::<u32>().unwrap_or(3),
                        label: field.label,
                        aria_label: field.aria_label,
                        description: field.description,
                        helper: field.helper,
                        status: field.status,
                        placeholder: (values.str("placeholder") == "true")
                            .then(|| "Start typing".to_string()),
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        counter: values.str("counter") == "true",
                        maxlength: (values.str("counter") == "true").then_some(200),
                        value: value(),
                        oninput: move |next| value.set(next),
                    }
                }
                },
            }
        }
    }
}
