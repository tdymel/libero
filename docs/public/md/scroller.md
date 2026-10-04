# Scroller

Crate: `libero`
Import: `use libero::components::Scroller;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/scroller.rs>
Index: [index.md](index.md) lists every other page
Description: A horizontal strip with a hidden scrollbar and a step control over each end, shown while there is more content that way.

A horizontal strip with a hidden scrollbar and a step control over each end,
shown while there is more content that way. A press scrolls by `scroll_amount`
pixels. Touch, trackpad and the arrow keys scroll it as usual. The content fades
out under each control, so set `fade_color` to the surface the strip sits on.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Button, Chip, Flex, Scroller, ScrollerEdges, Text, use_scroller},
    sx::sx,
};

const TAGS: [&str; 6] = ["Rust", "Dioxus", "WebAssembly", "Accessibility", "Layout", "Theming"];

#[component]
fn Demo() -> Element {
    let mut edges = use_signal(|| None::<ScrollerEdges>);
    let strip = use_scroller();

    rsx! {
        Flex { direction: "column", gap: "sm", sx: sx().width("100%"),
            Scroller {
                aria_label: "Tags",
                controls: "never",
                handle: strip,
                onedgechange: move |next| edges.set(Some(next)),
                Flex { direction: "row", gap: "sm", wrap: "nowrap",
                    for tag in TAGS {
                        Chip { key: "{tag}", "{tag}" }
                    }
                }
            }
            Flex { direction: "row", gap: "sm",
                Button { variant: "outlined", onclick: move |_| strip.step_back(), "Scroll tags back" }
                Button { variant: "outlined", onclick: move |_| strip.step_forward(), "Scroll tags forward" }
            }
            Text { size: "sm", role: "status",
                match edges() {
                    Some(ScrollerEdges { at_start: true, at_end: true }) => "Everything fits",
                    Some(ScrollerEdges { at_end: false, .. }) => "More after this",
                    Some(_) => "End of the list",
                    None => "",
                }
            }
        }
    }
}
```

## Edges

`onedgechange` reports whether the strip rests against an end, printed under the
strip. With `controls: "never"`, move the strip from your own buttons through a
`use_scroller()` handle.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | required | Names the strip, a tab stop while it overflows with nothing focusable inside. |
| `scroll_amount` | `u32` | `200` | Pixels one control press scrolls. |
| `controls` | `ScrollerControls` | `auto` | `auto` shows each control while there is content that way. `always` keeps both, dimmed at their end. `never` shows neither. |
| `control_size` | `Size` | `md` | Width of each control and its glyph. |
| `fade_color` | `ThemeAwareValue` | paper background | The colour the content fades into under a control. Set it to the surface the strip sits on. |
| `draggable` | `bool` | `false` | Lets the mouse drag the strip. Touch and trackpad scroll it either way. |
| `onedgechange` | `EventHandler<ScrollerEdges>` | - | Fires when the strip reaches or leaves an end, and once when it is first measured. `ScrollerEdges { at_start, at_end }`, both `true` when nothing overflows. |
| `handle` | `ScrollerHandle` | - | From `use_scroller()`. Its `step_forward()` and `step_back()` move the strip as the controls do, for buttons of your own. |
| `children` | `Element` | required | The strip. |
| `parts` | `Parts<ScrollerPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Scroller` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes, which go on the root.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ScrollerPart::Viewport` | `viewport` | The `ScrollArea` that scrolls, the named region. |
| `ScrollerPart::Content` | `content` | The strip around the children. |
| `ScrollerPart::Control` | `control` | Both step buttons. `data-state` holds `start` or `end`. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Left` or `Right` | Scrolls the focused strip. |

### Libero handles

- The strip is a tab stop while it overflows with nothing focusable inside.
  Focusable children take the focus themselves, and the strip scrolls each
  into view.
- A control at its own end leaves the tab order, but keeps focus if it had it.

### You must

- Name the strip with `aria_label`. It is required, as the strip is a named
  region and can be a tab stop.

## Theme defaults

`ScrollerDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `controls` | `ScrollerControls` | `Auto`. |
| `scroll_amount` | `u32` | `200`. |
| `control_size` | `Size` | `Md`. |
| `control_sizes` | `Sizes<u16>` | Control width per step in px: `24`, `32`, `40`, `48`, `56`, `64`. |
| `fade_color` | `&'static str` | `var(--lsx-paper-background)`. |
| `draggable` | `bool` | `false`. |

The controls' names, "Scroll backward" and "Scroll forward", are
`ScrollerLabels` in [localization.md](localization.md).

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-scroller-control-size-<size>` | Control width for that step. |
| `--lsx-scroller-control` | The width of the active step. |
| `--lsx-scroller-fade` | The theme's fade colour. |

## Data attributes

| Element | Token | Condition |
|---|---|---|
| root | `size-<size>` | The `control_size` in effect. |
| root, controls | `controls-<mode>` | The `controls` mode in effect. |
| strip | `draggable` | `draggable` is on. |
| strip | `dragging` | A mouse drag is in progress. |
| control | `start` / `end` | Which end it sits at. |
| control | `disabled` | The strip rests against that end. |
