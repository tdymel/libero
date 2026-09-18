# Overlay

Crate: `libero`
Import: `use libero::components::Overlay;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/overlay.rs>
Index: [index.md](index.md) lists every other page
Description: A layer that dims and blurs the page behind it, with centred content. The backdrop behind a modal, or a loading screen.

Dims and blurs whatever is behind it. A [modal](modal.md) renders one behind its
content. There is no `open` prop, so render it conditionally. Its children are
centred, which makes it a loading screen as well as a backdrop.

It covers the viewport. To keep it inside a box of your own, give that box a
`position` and a `z-index`, and the overlay `position: absolute`. Without the
`z-index`, the overlay still stacks against the whole page.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Overlay;

#[component]
fn Demo() -> Element {
    rsx! {
        Overlay { opacity: "0.6", "Loading..." }
    }
}
```

Rendered conditionally, with a backdrop click that closes it.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Overlay, Text};

#[component]
fn Demo() -> Element {
    let mut busy = use_signal(|| false);

    rsx! {
        Button { onclick: move |_| busy.set(true), "Load" }
        if busy() {
            Overlay { blur: "4px", onclick: move |_| busy.set(false),
                Text { "Loading..." }
            }
        }
    }
}
```

## Accessibility

An overlay does not trap focus or hide the page from a screen reader. For a
modal backdrop, use a [`Dialog`](dialog.md) in [`use_modal`](modal.md), which
brings its own overlay.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `z_index` | `ThemeAwareValue` | `300` | Stacking order of the layer. |
| `opacity` | `ThemeAwareValue` | `0.6` | How dark the dim is. |
| `blur` | `ThemeAwareValue` | `none` | How much the content behind is blurred. A bare number becomes `px`. |
| `onclick` | `EventHandler<MouseEvent>` | - | Called on a click anywhere on the overlay, such as a backdrop click. |
| `children` | `Element` | - | Content centred on the overlay, such as a loading spinner. |

Like every component, `Overlay` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`OverlayDefaults` on the theme holds `opacity` (`f32`) and `blur` (a
`backdrop-filter` value, `"none"` for no blur). The stacking order comes from
the theme's `z_index` scale.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-overlay-opacity` | The dim, from the theme. |
| `--lsx-overlay-opacity-override` | The `opacity` prop. |
| `--lsx-overlay-blur` | The blur, from the theme. |
| `--lsx-overlay-blur-override` | The `blur` prop, as `blur(..)`. |
| `--lsx-z-index-overlay-override` | The `z_index` prop. |

## Data attributes

None of its own. The `states` prop renders as `data-state`.
