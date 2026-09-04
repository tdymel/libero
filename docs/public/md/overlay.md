# Overlay

Crate: `libero`
Import: `use libero::components::Overlay;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/overlay.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A full-viewport dim and blur layer with centred content - the backdrop behind a modal, or a loading screen.

Dims and blurs whatever is behind it - a [modal](modal.md) renders one behind its content.
Render it conditionally; there is no `open`. It spans the viewport as
`position: fixed`, and children are centred in it, which is what makes it a
loading layer as well as a backdrop.

Containing an overlay inside a box of your own takes both a `position` *and* a
`z-index` on that box: `position` alone starts no stacking context, so the
overlay's `z-index: 300` would escape and compete with the whole page.

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

Conditional, with a backdrop click that closes it - there is no `open` prop:

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

An `Overlay` does not trap focus or hide the content behind it from a screen
reader, so a modal backdrop belongs with a `Dialog` in a modal rather than on
its own. `onclick` covers the backdrop click; a keyboard user needs an Escape
handler on the dialog.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `z_index` | `ThemeAwareValue` | `300` | Stacking order for the dim/blur layer. |
| `opacity` | `ThemeAwareValue` | `0.6` | Alpha of the black dim behind the content. |
| `blur` | `ThemeAwareValue` | `none` | A `backdrop-filter` blur amount. A bare number becomes `px`. |
| `onclick` | `EventHandler<MouseEvent>` | - | Fires on a click anywhere on the overlay - the backdrop-click case. |
| `children` | `Element` | - | Centred content, e.g. a loading spinner. |

Like every component, `Overlay` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`OverlayDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `opacity` | `f32` | Alpha of the black dim behind the overlay's content. |
| `blur` | `&'static str` | A `backdrop-filter` value - `"none"` for no blur. |

The stacking order comes from the theme's shared `z_index` scale, not from
`OverlayDefaults`.

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-overlay-opacity` | Dim alpha, from the theme. |
| `--lsx-overlay-opacity-override` | The `opacity` prop, set per instance. |
| `--lsx-overlay-blur` | `backdrop-filter` value, from the theme. |
| `--lsx-overlay-blur-override` | The `blur` prop, set per instance, wrapped as `blur(..)`. |
| `--lsx-z-index-overlay-override` | The `z_index` prop, set per instance. |

## Data attributes

Only what you pass: the `states` prop renders as `data-state`. `Overlay` adds no
state tokens of its own - visibility is your conditional render, not a state.
