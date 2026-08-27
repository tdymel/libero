use crate::components::{Control, Demo, DemoValues, DocPage, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, Options, Select, Text};

/// The enum is the option list, so the snippet has to show it.
const SIZE_ENUM: &str = r#"#[derive(Clone, Copy, PartialEq, Options)]
enum FontSize {
    #[option(label = "Extra small")]
    Xs,
    Small,
    Medium,
    Large,
    #[option(label = "Extra large")]
    Xl,
}

"#;

#[derive(Clone, Copy, PartialEq, Options)]
enum FontSize {
    #[option(label = "Extra small")]
    Xs,
    Small,
    Medium,
    Large,
    #[option(label = "Extra large")]
    Xl,
}

#[component]
pub fn SelectPage() -> Element {
    let mut value = use_signal(|| Some(FontSize::Small));

    rsx! {
        DocPage {
            title: "Select",
            source: "libero/src/components/form/select.rs",
            markdown: "/md/select.md",
            properties: vec![
                props("Select", vec![
                    prop("size", "Size").default("md").doc("Controls height, padding, and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius, independent of size."),
                    prop("value", "Option<T>")
                        .doc("The selected option; strictly controlled. `None` shows `placeholder` and selects nothing."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the option the caller should select next. Never fires for the placeholder, which cannot be picked."),
                    prop("options", "Vec<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set - `String`s, or records fetched from a server - passes them here, since only an enum lists its own."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Overrides what the derive named an option. Returns a `String`, not an `OptionLabel`: an `<option>` holds text and nothing else."),
                    prop("placeholder", "String")
                        .doc("Shown while `value` is `None`, as an unpickable first entry."),
                    prop("label", "Caption")
                        .doc("The field's caption, above the control. Names the field through a `for`/`id` pair. Takes a string or an `Element`."),
                    prop("description", "Caption")
                        .doc("Between the label and the control: what to pick."),
                    prop("helper", "Caption")
                        .doc("Under the control: constraints, or what the choice affects."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Marks the field required, adds `aria-required` and shows an asterisk in the label."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the field."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A styled native select over an enum, with the five slots every field "
                    "shares: label, description, the control, helper text, and a validation "
                    "message. The options are the enum's variants - "
                    Code { source: "#[derive(Options)]" }
                    " lists them in declaration order - so "
                    Code { source: "onchange" }
                    " hands back the value itself rather than a string to look up again. "
                    "Strictly controlled: "
                    Code { source: "value" }
                    " drives it, and "
                    Code { source: "None" }
                    " is a real state - the field nobody has filled in yet."
                }
            },
            Demo {
                component: "Select",
                children_text: "",
                // Printed above the snippet: the list is the enum, so the
                // code block is a lie without it.
                wrap: Wrap(|_: &DemoValues, source: &str| format!("{SIZE_ENUM}{source}")),
                fixed: vec![
                    "value: value()".to_string(),
                    "onchange: move |next| value.set(Some(next))".to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("md"),
                    Control::slider("radius", ["xs", "sm", "md", "lg", "xl", "xxl"])
                        .default("sm"),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                "status: FieldStatus::Warning(\"Large sizes reflow the page.\".into())".to_string(),
                            ],
                            "error" => vec!["status: \"Pick a size.\"".to_string()],
                            _ => vec![],
                        }),
                    // Each caption is a whole element, not a style: unset,
                    // the slot is not in the markup at all.
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Size\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match values.str("description").as_str() {
                            "true" => vec!["description: \"Applies to body text.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| {
                        match values.str("helper").as_str() {
                            "true" => vec!["helper: \"You can change this later.\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Select {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Size".to_string()),
                        description: (values.str("description") == "true")
                            .then(|| "Applies to body text.".to_string()),
                        helper: (values.str("helper") == "true")
                            .then(|| "You can change this later.".to_string()),
                        status: match values.str("status").as_str() {
                            "warning" => {
                                FieldStatus::Warning("Large sizes reflow the page.".to_string())
                            }
                            "error" => FieldStatus::Error("Pick a size.".to_string()),
                            _ => FieldStatus::Valid,
                        },
                        required: (values.str("required") == "true").then_some(true),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        onchange: move |next| value.set(Some(next)),
                    }
                },
            }
        }
    }
}
