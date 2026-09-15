# FileField

Crate: `libero`
Import: `use libero::components::{FileField, Files};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/form/file_field>
Index: [index.md](index.md) - every other component's markdown page
Description: Files picked from the system dialog or dropped on the control, as a one-line input or a drop surface.

Files picked from the system dialog or dropped on the control. Controlled: it
renders `value` and asks for the next set through `onchange`, which carries a
`Files` - one type for both arities, so a single-file field and a `multiple`
one read the same.

Both variants drive one hidden `input[type="file"]`, which is also what a `name`
posts: the component writes the caller's own list back into it, so a removed
file stops posting.

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

`Files` takes a `FileData`, an `Option<FileData>` or a `Vec<FileData>`, so a
caller who holds one file needs no wrapper of their own:

```rust,ignore
let mut avatar = use_signal(|| None::<FileData>);

FileField { value: avatar(), onchange: move |files: Files| avatar.set(files.one()) }
```

## One file or several

`multiple: false` is the default, and it is its own design rather than a
degraded `multiple`: the control shows the bare filename, a second pick
replaces the first, and the clear button is the only remove. With `multiple`,
each file is a `Chip` with an x.

A drop carrying several files onto a single-file field keeps the first.

**A pick or a drop emits what just arrived, not the union.** The system picker
replaces its own selection, and the component does not second-guess that - so a
`multiple` field that should accumulate merges in its handler:

```rust,ignore
onchange: move |picked: Files| {
    let mut all = files().into_vec();
    all.extend(picked.into_vec());
    files.set(all.into())
}
```

## The dropzone variant

`variant: "dropzone"` swaps the one-line control for a tall dashed surface: an
upload glyph, a prompt, and a line naming what `accept` takes - read off the
attribute itself, so the prompt cannot promise what the picker would refuse.

The picked files are a **list of cards under the surface**, not chips inside it:
a dropzone that grows with its own contents stops being a target to aim at. Each
card carries the filename, its size and a remove button, and those buttons are
ordinary tab stops.

The prompt is `children` when the caller writes one, `placeholder` next, and an
English default last.

**A single-file dropzone puts its surface away once it holds a file**: there is
nothing left to ask for, so the card takes the surface's place and removing the
file brings it back. A `multiple` one keeps its surface, since it keeps taking
files.

```rust,ignore
FileField {
    label: "Attachments",
    variant: "dropzone",
    multiple: true,
    value: files(),
    onchange: move |picked: Files| files.set(picked),
    "Drop files here, or click to pick"
}
```

Height, padding, font size and radius all come off the field's own scale, so a
dropzone and a text field at one `size` read as a family.

The surface is `idle` or `dragging`, and **not** accept/reject while a file is
over it: `DataTransfer` exposes no item types, and browsers blank `files`
during `dragover`, so nothing about the payload is knowable until it lands.
Validity is decided on drop.

`max_size`, `max_files` and a rejection handler are not built. Check what you
need in `onchange` - the files are already in hand there.

## `accept`, and why a drop needs it twice

`accept` goes on the input, where the system picker applies it. A **drop** never
sees the picker, so the component matches dropped files against the same
attribute itself - `.pdf` suffixes, `image/png` types and `image/*` wildcards,
against the file's name and content type. A file the attribute excludes is
dropped with a dev-mode warning rather than in silence.

## Posting with a form

`name` goes on the hidden input, which is the only thing a form posts. A
`FileList` cannot be edited, so removing one file of three would otherwise leave
the input still posting all three: the component writes the caller's list back
into the input whenever the two differ. Clearing, removing and dropping all post
correctly as a result.

That write is a web capability. Off the web there is no `FileList` and no native
form post either, so nothing is lost. Under Blitz (`native` feature) the input
opens no picker, so the field opens the system file dialog; `accept` narrows
what it shows.

## Drawing a file yourself

`selection` is called once per file with `{ value, remove }` - the same
callback shape `MultiSelect` takes. Override it to draw a thumbnail, a size or
an upload progress bar; the remove control is yours to place, and `args.remove`
is the wiring.

```rust,ignore
FileField {
    multiple: true,
    value: files(),
    onchange: move |picked: Files| files.set(picked),
    selection: Callback::new(|args: SelectionArgs<FileData>| {
        let name = args.value.name();
        let size = args.value.size();
        rsx! {
            Chip { "{name} - {size} bytes" }
        }
    }),
}
```

## Accessibility

The field is a group named by its label. Inside it sit the picked files and a
Browse button, which opens the picker on Enter or Space. A click anywhere on
the field opens it too.

In the `Input` variant the files are a list with one tab stop. Left and Right
move along the files, Home and End jump to the first and the last one, and
Right past the last file returns to the Browse button. Backspace or Delete
removes the focused file. On the Browse button, Left moves to the last file
and Backspace removes it. In the `Dropzone` variant each card's remove button
is a tab stop of its own.

`required` puts a hidden "Required" in the Browse button's description. ARIA
allows `aria-required` on neither a group nor a button. A read-only field
keeps the Browse button and the files in the tab order and refuses every
edit.

`loading` does not block picking; pass `disabled` for that.

Without a `label`, pass `aria_label`: it names the Browse button, which the
field's `attributes` reach.

## Props

`FileField`

| Prop | Type | Default | Description |
|---|---|---|---|
| `value` | `Files` | empty | Strictly controlled - pair it with `onchange`. Takes a `FileData`, an `Option<FileData>` or a `Vec<FileData>`. |
| `multiple` | `bool` | `false` | Lets the user pick and drop several files. A single-file field keeps the first of whatever it is given. |
| `variant` | `FileFieldVariant` | `input` | `input` is one line in the field frame; `dropzone` is a tall surface to drop onto or click. |
| `accept` | `String` | - | The `accept` attribute: `.pdf`, `image/png`, `image/*`, or a comma-separated list. The picker applies it, and so does a drop. |
| `capture` | `String` | - | Asks a phone for a fresh capture - `user` or `environment`. |
| `placeholder` | `String` | - | Shown while nothing is picked. In the `Dropzone` variant it is the prompt, when `children` is empty. |
| `clearable` | `bool` | `true` | Shows an x that empties the field. |
| `loading` | `bool` | `false` | An upload is in flight: a `Loader` in the control and `aria-busy` on the field's group. Blocks nothing. |
| `selection` | `Callback<SelectionArgs<FileData>, Element>` | a `Chip`, or the filename | Draws one picked file, remove control included. |
| `name` | `String` | - | The hidden input's name, so the files post with a form. Its list is kept equal to `value`. |
| `onchange` | `EventHandler<Files>` | - | Fires with the files the field should hold next - a pick, a drop, a removal or a clear. |
| `children` | `Element` | - | The `Dropzone` variant's prompt. The `Input` variant shows `placeholder` instead. |
| `size` | `Size` | `md` | Control height, font size and the chips' own size. |
| `radius` | `Size` | `sm` | Corner radius of the frame. |
| `label` | `Caption` | - | The field's caption. Names the group and the Browse button through `aria-labelledby`. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `required` to the hidden input and a hidden "Required" to the Browse button's description, and marks the label. |
| `disabled` | `bool` | `false` | Disables picking and dropping, and dims the field. |
| `readonly` | `bool` | `false` | Focusable and posted with the form, but not editable - unlike `disabled`, which drops the field from the tab order and from the post. |

Like every component, `FileField` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

`Files`

| Method | Returns | Description |
|---|---|---|
| `one` | `Option<FileData>` | The first file, which is a single-file field's whole value. |
| `into_vec` | `Vec<FileData>` | Every file. |
| deref | `&[FileData]` | So `len`, `iter` and `is_empty` work directly. |

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

The per-size scale is declared once; the picked level resolves on the surface
and on the card list, so their children inherit it.

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
