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

```rust
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

```rust
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

```rust
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
form post either, so nothing is lost.

## Drawing a file yourself

`selection` is called once per file with `{ value, remove }` - the same
callback shape `MultiSelect` takes. Override it to draw a thumbnail, a size or
an upload progress bar; the remove control is yours to place, and `args.remove`
is the wiring.

```rust
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

The control is a `div` with `role="button"` rather than a `<button>`: the chips
carry their own remove buttons, and an interactive descendant of a button is
invalid. It answers Enter and Space itself, because only a real `<button>` gets
that from the browser.

A `label` names it through `aria-labelledby` - `for` names only a labelable
element. The filled caption slots join its `aria-describedby`, an error `status`
sets `aria-invalid`, and `required` sets `aria-required`. The hidden input is
`aria-hidden` and out of the tab order: it is plumbing, not the control.

**Keyboard.** Enter and Space open the picker. In the `Input` variant the chips
answer the arrows the way `MultiSelect`'s do: Left and Right move a cursor over
them, `aria-activedescendant` follows it, and Backspace or Delete removes the
file under the cursor - the last one when there is no cursor. The control stays
one tab stop; the chips' own x buttons are not in the tab order.

The `Dropzone` variant has no chips. Its cards sit outside the control, so each
card's remove button is a tab stop of its own and needs no cursor.

**Uploading.** `loading` draws a [`Loader`](loader.md) inside the control - in
the trailing slot ahead of the clear button, or in place of the dropzone's icon -
and sets `aria-busy="true"` on it. The loader is `aria-hidden`: the control is
already named by the field's label, and `aria-busy` is what says it is waiting.
Once a single-file dropzone has put its surface away, the loader moves onto the
card and the card list carries `aria-busy`. `loading` does not block picking;
pass `disabled` for that. It is not a `status`: that slot is validation, and an
upload in flight is neither valid nor invalid.

A filename is one unbreakable word, so every place one is shown - the chip, the
card, the single-file control - clips it with an ellipsis rather than letting it
widen the field.

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
| `loading` | `bool` | `false` | An upload is in flight: a `Loader` in the control and `aria-busy` on it. Blocks nothing. |
| `selection` | `Callback<SelectionArgs<FileData>, Element>` | a `Chip`, or the filename | Draws one picked file, remove control included. |
| `name` | `String` | - | The hidden input's name, so the files post with a form. Its list is kept equal to `value`. |
| `onchange` | `EventHandler<Files>` | - | Fires with the files the field should hold next - a pick, a drop, a removal or a clear. |
| `children` | `Element` | - | The `Dropzone` variant's prompt. The `Input` variant shows `placeholder` instead. |
| `size` | `Size` | `md` | Control height, font size and the chips' own size. |
| `radius` | `Size` | `sm` | Corner radius of the frame. |
| `label` | `Caption` | - | The field's caption. Names the control through `aria-labelledby`. |
| `description` | `Caption` | - | Between the label and the control. |
| `helper` | `Caption` | - | Under the control. |
| `status` | `FieldStatus` | `Valid` | Validation state, under the helper. A bare `&str` is an error. |
| `required` | `bool` | `false` | Adds `aria-required` to the control and marks the label. |
| `disabled` | `bool` | `false` | Disables picking and dropping, and dims the field. |

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
the status token when they apply. The control carries the tokens below.

| Token | Condition |
|---|---|
| `multiple` | `multiple` is set, so the value slot wraps its chips. |
| `dragging` | A file is being dragged over the control. |
| `disabled` | `disabled` is set. |
