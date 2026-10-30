# FloatingActionButton

Crate: `libero`
Import: `use libero::components::FloatingActionButton;`
Feature: `floating-action-button` (off by default, turns on `action-icon`, `button`, `float`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/floating_action_button.rs>
Index: [index.md](index.md) lists every other page
Description: The main action of a screen as a button that floats over the page at a corner, icon-only or extended with a label.

The main action of a screen, as a button that floats over the page at a
corner, also called a FAB. Icon-only by default, named by `aria_label`;
`children` make it an extended button with a label. It is fixed to the
viewport, `offset` from the edges and above a docked `NavigationBar`. It is
a [Float](float.md) around an [ActionIcon](action_icon.md), or a
[Button](button.md) with `children`. There is no speed dial and no
hide-on-scroll: render it conditionally to hide it.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{FloatingActionButton, Pictogram};
use pictogram_icons_lucide as lucide;

#[component]
fn Demo() -> Element {
    rsx! {
        FloatingActionButton {
            aria_label: "New message",
            icon: rsx! { Pictogram { icon: lucide::plus::outlined } },
            onclick: move |_| {},
        }
    }
}
```

Extended, with a label that names it:

```rust
use dioxus::prelude::*;
use libero::components::{FloatingActionButton, Pictogram};
use pictogram_icons_lucide as lucide;

#[component]
fn Demo() -> Element {
    rsx! {
        FloatingActionButton {
            icon: rsx! { Pictogram { icon: lucide::pencil::outlined } },
            "Compose"
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `icon` | `Element` | required | The glyph, as `rsx! { Pictogram { icon: lucide::plus::outlined } }`. It takes `Icon`'s size at the button's step. |
| `aria_label` | `String` | - | The name of an icon-only button, after the action: "New message", not "Plus". With `children`, leave it unset: the label names the button. |
| `tooltip` | `bool` | `false` | Icon-only: shows `aria_label` in a `Tooltip` on hover and keyboard focus. |
| `variant` | `Variant` | `filled` | Visual style, as on `Button`. |
| `gradient` | `Gradient` | - | With `variant: "gradient"`: the second stop and the angle. The first stop is `color`. |
| `color` | `ThemeAwareValue` | `primary` | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `xl` | A step of `Button`'s heights, 60px at `xl`. Icon-only, a CSS length works too; with `children`, only a size word applies. |
| `radius` | `ThemeAwareValue` | `xxl` | Corner radius. The default rounds an icon-only button into a circle and an extended one into a pill; `lg` gives a rounded square. |
| `placement` | `Placement` | `bottom-end` | The corner or edge it sits at. `start` and `end` follow the text direction. |
| `offset` | `ThemeAwareValue` | `md` | Distance from the placed edges: a spacing word or any CSS length. |
| `fixed` | `bool` | `true` | Against the viewport. A bottom one sits above a docked `NavigationBar` and the bottom safe area, a top one below the top safe area. `false` places it in the nearest `position: relative` ancestor instead. |
| `z_index` | `ThemeAwareValue` | header layer | Stacking layer. The default is `Header`'s: above the page, below the focused skip link, floating windows, overlays and modals. |
| `disabled` | `bool` | `false` | Disables and dims the button. |
| `loading` | `bool` | `false` | Shows a `Loader` and ignores presses; the button stays focusable and sets `aria-busy`. Ignored on a link. |
| `loader` | `Element` | - | Drawn in place of the `Loader` while `loading`. |
| `ripple` | `bool` | `true` | The press ripple. `false` drops it. |
| `onclick` | `EventHandler<MouseEvent>` | - | Click handler. Not called on a link. |
| `to` | `NavigationTarget` | - | Renders a link instead of a `<button>`: a path, a URL or a typed route. |
| `target` | `String` | - | The link's `target` attribute. `"_blank"` adds a hidden "(opens in a new tab)". |
| `children` | `Element` | - | A visible label: an extended button, the `icon` before the label. |

Like every component, `FloatingActionButton` also takes the shared props
`sx`, `class`, `style`, `states`, and any extra HTML attributes. They land on
the button, not on the floating wrapper.

The theme's `floating_action_button` (`FloatingActionButtonDefaults`) sets
the default `variant`, `size`, `radius`, `placement`, `offset` and the resting
shadow's `elevation` step (`md`).

## Accessibility

### Libero handles

- Icon-only, it is an `ActionIcon`, named by `aria_label`; an empty one warns
  in the console.
- Its shadow steps aside on keyboard focus, so the focus ring is drawn whole
  over it.
- Under a finger the button is at least 44 by 44px, at every size.
- A fixed one sits above a docked `NavigationBar` and out of the safe areas;
  `start` and `end` flip under `dir="rtl"`.

### You must

- Name an icon-only button after its action with `aria_label`, such as "New
  message".
- Render it where its action belongs in the reading order, such as at the end
  of `main`: Tab reaches it in DOM order, not where it floats.
- Pad the end of the page by its height plus `offset`, so it never covers the
  last content or the control that has focus.
- Keep it apart from `Notifications`, whose stack also sits at `bottom-end`:
  move one with `Notifications { placement }`.

### Limits

- An ancestor with a `transform`, `filter` or `contain` traps a fixed one
  inside it, as it does any `position: fixed` box.

### Example

A mail app's compose button, `FloatingActionButton { aria_label: "New
message", icon }`: a screen reader reads "New message, button", and Tab
reaches it after the message list.

## CSS variables

None of its own; it renders a `Float` around an `ActionIcon`, or a `Button`
with `children`, whose variables apply. A fixed bottom one reads
`--lsx-navigation-bar-height`.

## Data attributes

`ActionIcon`'s or `Button`'s, on the button's `data-state`, plus `extended`
with `children`. The wrapper carries `Float`'s placement tokens.
