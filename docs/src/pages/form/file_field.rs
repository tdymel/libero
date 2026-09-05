use crate::components::{Control, Demo, DemoValues, DocPage, prop, props};
use dioxus::prelude::*;
use libero::components::{Code, FieldStatus, FileField, Files, Flex, Text};

const SIZES: [&str; 6] = ["xs", "sm", "md", "lg", "xl", "xxl"];

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

fn dropzone(values: &DemoValues) -> bool {
    values.str("variant") == "dropzone"
}

/// The prompt is `children`, so it prints inside the braces rather than as a
/// prop - and only the `Dropzone` variant has one.
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
                        .doc("Strictly controlled - pair it with `onchange`. Takes a `FileData`, an `Option<FileData>` or a `Vec<FileData>`."),
                    prop("multiple", "bool")
                        .default("false")
                        .doc("Lets the user pick and drop several files. A single-file field keeps the first of whatever it is given."),
                    prop("variant", "FileFieldVariant")
                        .default("input")
                        .doc("`input` is one line in the field frame; `dropzone` is a tall surface to drop onto or click."),
                    prop("accept", "String")
                        .doc("The `accept` attribute: `.pdf`, `image/png`, `image/*`, or a comma-separated list. The picker applies it, and so does a drop."),
                    prop("capture", "String")
                        .doc("Asks a phone for a fresh capture - `user` or `environment`."),
                    prop("placeholder", "String")
                        .doc("Shown while nothing is picked. In the `Dropzone` variant it is the prompt, when `children` is empty."),
                    prop("clearable", "bool")
                        .default("true")
                        .doc("Shows an x that empties the field."),
                    prop("loading", "bool")
                        .default("false")
                        .doc("An upload is in flight: a `Loader` beside the selection, or in place of the dropzone's icon, and `aria-busy` on the control. It blocks nothing - `disabled` does that."),
                    prop("selection", "Callback<SelectionArgs<FileData>, Element>")
                        .default("a Chip, or the filename")
                        .doc("Draws one picked file. A caller who overrides it draws the whole thing, remove control included - `args.remove` is the wiring."),
                    prop("name", "String")
                        .doc("The hidden `input[type=\"file\"]`'s name, so the files post with a form. Its list is kept equal to `value`, removals included."),
                    prop("onchange", "EventHandler<Files>")
                        .doc("Fires with the files the field should hold next - a pick, a drop, a removal or a clear. It carries what just arrived, not the union: a `multiple` field that accumulates merges in its handler."),
                    prop("children", "Element")
                        .doc("The `Dropzone` variant's prompt, between the upload glyph and the line naming what `accept` takes. Falls back to `placeholder`, then to an English default."),
                    prop("size", "Size").default("md").doc("Control height, font size and the chips' own size."),
                    prop("radius", "Size").default("sm").doc("Corner radius of the frame."),
                    prop("label", "Caption")
                        .doc("The field's caption. Names the control through `aria-labelledby` - a `div` with a role is not labelable."),
                    prop("description", "Caption").doc("Between the label and the control: which files are wanted."),
                    prop("helper", "Caption").doc("Under the control: size limits, formats."),
                    prop("status", "FieldStatus")
                        .default("Valid")
                        .doc("Validation state, rendered under the helper. A bare `&str` is an error."),
                    prop("required", "bool").default("false").doc("Adds `aria-required` to the control and an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables picking and dropping, and dims the field."),
                    prop("readonly", "bool").default("false").doc("Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post."),
                ]),
                props("SelectionArgs", vec![
                    prop("value", "FileData").doc("The file this call draws."),
                    prop("remove", "Callback<()>").doc("Drops this file from the value."),
                ])
                .without_base_props(),
            ],
            lead: rsx! {
                Text {
                    "Files picked from the system dialog or dropped on the control. "
                    "Controlled: it renders "
                    Code { source: "value" }
                    " and asks for the next set through "
                    Code { source: "onchange" }
                    ", which carries a "
                    Code { source: "Files" }
                    " - one type for both arities, so a single-file field and a "
                    Code { source: "multiple" }
                    " one read the same."
                }
                Text {
                    "Both variants drive one hidden "
                    Code { source: "input[type=\"file\"]" }
                    ", which is also what a "
                    Code { source: "name" }
                    " posts: the component writes the caller's own list back into it, so a "
                    "removed file stops posting. A drop is filtered by "
                    Code { source: "accept" }
                    ", which the picker applies by itself and a drop does not."
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
                            false => vec![],
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
        }
    }
}

/// Its own component so the picked files survive a control change - which is
/// what shows a removal, a clear and the single-file replacement.
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
