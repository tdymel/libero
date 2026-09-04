# Splitter

Crate: `libero`
Import: `use libero::components::{Splitter, SplitterResizeEvent};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/splitter>
Index: [index.md](index.md) - every other component's markdown page
Description: Two panes divided by a draggable, keyboard-resizable divider; nest another `Splitter` in a pane for more than two.

Splits two panes with a draggable/keyboard-resizable divider. Panes go in
`panel_a` and `panel_b`; nest another `Splitter` in a pane for more than two.

`initial_size` is the starting percentage of pane A, clamped to `min_size` at
mount. It is uncontrolled after that - the divider owns its position and
`on_resize` only notifies. A splitter is `height: 100%` and a flex container, so
it needs a parent with a real size.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Splitter}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("160px").width("320px").border("1px solid var(--lsx-grey-3)"),
            Splitter {
                initial_size: 50.0,
                aria_label: "Resize panes",
                panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } },
                panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } },
            }
        }
    }
}
```

More than two panes is one `Splitter` inside another's pane, split the other way:

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Splitter}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("160px").width("320px").border("1px solid var(--lsx-grey-3)"),
            Splitter {
                initial_size: 50.0,
                aria_label: "Resize panes",
                panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("primary.1"), "A" } },
                panel_b: rsx! {
                    Splitter {
                        orientation: "horizontal",
                        initial_size: 65.0,
                        aria_label: "Resize pane B",
                        panel_a: rsx! { Box { sx: sx().height("100%").padding("md").background("secondary.1"), "B" } },
                        panel_b: rsx! { Box { sx: sx().height("100%").padding("md").background("info.1"), "C" } },
                    }
                },
            }
        }
    }
}
```

`on_resize` reports both panes' resulting percentages, A first -
`SplitterResizeEvent::Start`/`Change`/`End`, so a drag is bracketed the same way
a scroll is.

## Accessibility

Set `aria_label`: the divider is a focusable separator, and it has no name of its
own. Name it after the pane it resizes, such as `"Resize sidebar"`.

Once the divider has focus, Arrow keys move by `SplitterDefaults::step` (1%),
Shift+Arrow by `big_step` (10%), Home and End jump to the `min_size` floor and
its mirror. Left/Right act on a vertical divider, Up/Down on a horizontal one.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `vertical` | Divider line axis - `vertical` (side-by-side panes) or `horizontal` (stacked panes). |
| `initial_size` | `f64` | required | Initial % of pane A, clamped to `min_size` at mount. Uncontrolled afterward - `on_resize` only notifies. |
| `min_size` | `f64` | `10` | % floor applied to both panes, capped at 50. |
| `divider_size` | `Size` | `sm` | Which size level the divider uses. |
| `divider_color` | `ThemeAwareValue` | - | The divider's color. Unset it is grey. |
| `on_resize` | `EventHandler<SplitterResizeEvent>` | - | Fires as the divider moves, with both panes' resulting sizes as percentages. |
| `aria_label` | `String` | - | Names the divider, after the pane it resizes. Unset warns in a debug build. |
| `panel_a` | `Element` | required | Pane A (left/top). |
| `panel_b` | `Element` | required | Pane B (right/bottom). Nest another `Splitter` in a pane for more than two. |

Like every component, `Splitter` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

`SplitterDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Which size level `divider_size` uses when unset (`sm`). |
| `divider_size` | `Sizes<u8>` | Visible line thickness in px per size step - `1, 1, 2, 3, 4, 6`. |
| `hit_size` | `Sizes<u8>` | Invisible hit-target thickness in px - `10, 10, 12, 14, 16, 20`. Fixed regardless of `divider_size`. |
| `min_size` | `f64` | Percent floor applied to both panes (`10`). |
| `step` | `f64` | Percent moved per arrow key press (`1`). |
| `big_step` | `f64` | Percent moved per Shift+arrow press (`10`). |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-splitter-divider-size-<size>` | Visible divider thickness for that size step. |
| `--lsx-splitter-hit-size-<size>` | Hit-target thickness for that size step. |
| `--lsx-splitter-divider-color` | Set by `divider_color`; unset falls back to grey. |
| `--lsx-splitter-a` | Pane A's current size as a percentage - the one value the drag writes. |

## Data attributes

State tokens on the root's `data-state`; the divider and its hit target carry
their own.

| Token | On | Condition |
|---|---|---|
| `vertical` / `horizontal` | root, divider | The `orientation` in effect. |
| `dragging` | root | A pointer drag is in progress (also disables text selection). |
| `size-<size>` | divider | The `divider_size` in effect. |
