use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, Wrap, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, Options, Select, Text};

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

const EMPTY: &str = r#"let mut size = use_signal(|| None::<FontSize>);

Select {
    label: "Size",
    placeholder: "Pick a size",
    value: size(),
    onchange: move |next| size.set(Some(next)),
}"#;

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
    let mut empty = use_signal(|| None::<FontSize>);

    rsx! {
        DocPage {
            title: "Select",
            source: "libero/src/components/inputs/select",
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
                    prop("disabled", "bool")
                        .default("false")
                        .doc("Disables interaction and dims the select."),
                    prop("label", "String")
                        .doc("The field's own caption, above the control. The wrapper is a `<label>` either way; unset just leaves it wordless."),
                    prop("label_sx", "Sx").doc("Styles the caption alone - the rest of `sx` lands on the wrapper."),
                ]),
            ],
            lead: rsx! {
                Text {
                    "A styled native select over an enum, always wrapped in its own "
                    Code { source: "label" }
                    " element. The options are the enum's variants - "
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
                    // The label is a whole element, not a style: with it
                    // unset there is no `<span>` above the select at all.
                    Control::switch("label").default("true").code(|_, values| {
                        match values.str("label").as_str() {
                            "true" => vec!["label: \"Size\"".to_string()],
                            _ => vec![],
                        }
                    }),
                    Control::switch("disabled"),
                ],
                render: move |values: DemoValues| rsx! {
                    Select {
                        size: values.str("size"),
                        radius: values.str("radius"),
                        label: (values.str("label") == "true").then(|| "Size".to_string()),
                        disabled: (values.str("disabled") == "true").then_some(true),
                        value: value(),
                        onchange: move |next| value.set(Some(next)),
                    }
                },
            }
            DocSection {
                title: "Nothing picked yet",
                Text {
                    Code { source: "value" }
                    " is an "
                    Code { source: "Option" }
                    ", so a field the user has not filled in is a state the type can hold "
                    "rather than a sentinel option in the list. While it is "
                    Code { source: "None" }
                    " the "
                    Code { source: "placeholder" }
                    " shows as the selected entry, disabled and hidden - so the native control "
                    "cannot silently take the first option, and once a real value is picked "
                    "there is no way back to it."
                }
                Code { source: EMPTY }
                Select {
                    label: "Size",
                    placeholder: "Pick a size",
                    value: empty(),
                    onchange: move |next| empty.set(Some(next)),
                }
            }
        }
    }
}
