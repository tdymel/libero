# Splitter

Crate: `libero`
Import: `use libero::components::{Splitter, SplitterResizeEvent};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/splitter>
Index: [index.md](index.md) lists every other page
Description: Two panes split by a divider you can drag or move with the keyboard. Nest another `Splitter` in a pane for more than two.

Two panes, `panel_a` and `panel_b`, split by a divider you can drag or move with
the keyboard. Nest another `Splitter` in a pane for more than two. It fills its
parent, so give the parent a size.

`initial_size` is pane A's starting size in percent. After that the divider owns
the size, and `onresize` reports it.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Splitter}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("160px").width("100%").max_width("320px").border("1px solid var(--lsx-muted-3)"),
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

For more than two panes, put a `Splitter` in another's pane, split the other way.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Splitter}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().height("160px").width("100%").max_width("320px").border("1px solid var(--lsx-muted-3)"),
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

`onresize` sends a `SplitterResizeEvent` with both panes' sizes in percent, A
first. A drag sends `Start`, then `Change`, then `End`. A key press or
double-click sends `Change` then `End`, so saving the layout on `End` catches
every resize.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `vertical` | The divider's axis. `vertical` puts the panes side by side, `horizontal` stacks them. |
| `initial_size` | `f64` | required | Pane A's starting size in percent, kept within `min_size`. After that the divider owns the size, and `onresize` reports it. |
| `min_size` | `f64` | `10` | The smallest size of either pane in percent, at most 50. |
| `divider_size` | `Size` | `sm` | Thickness of the divider line. |
| `divider_color` | `ThemeAwareValue` | - | The divider's color. Unset it is `muted.6`, 3:1 on the page and `Paper`. |
| `onresize` | `EventHandler<SplitterResizeEvent>` | - | Fires as the divider moves, with both panes' sizes in percent. A key press or double-click sends `Change` then `End`. |
| `aria_label` | `String` | - | Names the divider after the pane it resizes. A debug build warns without it. |
| `panel_a` | `Element` | required | The start or top pane. |
| `panel_b` | `Element` | required | The end or bottom pane. |

Like every component, `Splitter` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | Focuses the divider, a tab stop. |
| `Left` or `Right` | Moves a vertical divider by 1%. |
| `Up` or `Down` | Moves a horizontal divider by 1%. |
| `Shift+Left`, `Shift+Right`, `Shift+Up` or `Shift+Down` | Moves the divider by 10%. |
| `Home` or `End` | Jumps to either limit. |

### Libero handles

- The divider is a focusable separator.
- Double-clicking the divider resizes without dragging (WCAG 2.5.7): pane A
  collapses to `min_size`, and the next double-click restores it. A single
  click only focuses the divider.
- The divider's hit area is 24px thick (WCAG 2.5.8), so it takes presses about
  12px into each pane.
- Each pane scrolls its own overflow, so a pane at `min_size` never paints
  over the divider or hides its controls under the other pane.
- A debug build warns without `aria_label`.

### You must

- Set `aria_label` to name the divider after the pane it resizes, such as
  `"Resize sidebar"`. It has no name of its own.
- Keep a pane's scrollbar or edge buttons out of the 12px gutter next to the
  divider, which gets no press there. For example, use `padding: 12px` on
  that side.

## Theme defaults

`SplitterDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `divider_size`, `sm`. |
| `divider_sizes` | `Sizes<u8>` | Line thickness in px per size step: `1, 1, 2, 3, 4, 6`. |
| `hit_sizes` | `Sizes<u8>` | Hit area thickness in px per size step, `24` at every step. |
| `min_size` | `f64` | Smallest pane size in percent, `10`. |
| `step` | `f64` | Percent moved per arrow key press (`1`). |
| `big_step` | `f64` | Percent moved per Shift+arrow press (`10`). |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-splitter-divider-size-<size>` | Visible divider thickness for that size step. |
| `--lsx-splitter-hit-size-<size>` | Hit-target thickness for that size step. |
| `--lsx-splitter-divider-color` | Set by `divider_color`. Unset falls back to `muted.6`. |
| `--lsx-splitter-a` | Pane A's current size in percent. |

## Data attributes

State tokens on the root's `data-state`. The divider carries its own.

| Token | On | Condition |
|---|---|---|
| `vertical` / `horizontal` | root, divider | The `orientation` in effect. |
| `dragging` | root | A pointer drag is in progress. |
| `size-<size>` | divider | The `divider_size` in effect. |
