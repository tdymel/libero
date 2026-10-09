# Dialog

Crate: `libero`
Import: `use libero::components::Dialog;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/dialog.rs>
Index: [index.md](index.md) lists every other page
Description: The dialog surface with a header and `role="dialog"`, which inside a modal also names and closes itself.

The dialog surface, a [`Paper`](paper.md) with a header, padding and
`role="dialog"`. It does no positioning. Open it in a modal with
[`use_modal`](modal.md), and it names itself from `title` and closes the modal
from its header button. Outside a modal the button calls `onclose`.

## Usage

Rendered inline, the close button calls `onclose`, so the example owns the open
state.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Dialog, Text};

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| true);

    rsx! {
        if open() {
            Dialog {
                title: "Unsaved changes",
                onclose: move |_| open.set(false),
                Text { "notes.md has changes you have not saved." }
            }
        } else {
            Button { variant: "outlined", onclick: move |_| open.set(true), "Reopen" }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | - | The dialog's accessible name. Wins over `title`. |
| `title` | `String` | - | Heading, and the accessible name unless `aria_label` is set. |
| `close_button` | `bool` | in a modal, or with `onclose` | Header close button. Inside a modal it closes the modal, outside one it calls `onclose`. |
| `onclose` | `EventHandler<()>` | - | Called by the close button outside a modal. |
| `ondismiss` | `Callback<Dismiss, bool>` | all close, an alertdialog's backdrop does not | Inside a modal: asked before Escape, the backdrop or Back closes it. Return `false` to keep it open, such as a form with unsaved input. The close button and your own `close()` are not asked. |
| `close_label` | `String` | `"Close"` | The close button's accessible name, such as "Close cart". Unset, the localization's `common.close`. |
| `radius` | `ThemeAwareValue` | `md` | Corner radius from the radius scale, or any CSS, e.g. `radius: "0"`. |
| `size` | `ThemeAwareValue` | `md` | Caps the width from the dialog scale. `md` is 510px. |
| `variables` | `Variables` | - | CSS variables layered onto the dialog's own, as `Drawer` does. |
| `parts` | `Parts<DialogPart>` | - | Styles for the inner parts in the Style API tab, under `sx`: `Parts::new().part(DialogPart::Title, sx().font_size("lg"))`. |
| `children` | `Element` | required | The dialog's content. |

Like every component, `Dialog` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Style API in Styling](styling.md#style-api)
explains how parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `DialogPart::Header` | `header` | The row holding the title and the close button. Rendered only with a title or a close button. |
| `DialogPart::Title` | `title` | The title heading. |
| `DialogPart::Close` | `close` | The close button. |

## Accessibility

### Libero handles

- Outside a modal, a close button without `onclose` warns in debug builds.
- `role: "alertdialog"`, as a spread attribute, makes it an alert dialog for a
  message that needs an answer, such as a delete confirmation. It ignores a
  click on the backdrop: a stray click is no answer (APG). Any other `role`
  stays `dialog`.

### You must

- Name it with `title` or `aria_label`.
- Give an `alertdialog` an `aria_describedby` pointing at its message, so a
  screen reader reads it on open.
- Open it in a modal: the focus trap, Escape and backdrop dismissal come from
  the modal. A `Dialog` on its own has none of them.
- Outside a modal, give a close button `onclose`, or it closes nothing.

### Example

A confirm dialog, `Dialog { title: "Delete file?", role: "alertdialog",
aria_describedby: "delete-message" }` inside a modal: the title names it, the
message describes it, and the modal traps focus and closes it on Escape.

## Theme defaults

`DialogDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `sizes` | `Sizes<u16>` | Max width in px per size step, 240, 300, 510, 600, 750 and 900. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-dialog-size-<size>` | Max width for that size step, from the theme. |
| `--lsx-dialog-size-override` | Set from the `size` prop. Wins over the scale value. |
| `--lsx-dialog-radius` | Set from the `radius` prop. Falls back to `--lsx-paper-radius`. |
