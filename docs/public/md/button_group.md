# ButtonGroup

Crate: `libero`
Import: `use libero::components::ButtonGroup;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/button_group.rs>
Index: [index.md](index.md) lists every other page
Description: Buttons and action icons side by side as one control, sharing seams and defaults.

Buttons and action icons side by side as one control. Neighbours share one
seam and only the group's outer corners are round, on logical sides, so the
ends swap under right-to-left text.

## Usage

`CheckmarkIcon` stands for any component of yours that renders an `svg`.

```rust
use dioxus::prelude::*;
use libero::components::{ActionIcon, Button, ButtonGroup};

#[component]
fn Demo() -> Element {
    rsx! {
        ButtonGroup { "aria-label": "Edit",
            Button { "Undo" }
            Button { "Redo" }
            ActionIcon { aria_label: "Confirm", CheckmarkIcon {} }
        }
    }
}
#
# #[component] fn CheckmarkIcon() -> Element { rsx! {} }
```

## Shared defaults

`variant`, `color`, `size`, `radius` and `disabled` set the default of every
button inside; a button's own prop wins. This site's header groups its
repository link, direction toggle and theme switcher, and on a phone the search.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `horizontal` | `"vertical"` stacks the buttons, each as wide as the widest. |
| `variant` | `Variant` | - | Default `variant` of the buttons inside. Between two buttons without a visible border of their own (every variant but `outlined`), the group draws a thin divider. |
| `color` | `ThemeAwareValue` | - | Default `color` of the buttons inside. |
| `size` | `Size` | - | Default `size` of the buttons inside. |
| `radius` | `Size` | - | The group's outer corners. The corners between two buttons are always square. |
| `disabled` | `bool` | - | Disables every button inside that does not set `disabled` itself. |
| `children` | `Element` | `required` | `Button`s, `ActionIcon`s, and components built on them, such as `ThemeSwitcher`. |

Like every component, `ButtonGroup` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- The root is a `role="group"`, so a screen reader announces the buttons as
  one set.
- Each button stays its own Tab stop, and the focused one's ring is drawn above
  its neighbours.

### You must

- Name the group with `aria-label`, or `aria-labelledby` on a visible heading.

### Limits

- It is not a `toolbar`: no arrow-key navigation between the buttons.
- For one choice out of several, use [`SegmentedControl`](segmented_control.md),
  which is a radio group.
- A child hidden with `display: none` still counts as first or last and squares its
  neighbour's outer corners; render a conditional control only when shown.

## CSS variables

None of its own; the buttons inside keep theirs.

## Data attributes

`horizontal` or `vertical` on the root's `data-state`.
