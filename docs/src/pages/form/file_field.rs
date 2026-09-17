use crate::components::{Control, Demo, DemoValues, DocPage, DocSection, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, FileField, Files, Flex, Text};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

fn dropzone(values: &DemoValues) -> bool {
    values.str("variant") == "dropzone"
}

/// The prompt is `children`, so it prints inside the braces. Only the
/// `Dropzone` variant has one.
fn children_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match dropzone(values) {
        true => vec![r#"variant: "dropzone""#.to_string()],
        false => vec![],
    }
}

#[component]
pub fn FileFieldPage() -> Element {
    rsx! {
        DocPage {
            title: "FileField",
            source: "libero/src/components/form/file_field",
            markdown: "/md/file_field.md",
            properties: vec![
                props("FileField", vec![
                    prop("value", "Files")
                        .default("empty")
                        .doc("Strictly controlled. Pair it with `onchange`. Takes a `FileData`, an `Option<FileData>` or a `Vec<FileData>`."),
                    prop("multiple", "bool")
                        .default("false")
                        .doc("Lets the user pick and drop several files, each drawn as a chip. A single-file field shows the file name, and a new pick replaces it."),
                    prop("variant", "FileFieldVariant")
                        .default("input")
                        .doc("`input` is one line in the field frame. `dropzone` is a tall surface to drop onto or click, with the files as cards below it. A single-file dropzone hides its surface while it holds a file."),
                    prop("accept", "String")
                        .doc("The file types to take, such as `.pdf`, `image/png`, `image/*` or a comma-separated list. It applies to the picker and to a drop."),
                    prop("capture", "String")
                        .doc("Asks a phone for a fresh capture, `user` or `environment`."),
                    prop("placeholder", "String")
                        .doc("Shown while nothing is picked. It is the dropzone's prompt when `children` is empty. With neither, the localization's `file_field.drop_file` or `drop_files`."),
                    prop("clearable", "bool")
                        .default("true")
                        .doc("Shows an x that empties the field."),
                    prop("loading", "bool")
                        .default("false")
                        .doc("Shows a `Loader` while an upload runs and marks the field busy. It blocks nothing, `disabled` does that."),
                    prop("selection", "Callback<SelectionArgs<FileData>, Element>")
                        .default("Chip, or the file name")
                        .doc("Draws one picked file, remove control included. `args.remove` removes it."),
                    prop("name", "FieldName<Files>")
                        .doc("What the files post as. A removed file stops posting. A path such as `Claim::FIELDS.receipts()` also binds the files to the surrounding `Form`'s value when the field has no `onchange`."),
                    prop("onchange", "EventHandler<Files>")
                        .doc("Fires with the files the field should hold next, after a pick, a drop, a removal or a clear. A pick carries only the new files, so a `multiple` field that collects files merges them in its handler."),
                    prop("validate", "Validators<Files>")
                        .doc("Rules over the files, shown once the field loses focus or its form is submitted."),
                    prop("children", "Element")
                        .doc("The dropzone's prompt. The `input` variant shows `placeholder` instead."),
                    prop("size", "Size").default("md").doc("Control height, font size and the chips' size."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("label", "Caption")
                        .doc("The field's caption. It names the field and its Browse button."),
                    prop("description", "Caption").doc("Between the label and the control. Which files are wanted."),
                    prop("helper", "Caption").doc("Under the control. Size limits, formats."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables picking and dropping, and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post."),
                ]),
                props("SelectionArgs", vec![
                    prop("value", "FileData").doc("The file this call draws."),
                    prop("remove", "Callback<()>").doc("Drops this file from the value."),
                ])
                .without_base_props(),
                props("Files", vec![
                    prop("one()", "Option<FileData>").doc("The first file, a single-file field's whole value."),
                    prop("into_vec()", "Vec<FileData>").doc("Every file."),
                    prop("deref", "&[FileData]").doc("So `len`, `iter` and `is_empty` work directly."),
                ])
                .without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "Files picked from the system dialog or dropped on the control. It shows "
                    Code { source: "value" }
                    " and asks for the next files through "
                    Code { source: "onchange" }
                    ". Both carry a "
                    Code { source: "Files" }
                    ", so a single-file field and a "
                    Code { source: "multiple" }
                    " one read the same. Checks such as a size limit go in "
                    Code { source: "onchange" }
                    ", where the files are already in hand. Under Blitz the field opens the "
                    "system file dialog."
                }
            },
            Demo {
                component: "FileField",
                children_text: "",
                controls: vec![
                    Control::toggle("variant", ["input", "dropzone"])
                        .labels(["Input", "Dropzone"])
                        .default("input")
                        .code(children_code),
                    Control::slider("size", SIZES).default("md"),
                    Control::slider("radius", SIZES).default("sm"),
                    Control::toggle("accept", ["any", "image/*", ".pdf"])
                        .labels(["Any", "Images", "PDF"])
                        .default("any")
                        .code(|_, values| match values.str("accept").as_str() {
                            "any" => vec![],
                            accept => vec![format!("accept: {accept:?}")],
                        }),
                    Control::toggle("status", ["valid", "warning", "error"])
                        .default("valid")
                        .code(|_, values| match values.str("status").as_str() {
                            "warning" => vec![
                                r#"status: FieldStatus::Warning("That is a large file.".into())"#
                                    .to_string(),
                            ],
                            "error" => vec![r#"status: "We cannot read that file.""#.to_string()],
                            _ => vec![],
                        }),
                    Control::switch("multiple").code(|_, values| match is_on(values, "multiple") {
                        true => vec!["multiple: true".to_string()],
                        false => vec![],
                    }),
                    Control::switch("clearable").default("true").code(|_, values| {
                        match is_on(values, "clearable") {
                            true => vec![],
                            false => vec!["clearable: false".to_string()],
                        }
                    }),
                    Control::switch("label").default("true").code(|_, values| {
                        match is_on(values, "label") {
                            true => vec![r#"label: "Attachment""#.to_string()],
                            false => vec![r#"aria_label: "Attachment""#.to_string()],
                        }
                    }),
                    Control::switch("description").code(|_, values| {
                        match is_on(values, "description") {
                            true => vec![r#"description: "Anything under 5 MB.""#.to_string()],
                            false => vec![],
                        }
                    }),
                    Control::switch("helper").code(|_, values| match is_on(values, "helper") {
                        true => vec![r#"helper: "We keep it for 30 days.""#.to_string()],
                        false => vec![],
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("loading"),
                ],
                render: move |values: DemoValues| rsx! {
                    FileFieldDemo { values }
                },
            }
            DocSection { title: "Accessibility",
                Text {
                    "The field is a group named by its label, holding the picked files and a "
                    "Browse button. Enter or Space on the button opens the picker, and so does a "
                    "click anywhere on the field. Without a "
                    Code { source: "label" }
                    ", pass "
                    Code { source: "aria_label" }
                    ", which names the Browse button."
                }
                Text {
                    "In the "
                    Code { source: "input" }
                    " variant the files are one tab stop. Arrow Left and Arrow Right move along "
                    "them, Home and End jump to the first and last, and Backspace or Delete "
                    "removes the focused file. On the Browse button, Arrow Left moves to the last "
                    "file and Backspace removes it. In the "
                    Code { source: "dropzone" }
                    " variant each card's remove button is its own tab stop."
                }
            }
        }
    }
}

/// Its own component, so the picked files survive a control change.
#[component]
fn FileFieldDemo(values: DemoValues) -> Element {
    let mut files = use_signal(Files::default);

    let accept = match values.str("accept").as_str() {
        "any" => None,
        accept => Some(accept.to_string()),
    };
    let names: Vec<String> = files().iter().map(|file| file.name()).collect();

    rsx! {
        Flex { direction: "column", gap: "sm", sx: libero::sx::sx().width("100%"),
            FileField {
                size: values.str("size"),
                radius: values.str("radius"),
                variant: values.str("variant"),
                multiple: is_on(&values, "multiple"),
                accept,
                clearable: is_on(&values, "clearable"),
                placeholder: "No file picked",
                value: files(),
                label: is_on(&values, "label").then(|| "Attachment".to_string()),
                aria_label: (!is_on(&values, "label")).then_some("Attachment"),
                description: is_on(&values, "description")
                    .then(|| "Anything under 5 MB.".to_string()),
                helper: is_on(&values, "helper").then(|| "We keep it for 30 days.".to_string()),
                status: match values.str("status").as_str() {
                    "warning" => FieldStatus::Warning("That is a large file.".to_string()),
                    "error" => FieldStatus::Error("We cannot read that file.".to_string()),
                    _ => FieldStatus::Valid,
                },
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
                loading: is_on(&values, "loading").then_some(true),
                onchange: move |picked: Files| files.set(picked),
                "Drop files here, or click to pick"
            }
            Text { size: "sm", "value: {names:?}" }
        }
    }
}
