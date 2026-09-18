use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, NativeSelect, OptionList, Options, Text};

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
pub fn NativeSelectPage() -> Element {
    let mut value = use_signal(|| Some(FontSize::Small));

    rsx! {
        DocPage {
            title: "NativeSelect",
            source: "libero/src/components/form/native_select.rs",
            markdown: "/md/native_select.md",
            properties: vec![
                props("NativeSelect", vec![
                    prop("size", "Size").default("md").doc("Height, padding and font size."),
                    prop("radius", "Size")
                        .default("sm")
                        .doc("Corner radius."),
                    prop("value", "Option<T>")
                        .doc("The selected option. Pair it with `onchange`. `None` shows `placeholder` and selects nothing."),
                    prop("onchange", "EventHandler<T>")
                        .doc("Called with the option to select next. Never for the placeholder, which cannot be picked."),
                    prop("name", "FieldName<Option<T>>")
                        .doc("What the select posts as. A path such as `Order::FIELDS.size()` also binds it to the surrounding `Form`'s value when it has no `onchange`."),
                    prop("validate", "Validators<Option<T>>")
                        .doc("Rules over the selection, shown once the select loses focus or its form is submitted."),
                    prop("options", "OptionSource<T>")
                        .default("T::options()")
                        .doc("Narrows or reorders the list. A runtime set, such as `String`s or records from a server, goes here. A `Vec<T>` converts, and an `OptionList` adds disabled options and named groups. A pending source draws no options."),
                    prop("option_label", "Callback<T, String>")
                        .default("T::label()")
                        .doc("Renames an option. Returns a `String`, since an `<option>` holds only text."),
                    prop("placeholder", "String")
                        .doc("Shown while `value` is `None`, as a first entry that cannot be picked."),
                    prop("label", "Caption")
                        .doc("The caption above the control, and the select's name. A string or an `Element`."),
                    prop("description", "Caption")
                        .doc("Between the label and the control. What to pick."),
                    prop("helper", "Caption")
                        .doc("Under the control. Constraints, or what the choice changes."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool")
                        .default("false")
                        .doc("Sets `aria-required` and marks the label. An untouched select is not announced invalid. `validate` or the surrounding `Form` enforces it."),
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables and dims the field. A native `<select>` has no read-only state, so there is no `readonly`. Use `Select` for that."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A styled native select over an enum, with a label, captions and a status "
                    "like every field. The options are the enum's variants, so "
                    Code { source: "onchange" }
                    " hands back the value itself. "
                    Code { source: "value" }
                    " is an "
                    Code { source: "Option" }
                    ", and "
                    Code { source: "None" }
                    " is a field nobody has filled in yet."
                }
            },
            // snippet: let mut value = use_signal(|| Some(FontSize::Small));
            Demo {
                component: "NativeSelect",
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
                        .labels(["Valid", "Warning", "Error"])
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
                            _ => vec!["aria_label: \"Size\"".to_string()],
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
                    Control::switch("disabled_option").code(|_, values| {
                        match values.str("disabled_option").as_str() {
                            "true" => vec![
                                "options: OptionList::from_options().disabling(|size| *size == FontSize::Xl)".to_string(),
                            ],
                            _ => vec![],
                        }
                    }),
                ],
                render: move |values: DemoValues| rsx! {
                    NativeSelect {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Size".to_string()),
                        aria_label: (values.str("label") != "true").then_some("Size"),
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
                        options: match values.str("disabled_option").as_str() {
                            "true" => OptionList::from_options().disabling(|size| *size == FontSize::Xl),
                            _ => OptionList::from_options(),
                        },
                        value: value(),
                        onchange: move |next| value.set(Some(next)),
                    }
                },
            }
            DocSection {
                title: "Accessibility",
                Text {
                    "Without a visible "
                    Code { source: "label" }
                    ", set "
                    Code { source: "aria_label" }
                    ". A select with no name is a defect. Your own "
                    Code { source: "aria-describedby" }
                    " ids come first, before the captions."
                }
            }
        }
    }
}
