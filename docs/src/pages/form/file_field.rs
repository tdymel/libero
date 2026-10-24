use crate::components::{
    Child, Control, Demo, DemoValues, DocPage, DocSection, FieldCopy, a11y, field_controls,
    field_props, prop, props, readonly_prop, status_prop,
};
use dioxus::prelude::*;
use libero::components::FileFieldPart;
use libero::components::{
    Avatar, Button, Code, CodeBlock, CropOptions, FileField, FileRejection, FileTriggerArgs, Files,
    Flex, Text,
};
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

// snippet: let mut files = use_signal(Files::default);
// snippet: in FileField { label: "Attachment", value: files(), onchange: move |picked| files.set(picked), .. }
const TRIGGER_CODE: &str = r#"trigger: move |t: FileTriggerArgs| rsx! {
    Button {
        variant: "outline",
        disabled: t.disabled,
        onclick: move |_| t.open.call(()),
        attributes: t.attributes,
        "Choose a file"
    }
}"#;

/// The demo's trigger: an outline `Button` naming the held file.
fn trigger(t: FileTriggerArgs) -> Element {
    let text = match t.files.len() {
        0 => "Choose a file".to_string(),
        1 => t.files[0].name(),
        n => format!("{n} files"),
    };
    rsx! {
        Button {
            variant: "outline",
            disabled: t.disabled,
            onclick: move |_| t.open.call(()),
            attributes: t.attributes,
            "{text}"
        }
    }
}

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
                    prop("max_files", "Option<usize>")
                        .doc("With `multiple`, the most files a pick or a drop keeps; the rest are refused with `RejectReason::TooMany` and said aloud."),
                    prop("max_size", "Option<u64>")
                        .doc("The largest file in bytes a pick or a drop keeps; a larger one is refused with `RejectReason::TooLarge` and said aloud, the limit written as a size, \"5.0 MB\"."),
                    prop("variant", "FileFieldVariant")
                        .default(theme.file_field.variant.as_str())
                        .doc("`input` is one line in the field frame. `dropzone` is a tall surface to drop onto or click, with the files as cards below it. A single-file dropzone hides its surface while it holds a file."),
                    prop("accept", "String")
                        .doc("The file types to take, such as `.pdf`, `image/png`, `image/*` or a comma-separated list. It applies to the picker and to a drop: a file of another type is refused, and the field says so. A dropzone also shows them as a hint under its prompt."),
                    prop("capture", "String")
                        .doc("Asks a phone for a fresh capture, `user` or `environment`."),
                    prop("placeholder", "String")
                        .doc("The `input` variant's Browse button text while nothing is picked. It is the dropzone's prompt when `children` is empty, which a `multiple` dropzone keeps after a pick. A dropzone with neither says the localization's `file_field.drop_file` or `drop_files`; the `input` variant shows nothing."),
                    prop("clearable", "bool")
                        .default(theme.file_field.clearable.to_string())
                        .doc("Shows an x that empties the field. `input` variant only: a dropzone's cards each have their own x."),
                    prop("loading", "bool")
                        .default("false")
                        .doc("Shows a `Loader` while an upload runs and marks the field busy. It blocks nothing, `disabled` does that."),
                    prop("crop", "CropOptions")
                        .doc("A single picked or dropped image opens in an `ImageCropper` dialog first, its box centred at 80% of the largest `aspect` allows: Apply hands `onchange` the cut file, Cancel drops it. A crop that fails keeps the dialog open on an error and hands over nothing. Under Blitz an AVIF passes through uncropped, and a WebP is cut lossless, so it can be larger than on the web. Ignored on a `multiple` field. `CropOptions` sets `aspect`, `shape`, `pan` and `max_size`, as on `ImageCropper`. Needs the `image-cropper` feature: without it the file passes uncropped, with a warning in a debug build."),
                    prop("oncrop", "EventHandler<CropRect>")
                        .doc("The box picked in the crop dialog, before the cut file reaches `onchange`."),
                    prop("selection","Callback<SelectionArgs<FileData>, Element>")
                        .default("Chip, file name or card")
                        .doc("Draws one picked file, remove control included. `args.remove` removes it. Unset, a multiple `input` field draws a chip, a single one the file name, and a dropzone a card under its surface."),
                    prop("trigger", "Callback<FileTriggerArgs, Element>")
                        .doc("Draws the app's own control, an avatar or a button, in place of the Browse surface, at its own width: no chips, cards, clear x or drop target. The field keeps its label, helper, error, `name`, `accept`, `max_size`, `capture` and `crop`. Make it a focusable `button`, spread `args.attributes` onto it, and call `args.open` from its click: a browser opens no picker outside a user gesture."),
                    prop("name", "FieldName<Files>")
                        .doc("What the files post as. A removed file stops posting. A path such as `Claim::FIELDS.receipts()` also binds the files to the surrounding `Form`'s value when the field has no `onchange`."),
                    prop("onchange", "EventHandler<Files>")
                        .doc("Fires with the files the field should hold next, after a pick, a drop, a removal or a clear. A pick or a drop hands over only the files it brought, so on a `multiple` field it replaces the earlier ones, as a native file input does. A removal or a clear hands over what is left."),
                    prop("onreject", "EventHandler<Vec<FileRejection>>")
                        .doc("Fires with the files a pick or a drop brought that the field refused: a type `accept` excludes, a file past `max_files` or past the first on a field without `multiple`, or a file larger than `max_size`. Each `FileRejection` holds the `file` and a `reason`, `RejectReason::Type`, `TooMany` or `TooLarge`. The field already announces the refusal politely, in the localization's `file_field.rejected_type`, `rejected_many`, `rejected_max` and `rejected_large`. Use it to show the refusal on screen, for example in `status`."),
                    prop("validate", "Validators<Files>")
                        .doc("Rules over the files, shown once the field loses focus or its form is submitted."),
                    prop("children", "Element")
                        .doc("The dropzone's prompt. The `input` variant shows `placeholder` instead."),
                    prop("size", "Size").default(theme.file_field.size.as_str()).doc("Control height, font size and the chips' size."),
                    prop("radius", "ThemeAwareValue").default(theme.file_field.radius.as_str()).doc("Corner radius of the frame. Or any CSS, e.g. `radius: \"0\"`."),
                    prop("label", "Caption")
                        .doc("The field's caption. It names the field and its Browse button."),
                    prop("description", "Caption").doc("Between the label and the control. Which files are wanted."),
                    prop("helper", "Caption").doc("Under the control. Size limits, formats."),
                    status_prop(),
                    prop("required", "bool").default("false").doc("Marks the field required and adds an asterisk to the label; a field without a visible `label` shows none, so give a required field one. Inside a `Form`, an empty one fails the submit."),
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
                props("FileTriggerArgs", vec![
                    prop("open", "Callback<()>").doc("Opens the system picker. Call it inside the trigger's click."),
                    prop("files", "Files").doc("What the field holds, to draw an avatar or a name."),
                    prop("disabled", "bool").doc("The field is disabled or read-only: `open` does nothing."),
                    prop("loading", "bool").doc("The field's `loading`."),
                    prop("attributes", "Vec<Attribute>").doc("The field's id, its names, `aria-invalid`, `aria-busy` and the field's own attributes. Spread them onto the trigger."),
                ])
                .without_base_props(),
                props("FileRejection", vec![
                    prop("file", "FileData").doc("The refused file."),
                    prop("reason", "RejectReason").doc("`Type`: `accept` excludes it. `TooMany`: it came past `max_files`, or past the first on a field without `multiple`. `TooLarge`: it is larger than `max_size`."),
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
                .key(["Enter", "Space"], "On the Browse button: opens the picker. A click elsewhere on the field opens it too, except on a file of a `multiple` field.")
                .key(["Left", "Right"], "`input` variant with `multiple`: moves along the files.")
                .key(["Home", "End"], "`input` variant with `multiple`: jumps to the first or last file.")
                .key(["Backspace", "Delete"], "`input` variant with `multiple`: removes the focused file.")
                .key(["Left"], "On the Browse button, with `multiple`: moves to the last file.")
                .key(["Backspace", "Delete"], "On the Browse button: removes the last file.")
                .handles([
                    "The field is a group named by its label, holding the picked files and a Browse button.",
                    "In the `input` variant several files are one tab stop; a single file is none, Browse and clear act on it.",
                    "A single-file dropzone holding its file hands its label, error and description to the card's remove button.",
                    "In the `dropzone` variant each card's remove button is its own tab stop.",
                    "A `readonly` field refuses a drop or a Browse press without a message: Browse says `aria-disabled` and no file is added.",
                ])
                .must([
                    "Without a `label`, pass `aria_label`, which names the group and the Browse button.",
                    "A `trigger` is a focusable `button` carrying `args.attributes`, and calls `args.open` inside its click.",
                ])
                .example("An attachment field, `FileField { label: \"Attachment\" }`: a group named \"Attachment\". Browse is its tab stop: Enter opens the picker and Backspace removes the picked file."),
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
                    " one read the same. "
                    Code { source: "max_size" }
                    " and "
                    Code { source: "max_files" }
                    " refuse files past a limit; other checks go in "
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
                        .default(theme.file_field.variant.as_str())
                        .code(variant_code),
                    Control::sizes("size").default(theme.file_field.size.as_str()),
                    // `0` is plain CSS, next to the size scale.
                    Control::slider("radius", ["0", "xs", "sm", "md", "lg", "xl", "xxl"]).default(theme.file_field.radius.as_str()),
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
                    // Files past a limit go to `onreject`, listed under the field.
                    Control::switch("limits").code(|_, values| match is_on(values, "limits") {
                        true => vec!["max_files: 3".to_string(), "max_size: 5_000_000".to_string()],
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
                    // The app's own button in place of the surface (todo 4104).
                    Control::switch("trigger").code(|_, values| match is_on(values, "trigger") {
                        true => vec![TRIGGER_CODE.to_string()],
                        false => vec![],
                    }),
                    Control::switch("required"),
                    Control::switch("disabled"),
                    Control::switch("loading"),
                ]].concat(),
                render: move |values: DemoValues| rsx! {
                    FileFieldDemo { values }
                },
            }
            DocSection { id: "avatar-trigger", title: "Avatar trigger",
                Text {
                    "A "
                    Code { source: "trigger" }
                    " keeps its own width, so an avatar stays round. The button carries "
                    Code { source: "args.attributes" }
                    " and opens the picker in its click; the label and helper stay the field's."
                }
                AvatarTriggerDemo {}
                CodeBlock { source: AVATAR_CODE, language: "rust" }
            }
        }
    }
}

// snippet: let mut photo = use_signal(Files::default);
const AVATAR_CODE: &str = r#"FileField {
    label: "Photo",
    helper: "Click the avatar to pick a picture.",
    accept: "image/*",
    value: photo(),
    onchange: move |picked| photo.set(picked),
    trigger: move |t: FileTriggerArgs| rsx! {
        button {
            r#type: "button",
            style: "padding: 0; border: 0; background: none; border-radius: 50%; cursor: pointer",
            onclick: move |_| t.open.call(()),
            ..t.attributes,
            Avatar {
                name: t.files.one().map_or("No photo".to_string(), |file| file.name()),
                size: "xl",
            }
        }
    },
}"#;

/// `AVATAR_CODE`, live.
#[component]
fn AvatarTriggerDemo() -> Element {
    let mut photo = use_signal(Files::default);
    rsx! {
        FileField {
            label: "Photo",
            helper: "Click the avatar to pick a picture.",
            accept: "image/*",
            value: photo(),
            onchange: move |picked| photo.set(picked),
            trigger: move |t: FileTriggerArgs| rsx! {
                button {
                    r#type: "button",
                    style: "padding: 0; border: 0; background: none; border-radius: 50%; cursor: pointer",
                    onclick: move |_| t.open.call(()),
                    ..t.attributes,
                    Avatar {
                        name: t.files.one().map_or("No photo".to_string(), |file| file.name()),
                        size: "xl",
                    }
                }
            },
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
                max_files: is_on(&values, "limits").then_some(3),
                max_size: is_on(&values, "limits").then_some(5_000_000),
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
                trigger: is_on(&values, "trigger").then_some(Callback::new(trigger)),
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
