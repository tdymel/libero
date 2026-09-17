# Dialog

Crate: `libero`
Import: `use libero::components::Dialog;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/dialog.rs>
Index: [index.md](index.md) - every other component's markdown page
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

## Accessibility

Name it with `title` or `aria_label`. The focus trap, Escape and backdrop
dismissal come from the modal. A `Dialog` on its own has none of them. Outside
a modal, a close button without `onclose` closes nothing and warns in debug
builds.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | - | The dialog's accessible name. Wins over `title`. |
| `title` | `String` | - | Heading, and the accessible name unless `aria_label` is set. |
| `close_button` | `bool` | in a modal, or with `onclose` | Header close button. Inside a modal it closes the modal, outside one it calls `onclose`. |
| `onclose` | `EventHandler<()>` | - | Called by the close button outside a modal. |
| `close_label` | `String` | `"Close"` | The close button's accessible name, such as "Close cart". Unset, the localization's `common.close`. |
| `radius` | `Size` | `md` | Corner radius from the radius scale. Other values go through `sx`. |
| `size` | `ThemeAwareValue` | `md` | Caps the width from the dialog scale. `md` is 510px. |
| `variables` | `Variables` | - | CSS variables layered onto the dialog's own, as `Drawer` does. |
| `children` | `Element` | required | The dialog's content. |

Like every component, `Dialog` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

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
