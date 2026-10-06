use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, FieldCopy, a11y, field_controls, field_props, prop,
    props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FileFieldPart;
use libero::components::{Code, CropOptions, FileField, FileRejection, Files, Flex, Text};
use libero::use_theme;

struct AttachmentCopy;

impl FieldCopy for AttachmentCopy {
    const LABEL: &'static str = "Attachment";
    const DESCRIPTION: &'static str = "Anything under 5 MB.";
    const HELPER: &'static str = "We keep it for 30 days.";
    const WARNING: &'static str = "That is a large file.";
    const ERROR: &'static str = "We cannot read that file.";
}

fn is_on(values: &DemoValues, name: &str) -> bool {
    values.str(name) == "true"
}

fn dropzone(values: &DemoValues) -> bool {
    values.str("variant") == "dropzone"
}

fn variant_code(_control: &Control, values: &DemoValues) -> Vec<String> {
    match dropzone(values) {
        true => vec![r#"variant: "dropzone""#.to_string()],
        false => vec![],
    }
}

/// The prompt is `children`, so it prints inside the braces. Only the
/// `Dropzone` variant has one.
fn prompt(values: &DemoValues) -> String {
    match dropzone(values) {
        true => PROMPT.to_string(),
        false => String::new(),
    }
}

const PROMPT: &str = "Drop files here, or click to pick";

#[component]
pub fn FileFieldPage() -> Element {
    let theme = use_theme();
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
                        .default(theme.file_field.variant.as_str())
                        .doc("`input` is one line in the field frame. `dropzone` is a tall surface to drop onto or click, with the files as cards below it. A single-file dropzone hides its surface while it holds a file."),
                    prop("accept", "String")
                        .doc("The file types to take, such as `.pdf`, `image/png`, `image/*` or a comma-separated list. It applies to the picker and to a drop: a file of another type is refused, and the field says so. A dropzone also shows them as a hint under its prompt."),
                    prop("capture", "String")
                        .doc("Asks a phone for a fresh capture, `user` or `environment`."),
                    prop("placeholder", "String")
                        .doc("Shown while nothing is picked. It is the dropzone's prompt when `children` is empty. A dropzone with neither says the localization's `file_field.drop_file` or `drop_files`; the `input` variant shows nothing."),
                    prop("clearable", "bool")
                        .default(theme.file_field.clearable.to_string())
                        .doc("Shows an x that empties the field. `input` variant only: a dropzone's cards each have their own x."),
                    prop("loading", "bool")
                        .default("false")
                        .doc("Shows a `Loader` while an upload runs and marks the field busy. It blocks nothing, `disabled` does that."),
                    prop("crop", "CropOptions")
                        .doc("A single picked or dropped image opens in an `ImageCropper` dialog first, its box centred at 80% of the largest `aspect` allows: Apply hands `onchange` the cut file, Cancel drops it. A crop that fails keeps the dialog open on an error and hands over nothing. Under Blitz an AVIF passes through uncropped, and a WebP is cut lossless, so it can be larger than on the web. Ignored on a `multiple` field."),
                    prop("oncrop", "EventHandler<CropRect>")
                        .doc("The box picked in the crop dialog, before the cut file reaches `onchange`."),
                    prop("selection","Callback<SelectionArgs<FileData>, Element>")
                        .default("Chip, file name or card")
                        .doc("Draws one picked file, remove control included. `args.remove` removes it. Unset, a multiple `input` field draws a chip, a single one the file name, and a dropzone a card under its surface."),
                    prop("name", "FieldName<Files>")
                        .doc("What the files post as. A removed file stops posting. A path such as `Claim::FIELDS.receipts()` also binds the files to the surrounding `Form`'s value when the field has no `onchange`."),
                    prop("onchange", "EventHandler<Files>")
                        .doc("Fires with the files the field should hold next, after a pick, a drop, a removal or a clear. A pick or a drop hands over only the files it brought, so on a `multiple` field it replaces the earlier ones, as a native file input does. A removal or a clear hands over what is left."),
                    prop("onreject", "EventHandler<Vec<FileRejection>>")
                        .doc("Fires with the files a pick or a drop brought that the field refused: a type `accept` excludes, or every file past the first on a field without `multiple`. Each `FileRejection` holds the `file` and a `reason`, `RejectReason::Type` or `TooMany`. The field already announces the refusal politely, in the localization's `file_field.rejected_type` and `rejected_many`. Use it to show the refusal on screen, for example in `status`."),
                    prop("validate", "Validators<Files>")
                        .doc("Rules over the files, shown once the field loses focus or its form is submitted."),
                    prop("children", "Element")
                        .doc("The dropzone's prompt. The `input` variant shows `placeholder` instead."),
                    prop("size", "Size").default(theme.file_field.size.as_str()).doc("Control height, font size and the chips' size."),
                    prop("radius", "Size").default(theme.file_field.radius.as_str()).doc("Corner radius of the frame."),
                    prop("label", "Caption")
                        .doc("The field's caption. It names the field and its Browse button."),
                    prop("description", "Caption").doc("Between the label and the control. Which files are wanted."),
                    prop("helper", "Caption").doc("Under the control. Size limits, formats."),
                    status_prop(),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label."),
                    prop("disabled", "bool").default("false").doc("Disables picking and dropping, and dims the field."),
                    readonly_prop("field"),
                ])
                .parts("FileFieldPart", vec![
                    (FileFieldPart::Label, "The label above the control."),
                    (FileFieldPart::Required, "The required asterisk, in the label."),
                    (FileFieldPart::Description, "The caption between the label and the control."),
                    (FileFieldPart::Frame, "The `Input` variant's bordered box."),
                    (FileFieldPart::Control, "The group holding the Browse button: inside the frame, or the dropzone's surface."),
                    (FileFieldPart::Browse, "The Browse button."),
                    (FileFieldPart::Chip, "One picked file's chip, `Input` variant."),
                    (FileFieldPart::Trailing, "The loader and the clear button, `Input` variant."),
                    (FileFieldPart::Card, "One picked file's card under the surface, `Dropzone` variant."),
                    (FileFieldPart::Helper, "The caption under the control."),
                    (FileFieldPart::Status, "The validation message."),
                ]),
                props("SelectionArgs", vec![
                    prop("value", "FileData").doc("The file this call draws."),
                    prop("remove", "Callback<()>").doc("Drops this file from the value."),
                    prop("disabled", "bool").doc("The field is disabled: `remove` does nothing, so draw no remove control."),
                    prop("readonly", "bool").doc("The field is read-only: `remove` does nothing, so draw no remove control."),
                ])
                .without_base_props(),
                props("FileRejection", vec![
                    prop("file", "FileData").doc("The refused file."),
                    prop("reason", "RejectReason").doc("`Type`: `accept` excludes it. `TooMany`: it came after the first on a field without `multiple`."),
                ])
                .without_base_props(),
                props("Files", vec![
                    prop("one()", "Option<FileData>").doc("The first file, a single-file field's whole value."),
                    prop("into_vec()", "Vec<FileData>").doc("Every file."),
                    prop("deref", "&[FileData]").doc("So `len`, `iter` and `is_empty` work directly."),
                ])
                .without_base_props(),
            ],
            accessibility: a11y()
                .key(["Enter", "Space"], "On the Browse button: opens the picker. A click elsewhere on the field opens it too, except on a file.")
                .key(["Left", "Right"], "`input` variant: moves along the files.")
                .key(["Home", "End"], "`input` variant: jumps to the first or last file.")
                .key(["Backspace", "Delete"], "`input` variant: removes the focused file.")
                .key(["Left"], "On the Browse button: moves to the last file.")
                .key(["Backspace"], "On the Browse button: removes the last file.")
                .handles([
                    "The field is a group named by its label, holding the picked files and a Browse button.",
                    "In the `input` variant the files are one tab stop.",
                    "In the `dropzone` variant each card's remove button is its own tab stop.",
                ])
                .must(["Without a `label`, pass `aria_label`, which names the Browse button."])
                .example("An attachment field, `FileField { label: \"Attachment\" }`: a group named \"Attachment\". The picked files are one tab stop and Browse is another; on Browse, Enter opens the picker and Backspace removes the last file."),
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
            // snippet: let mut files = use_signal(Files::default);
            Demo {
                component: "FileField",
                children_text: "",
                child: Child(prompt),
                fixed: vec![
                    r#"placeholder: "No file picked""#.to_string(),
                    "value: files()".to_string(),
                    "onchange: move |picked| files.set(picked)".to_string(),
                ],
                controls: [vec![
                    Control::toggle("variant", ["input", "dropzone"])
                        .labels(["Input", "Dropzone"])
                        .default("input")
                        .code(variant_code),
                    Control::sizes("size").default("md"),
                    Control::sizes("radius").default("sm"),
                    Control::toggle("accept", ["any", "image/*", ".pdf"])
                        .labels(["Any", "Images", "PDF"])
                        .default("any")
                        .code(|_, values| match values.str("accept").as_str() {
                            "any" => vec![],
                            accept => vec![format!("accept: {accept:?}")],
                        }),
                ], field_controls::<AttachmentCopy>(), vec![
                    Control::switch("multiple").code(|_, values| match is_on(values, "multiple") {
                        true => vec!["multiple: true".to_string()],
                        false => vec![],
                    }),
                    Control::switch("crop").code(|_, values| match is_on(values, "crop") {
                        true => vec![
                            "crop: CropOptions { aspect: Some(1.0), ..Default::default() }".to_string(),
                        ],
                        false => vec![],
                    }),
                    // A dropzone has no clear button, so the switch goes with it.
                    Control::switch("clearable")
                        .default("true")
                        .hidden_when(|values| values.str("variant") != "input")
                        .code(|_, values| {
                            match is_on(values, "clearable") || values.str("variant") != "input" {
                                true => vec![],
                                false => vec!["clearable: false".to_string()],
                            }
                        }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("loading"),
                ]].concat(),
                render: move |values: DemoValues| rsx! {
                    FileFieldDemo { values }
                },
            }
        }
    }
}

/// Its own component, so the picked files survive a control change.
#[component]
fn FileFieldDemo(values: DemoValues) -> Element {
    let field = field_props::<AttachmentCopy>(&values);
    let mut files = use_signal(Files::default);
    let mut refused = use_signal(Vec::<String>::new);

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
                crop: is_on(&values, "crop")
                    .then(|| CropOptions { aspect: Some(1.0), ..Default::default() }),
                clearable: is_on(&values, "clearable"),
                placeholder: "No file picked",
                value: files(),
                label: field.label,
                aria_label: field.aria_label,
                description: field.description,
                helper: field.helper,
                status: field.status,
                required: is_on(&values, "required").then_some(true),
                disabled: is_on(&values, "disabled").then_some(true),
                loading: is_on(&values, "loading").then_some(true),
                onchange: move |picked: Files| files.set(picked),
                onreject: move |rejected: Vec<FileRejection>| {
                    refused.set(
                        rejected
                            .iter()
                            .map(|rejection| format!("{} ({:?})", rejection.file.name(), rejection.reason))
                            .collect(),
                    );
                },
                {prompt(&values)}
            }
            Text { size: "sm", "value: {names:?}" }
            Text { size: "sm", "refused: {refused:?}" }
        }
    }
}
