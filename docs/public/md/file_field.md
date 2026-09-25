# FileField

Crate: `libero`
Import: `use libero::components::{FileField, Files};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/file_field>
Index: [index.md](index.md) lists every other page
Description: Files picked from the system dialog or dropped on the control, as a one-line input or a drop surface.

Files picked from the system dialog or dropped on the control. It shows
`value` and asks for the next files through `onchange`. Both carry a `Files`,
so a single-file field and a `multiple` one read the same. Checks such as a
size limit go in `onchange`, where the files are already in hand. Under Blitz
the field opens the system file dialog.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{FileField, Files};

#[component]
fn Demo() -> Element {
    let mut avatar = use_signal(Files::default);

    rsx! {
        FileField {
            label: "Avatar",
            accept: "image/*",
            placeholder: "No file picked",
            value: avatar(),
            onchange: move |picked: Files| avatar.set(picked),
        }
    }
}
```

A pick carries only the new files, so a `multiple` field that collects files
merges them in its handler:

```rust
use dioxus::prelude::*;
use libero::components::{FileField, Files};

#[component]
fn Demo() -> Element {
    let mut files = use_signal(Files::default);

    rsx! {
        FileField {
            label: "Attachments",
            variant: "dropzone",
            multiple: true,
            value: files(),
            onchange: move |picked: Files| {
                let mut all = files().into_vec();
                all.extend(picked.into_vec());
                files.set(all.into());
            },
            "Drop files here, or click to pick"
        }
    }
}
```

`crop` cuts a picked image before the field takes it, here to a square of at
most 512 px. [ImageCropper](image_cropper.md) explains the dialog's cropper:

```rust,ignore
FileField {
    label: "Avatar",
    accept: "image/*",
    crop: CropOptions { aspect: Some(1.0), max_size: Some(512), ..Default::default() },
    value: avatar(),
    onchange: move |picked: Files| avatar.set(picked),
}
```

## Props

### `FileField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Files` | empty | Strictly controlled. Pair it with `onchange`. Takes a `FileData`, an `Option<FileData>` or a `Vec<FileData>`. |
| `multiple` | `bool` | `false` | Lets the user pick and drop several files, each drawn as a chip. A single-file field shows the file name, and a new pick replaces it. |
| `variant` | `FileFieldVariant` | `input` | `input` is one line in the field frame. `dropzone` is a tall surface to drop onto or click, with the files as cards below it. A single-file dropzone hides its surface while it holds a file. |
| `accept` | `String` | - | The file types to take, such as `.pdf`, `image/png`, `image/*` or a comma-separated list. It applies to the picker and to a drop. |
| `capture` | `String` | - | Asks a phone for a fresh capture, `user` or `environment`. |
| `placeholder` | `String` | - | Shown while nothing is picked. It is the dropzone's prompt when `children` is empty. With neither, the localization's `file_field.drop_file` or `drop_files`. |
| `clearable` | `bool` | `true` | Shows an x that empties the field. |
| `loading` | `bool` | `false` | Shows a `Loader` while an upload runs and marks the field busy. It blocks nothing, `disabled` does that. |
| `crop` | `CropOptions` | - | A single picked or dropped image opens in an `ImageCropper` dialog first: Apply hands `onchange` the cut file, Cancel drops it. Under Blitz the field keeps the whole file. Ignored on a `multiple` field. |
| `oncrop` | `EventHandler<CropRect>` | - | The box picked in the crop dialog, before the cut file reaches `onchange`. |
| `selection` | `Callback<SelectionArgs<FileData>, Element>` | `Chip`, or the file name | Draws one picked file, remove control included. `args.remove` removes it. |
| `name` | `FieldName<Files>` | - | What the files post as. A removed file stops posting. A path such as `Claim::FIELDS.receipts()` also binds the files to the surrounding `Form`'s value when the field has no `onchange`. |
| `onchange` | `EventHandler<Files>` | - | Fires with the files the field should hold next, after a pick, a drop, a removal or a clear. A pick carries only the new files, so a `multiple` field that collects files merges them in its handler. |
| `validate` | `Validators<Files>` | - | Rules over the files, shown once the field loses focus or its form is submitted. |
| `children` | `Element` | - | The dropzone's prompt. The `input` variant shows `placeholder` instead. |
| `size` | `Size` | `md` | Control height, font size and the chips' size. |
| `radius` | `Size` | `sm` | Corner radius of the frame. |
| `label` | `Caption` | - | The field's caption. It names the field and its Browse button. |
| `description` | `Caption` | - | Between the label and the control. Which files are wanted. |
| `helper` | `Caption` | - | Under the control. Size limits, formats. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Marks the field required and adds an asterisk to the label. |
| `disabled` | `bool` | `false` | Disables picking and dropping, and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable. `disabled` instead drops the field from the tab order and from the post. |

Like every component, `FileField` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

### `SelectionArgs`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `FileData` | - | The file this call draws. |
| `remove` | `Callback<()>` | - | Drops this file from the value. |

### `Files`

| Method | Returns | Description |
|---|---|---|
| `one()` | `Option<FileData>` | The first file, a single-file field's whole value. |
| `into_vec()` | `Vec<FileData>` | Every file. |
| `deref` | `&[FileData]` | So `len`, `iter` and `is_empty` work directly. |

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `FileFieldPart::Label` | `label` | The label above the control. |
| `FileFieldPart::Required` | `required` | The required asterisk, in the label. |
| `FileFieldPart::Description` | `description` | The caption between the label and the control. |
| `FileFieldPart::Frame` | `frame` | The `Input` variant's bordered box. |
| `FileFieldPart::Control` | `control` | The group holding the Browse button: inside the frame, or the dropzone's surface. |
| `FileFieldPart::Browse` | `browse` | The Browse button. |
| `FileFieldPart::Chip` | `chip` | One picked file's chip, `Input` variant. |
| `FileFieldPart::Trailing` | `trailing` | The loader and the clear button, `Input` variant. |
| `FileFieldPart::Card` | `card` | One picked file's card under the surface, `Dropzone` variant. |
| `FileFieldPart::Helper` | `helper` | The caption under the control. |
| `FileFieldPart::Status` | `status` | The validation message. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Enter` or `Space` | On the Browse button: opens the picker. A click anywhere on the field opens it too. |
| `Left` or `Right` | `input` variant: moves along the files. |
| `Home` or `End` | `input` variant: jumps to the first or last file. |
| `Backspace` or `Delete` | `input` variant: removes the focused file. |
| `Left` | On the Browse button: moves to the last file. |
| `Backspace` | On the Browse button: removes the last file. |

### Libero handles

- The field is a group named by its label, holding the picked files and a
  Browse button.
- In the `input` variant the files are one tab stop.
- In the `dropzone` variant each card's remove button is its own tab stop.

### You must

- Without a `label`, pass `aria_label`, which names the Browse button.

## Theme defaults

`FileFieldDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `radius` | `Size` | Default `radius` when the prop is omitted. |
| `variant` | `FileFieldVariant` | Which control is drawn when the caller states no `variant`. |
| `clearable` | `bool` | Whether the clear button shows once a file is picked. |
| `dropzone_heights` | `Sizes<&'static str>` | Minimum height of the `Dropzone` surface, per size step. |

## CSS variables

The per-size scale is declared once. The level in effect resolves on the
surface and on the card list, so their children inherit it.

| Variable | Description |
|---|---|
| `--lsx-file-field-dropzone-height-<size>` | Surface height for that size step. |
| `--lsx-file-field-dropzone-height` | The height in effect. |
| `--lsx-file-field-padding` | The surface's and the cards' padding, off the field's scale. |
| `--lsx-file-field-radius` | The radius in effect, shared by the surface and the cards. |

## Data attributes

The field wrapper carries `size-*`, `radius-*`, and `disabled`, `required` and
the status token when they apply. The field's group carries the tokens below.

| Token | Condition |
|---|---|
| `multiple` | `multiple` is set, so the value slot wraps its chips. |
| `dragging` | A file is being dragged over the control. |
| `disabled` | `disabled` is set. |
