# Dialog

Crate: `libero`
Import: `use libero::components::Dialog;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/surface/dialog.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The dialog surface - padding, radius, shadow and the `role="dialog"` wiring - to pair with `Modal` for an overlay.

The dialog surface itself - padding, radius, shadow, and the `role`/`aria-modal`
wiring. Pair it with [`Modal`](modal.md) for the portaled, backdrop-dimmed,
focus-trapped overlay behaviour. `size` caps its width from the dialog scale
(`md` is 510px).

## Usage

`Dialog` does no positioning of its own. Rendered inline like this it still
carries the centering `margin` it wants inside `Modal`, so the example zeroes it
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
            Text { "Dialog rendered inline, without Modal's portal and backdrop." }
        }
    }
}
```

## Accessibility

The root is always `role="dialog"`. Nested inside a `Modal` it also gets
`aria-modal="true"` - detected from the modal's context, not from a prop. Give
it a name: `aria_label`, or an `aria-labelledby` through the pass-through
attributes pointing at your own `Title`. The focus trap and the escape/backdrop
dismissal come from `Modal`; `Dialog` has none of that on its own.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | - | Accessible name for the dialog. |
| `radius` | `ThemeAwareValue` | `md` | Corner radius - the radius scale, or any CSS length. |
| `size` | `ThemeAwareValue` | `md` | Caps the dialog's width from the dialog scale (`md` is 510px). |
| `variables` | `Variables` | - | Layered onto `Dialog`'s own - e.g. `Drawer`'s anchor/size vars. |
| `children` | `Element` | required | The dialog's content. |

Like every component, `Dialog` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`DialogDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Sizes<u16>` | Max width in px per size step - 240, 300, 510, 600, 750, 900. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-dialog-size-<size>` | Max width for that size step, from the theme. |
| `--lsx-dialog-size-override` | Set from the `size` prop; wins over the scale value. |
| `--lsx-dialog-radius` | Set from the `radius` prop; falls back to the `md` radius. |
