# Scroller

Crate: `libero`
Import: `use libero::components::Scroller;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroller.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A horizontal strip with a hidden scrollbar and a step control over each end that shows while there is more content that way.

A horizontal strip with its scrollbar hidden and a step control over each end,
which shows while there is more content that way. A press scrolls by
`scroll_amount` pixels. The scrolling itself is the browser's, so touch,
trackpad and the arrow keys on the focused strip work untouched. Each
control's background is a gradient from the surface colour, so the content
fades out beneath it.

The strip is a tab stop, so `aria_label` is required. On the focused strip
`←` and `→` scroll it natively. A control at its own end leaves the tab order,
but it keeps focus if it had it.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Flex, Scroller};

const TAGS: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Accessibility", "Layout", "Theming"];

#[component]
fn Demo() -> Element {
    rsx! {
        Scroller {
            aria_label: "Tags",
            Flex { direction: "row", gap: "sm", wrap: "nowrap",
                for tag in TAGS {
                    Chip { key: "{tag}", "{tag}" }
                }
            }
        }
    }
}
```

On a surface that is not the page's paper, pass that surface's colour as
`fade_color`, or the control strip shows as a band:

```rust
Scroller { aria_label: "Tags", fade_color: "grey.1", /* .. */ }
```

## Reacting to the edges

`on_edge_change` reports whether the strip rests against either end, once it
is first measured and again whenever that changes. With `controls: "never"` it
is the whole affordance, for a strip that should say so in its own words.

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Flex, Scroller, ScrollerEdges, Text};

const TAGS: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Accessibility", "Layout", "Theming"];

#[component]
fn Demo() -> Element {
    let mut edges = use_signal(|| None::<ScrollerEdges>);

    rsx! {
        Scroller {
            aria_label: "Tags",
            controls: "never",
            on_edge_change: move |next| edges.set(Some(next)),
            Flex { direction: "row", gap: "sm", wrap: "nowrap",
                for tag in TAGS {
                    Chip { key: "{tag}", "{tag}" }
                }
            }
        }
        Text { size: "sm",
            match edges() {
                Some(ScrollerEdges { at_start: true, at_end: true }) => "Everything fits",
                Some(ScrollerEdges { at_end: false, .. }) => "More to the right",
                Some(_) => "End of the list",
                None => "",
            }
        }
    }
}
```

To drive the strip from buttons of your own, create a handle with
`use_scroller()` and pass it as `handle`. `step_forward()` and `step_back()`
move the strip exactly as the built-in controls do: by `scroll_amount`, from
where the strip is now, and never past either end.

```rust
let strip = use_scroller();

rsx! {
    Scroller { aria_label: "Tags", controls: "never", handle: strip,
        on_edge_change: move |next| edges.set(Some(next)),
        // ...
    }
    Button { onclick: move |_| strip.step_back(), "Back" }
    Button { onclick: move |_| strip.step_forward(), "Forward" }
}
```

A call before the strip has mounted does nothing.

## Accessibility

- **The strip is a named region and a tab stop** (`role="region"`,
  `tabindex="0"`, the required `aria_label`). A strip of plain text or images
  has no other keyboard path.
- **`←`/`→` on the focused strip are the browser's own scrolling** - 40px per
  press in Chromium. Nothing here adds key handling, so `Home`/`End` do not
  scroll a horizontal strip.
- **The controls are `<button type="button">`** named from the theme,
  "Scroll backward" and "Scroll forward" (not "left"/"right"), each with
  `aria-controls` pointing at the strip.
- **A control at its own end is `aria-disabled="true"` and `tabindex="-1"`**,
  not `disabled`: a focused button that becomes `disabled` drops focus to the
  page. Pressing `Enter` until the end keeps focus on the control. Under
  `controls: "auto"` it then disappears once focus leaves it, not before.
- **Focus rings**: the strip's is outset, the controls' inset. Both read
  `--lsx-focus-contrast` from the surface around them; the gradient publishes
  none of its own.
- **Smooth stepping stops under `prefers-reduced-motion: reduce`.** Chrome and
  Safari do not do that for `scroll-behavior: smooth` themselves.
- **Drag** (`draggable`) is mouse only and never sets `touch-action: none`, so
  touch keeps its native scroll. A press becomes a drag only after 5px of
  travel, so a plain click still reaches the item under it; after a real drag
  the click lands on the strip, not the item. While draggable, a mouse press
  does not start a text selection and does not move focus.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | required | Names the scrollable region, which is a tab stop. |
| `scroll_amount` | `u32` | `200` | Pixels one control press scrolls. |
| `controls` | `ScrollerControls` | `auto` | `auto` shows each control while there is content that way; `always` keeps both, dimmed at their end; `never` renders neither. |
| `control_size` | `Size` | `md` | Width of each control strip and its glyph. |
| `fade_color` | `ThemeAwareValue` | paper background | What the gradient under a control fades from. Set it to the surface the strip sits on. |
| `draggable` | `bool` | `false` | Mouse drag-to-pan. Touch and trackpad scroll natively either way. |
| `on_edge_change` | `EventHandler<ScrollerEdges>` | - | Fires when either edge state flips, including the first measurement. `ScrollerEdges { at_start, at_end }`; both `true` means nothing overflows. |
| `handle` | `ScrollerHandle` | - | From `use_scroller()`. `step_forward()`/`step_back()` move the strip as the controls do. |
| `children` | `Element` | - | The strip. |

Like every component, `Scroller` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes, which go on the root.

## Theme defaults

`ScrollerDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `controls` | `ScrollerControls` | `Auto`. |
| `scroll_amount` | `u32` | `200`. |
| `control_size` | `Size` | `Md`. |
| `control_sizes` | `Sizes<u16>` | Control width per step, px: `24`, `32`, `40`, `48`, `56`, `64`. |
| `fade_color` | `&'static str` | `var(--lsx-paper-background)`, so dark mode is a change to `PaperDefaults`. |
| `draggable` | `bool` | `false`. |
| `scroll_start_label` / `scroll_end_label` | `&'static str` | `"Scroll backward"` / `"Scroll forward"`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-scroller-control-size-<size>` | Control width for that step. |
| `--lsx-scroller-control` | The active step's width, resolved on the root. |
| `--lsx-scroller-fade` | The theme's fade colour. |
| `--lsx-scroller-fade-override` | Set by `fade_color`; wins over the theme's. |

## Data attributes

| Element | Token | Condition |
|---|---|---|
| root | `size-<size>` | The `control_size` in effect. |
| root, controls | `controls-<mode>` | The `controls` mode in effect. |
| strip | `draggable` | `draggable` is on. |
| strip | `dragging` | A mouse drag is in progress. |
| control | `start` / `end` | Which end it sits at. |
| control | `disabled` | The strip rests against that end. |
