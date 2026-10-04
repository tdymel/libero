use crate::components::{
    Control, Demo, DemoValues, DocPage, FieldCopy, Wrap, a11y, disabled_prop, field_controls,
    field_props, prop, props, required_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FieldPart;
use libero::components::{Code, NativeSelect, OptionList, Options, Text};

struct SizeCopy;

impl FieldCopy for SizeCopy {
    const LABEL: &'static str = "Size";
    const DESCRIPTION: &'static str = "Applies to body text.";
    const HELPER: &'static str = "You can change this later.";
    const WARNING: &'static str = "Large sizes reflow the page.";
    const ERROR: &'static str = "Pick a size.";
}

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
    let value = use_signal(|| Some(FontSize::Small));
    // The placeholder shows only while nothing is picked, so its case starts at `None`.
    let unpicked = use_signal(|| None::<FontSize>);

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
                    status_prop(),
                    required_prop().also("An untouched select is not announced invalid. `validate` or the surrounding `Form` enforces it."),
                    disabled_prop("field").also("A native `<select>` has no read-only state, so there is no `readonly`. Use `Select` for that."),
                ])
                .parts("FieldPart", vec![
                    (FieldPart::Label, "The label above the control."),
                    (FieldPart::Required, "The required asterisk, in the label."),
                    (FieldPart::Description, "The caption between the label and the control."),
                    (FieldPart::Frame, "The bordered box around the control."),
                    (FieldPart::Control, "The element the label names."),
                    (FieldPart::Trailing, "The slot after the control: a chevron, a toggle."),
                    (FieldPart::Helper, "The caption under the control."),
                    (FieldPart::Status, "The validation message."),
                ]),
            ],
            accessibility: a11y()
                .handles(["Your own `aria-describedby` ids come first, before the captions."])
                .must(["Without a visible `label`, set `aria_label`. A select with no name is a defect."])
                .example("A font size select with no visible label, `NativeSelect { aria_label: \"Size\" }`: a screen reader names it \"Size\", and the browser's own list handles the arrows and type-ahead."),
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
                controls: [vec![
                    Control::sizes("size")
                        .default("md"),
                    Control::sizes("radius")
                        .default("sm"),
                ], field_controls::<SizeCopy>(), vec![
                    Control::switch("placeholder").code(|_, values| {
                        match values.str("placeholder").as_str() {
                            "true" => vec!["placeholder: \"Pick a size\"".to_string()],
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
                ]].concat(),
                render: move |values: DemoValues| {
                    let field = field_props::<SizeCopy>(&values);
                    let placeholder = values.str("placeholder") == "true";
                    let mut current = if placeholder { unpicked } else { value };
                    rsx! {
                        NativeSelect {
                            placeholder: placeholder.then(|| "Pick a size".to_string()),
                            size: values.str("size"),
                            radius: values.str("radius"),
                            label: field.label,
                            aria_label: field.aria_label,
                            description: field.description,
                            helper: field.helper,
                            status: field.status,
                            required: (values.str("required") == "true").then_some(true),
                            disabled: (values.str("disabled") == "true").then_some(true),
                            options: match values.str("disabled_option").as_str() {
                                "true" => OptionList::from_options().disabling(|size| *size == FontSize::Xl),
                                _ => OptionList::from_options(),
                            },
                            value: current(),
                            onchange: move |next| current.set(Some(next)),
                        }
                    }
                },
            }
        }
    }
}
