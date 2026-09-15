# Dialog

Crate: `libero`
Import: `use libero::components::Dialog;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/surface/dialog.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The dialog surface - padding, radius, shadow and the `role="dialog"` wiring - which inside a modal also names and closes itself.

The dialog surface itself - padding, radius, shadow, and the `role`/`aria-modal`
wiring. It is a [`Paper`](paper.md): the background, its focus contrast and the
default radius are the surface's, and only the chrome above is its own. It
never sets `bordered`, so the surface's border colour does not reach it. Inside a modal it also names itself from `title` and closes itself from
its own header button; open one with [`use_modal`](modal.md). Outside a modal
the button calls `onclose`. `size` caps its
width from the dialog scale (`md` is 510px).

## Usage

`Dialog` does no positioning of its own. Rendered inline like this it still
carries the centering `margin` it wants inside a modal, so the example zeroes it
out.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Dialog, Text, Title},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Dialog {
            aria_label: "Dialog surface",
            sx: sx().margin("0"),
            Title { size: "lg", "Dialog surface" }
            Text { "Dialog rendered inline, without a modal's portal and backdrop." }
        }
    }
}
```

## Accessibility

Give it a name: `title`, or `aria_label`, which overrides it. The focus trap
and Escape/backdrop dismissal come from the modal layer; a `Dialog` on its own
has none of that. Outside a modal, pass `onclose` for a working close button:
`close_button: true` without it closes nothing, and warns in debug builds.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | - | Accessible name for the dialog; overrides `title` as the name. |
| `title` | `String` | - | Heading, and the accessible name unless `aria_label` overrides it. |
| `close_button` | `bool` | in a modal, or with `onclose` | Header button. Inside a modal it closes the modal, outside one it calls `onclose`. |
| `onclose` | `EventHandler<()>` | - | Called by the close button outside a modal. Inside one the button closes the modal instead. |
| `close_label` | `String` | `common.close` | Accessible name for the close button, e.g. "Close cart". Unset, the localization's `common.close` - "Close" in English. |
| `radius` | `Size` | `md` | Corner radius, a step on the radius scale. Anything else goes through `sx`. |
| `size` | `ThemeAwareValue` | `md` | Caps the dialog's width from the dialog scale (`md` is 510px). |
| `variables` | `Variables` | - | Layered onto `Dialog`'s own - e.g. `Drawer`'s anchor/size vars. |
| `children` | `Element` | required | The dialog's content. |

Like every component, `Dialog` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`DialogDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `sizes` | `Sizes<u16>` | Max width in px per size step - 240, 300, 510, 600, 750, 900. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-dialog-size-<size>` | Max width for that size step, from the theme. |
| `--lsx-dialog-size-override` | Set from the `size` prop; wins over the scale value. |
| `--lsx-dialog-radius` | Set from the `radius` prop; falls back to `--lsx-paper-radius`. |
