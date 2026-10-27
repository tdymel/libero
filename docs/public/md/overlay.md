# Overlay

Crate: `libero`
Import: `use libero::components::Overlay;`
Feature: `overlay-backdrop` (on by default, turns on `box`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/overlay.rs>
Index: [index.md](index.md) lists every other page
Description: A layer that dims and blurs the page behind it, with centred content. The backdrop behind a modal, or a loading screen.

Dims and blurs whatever is behind it. A [modal](modal.md) renders one behind its
content. There is no `open` prop, so render it conditionally. Its children are
centred, which makes it a loading screen as well as a backdrop. `contained`
keeps it inside a box, and `LoadingOverlay` shows one box loading.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Overlay, Paper},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Overlay {
            contained: true,
            Paper { role: "status", sx: sx().padding("8px 16px"), "Loading..." }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `z_index` | `ThemeAwareValue` | `300` | Stacking order of the layer. |
| `opacity` | `ThemeAwareValue` | `0.6` | How dark the dim is. |
| `blur` | `ThemeAwareValue` | `none` | How much the content behind is blurred. A bare number becomes `px`. |
| `contained` | `bool` | `false` | Covers its nearest positioned ancestor instead of the viewport. Give that box `position: relative` and a `z-index`, or the layer still stacks against the whole page. |
| `onclick` | `EventHandler<MouseEvent>` | - | Called on a click anywhere on the overlay, such as a backdrop click. A click inside its children bubbles here too, as in Mantine: stop it on the content, such as `onclick: move |event| event.stop_propagation()` on a card, to keep the overlay open. |
| `children` | `Element` | - | Content centred on the overlay, such as a loading spinner. |

### `LoadingOverlay`

| Prop | Type | Default | Description |
|---|---|---|---|
| `loading` | `bool` | `false` | Covers the children with a dim and a `Loader`, makes them `inert` and the box `aria-busy`, and says "Loading" once. Its feature is `loading-overlay`, off by default. |
| `children` | `Element` | required | The content that loads. `sx`, `class` and extra attributes land on the box around it. |

Like every component, `Overlay` and `LoadingOverlay` also take the shared props
`sx`, `class`, `style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `LoadingOverlay` marks its box `aria-busy` and its children `inert` while
  loading, so Tab and a screen reader skip the covered content, and a status
  region outside the box says "Loading".

### You must

- For a modal backdrop, use a [`Dialog`](dialog.md) in [`use_modal`](modal.md),
  which brings its own overlay. An overlay does not trap focus or hide the
  page from a screen reader.
- Give an overlay you close on a click a keyboard close as well, such as a
  `Button` inside it: a backdrop click alone leaves keyboard users no way out.
- Move focus out of a `LoadingOverlay`'s children before it starts loading, or
  keep it for loads the user did not start there: focus inside `inert` content
  drops to the page.

### Example

A profile card saving, `LoadingOverlay { loading: saving(), .. }` around it:
the card dims under a spinner, Tab skips its buttons, and a screen reader hears
"Loading" once.

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

`LoadingOverlay`'s overlay carries `data-slot="overlay"`. Otherwise none of their own. The `states` prop renders as `data-state`.
