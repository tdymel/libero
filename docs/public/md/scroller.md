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

On another surface, pass its colour as `fade_color`, or the control shows as a
band.

```rust,ignore
Scroller { aria_label: "Tags", fade_color: "muted.1", /* .. */ }
```

## Edges

`onedgechange` reports whether the strip rests against an end, once when it is
first measured and again whenever that changes. With `controls: "never"` a strip
can say so in its own words.

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
            onedgechange: move |next| edges.set(Some(next)),
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

To move the strip from your own buttons, pass a `use_scroller()` handle as
`handle`. `step_forward()` and `step_back()` move it as the controls do, by
`scroll_amount` and never past an end.

```rust,ignore
let strip = use_scroller();

rsx! {
    Scroller { aria_label: "Tags", controls: "never", handle: strip,
        onedgechange: move |next| edges.set(Some(next)),
        // ...
    }
    Button { onclick: move |_| strip.step_back(), "Back" }
    Button { onclick: move |_| strip.step_forward(), "Forward" }
}
```

A call before the strip has mounted does nothing.

## Accessibility

The strip is a named region and a tab stop, so `aria_label` is required. On the
focused strip `←` and `→` scroll it. `Home` and `End` do not. A control at its
own end leaves the tab order, but keeps focus if it had it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `aria_label` | `String` | required | Names the strip, which is a tab stop. |
| `scroll_amount` | `u32` | `200` | Pixels one control press scrolls. |
| `controls` | `ScrollerControls` | `auto` | `auto` shows each control while there is content that way. `always` keeps both, dimmed at their end. `never` shows neither. |
| `control_size` | `Size` | `md` | Width of each control and its glyph. |
| `fade_color` | `ThemeAwareValue` | paper background | The colour the content fades into under a control. Set it to the surface the strip sits on. |
| `draggable` | `bool` | `false` | Lets the mouse drag the strip. Touch and trackpad scroll it either way. |
| `onedgechange` | `EventHandler<ScrollerEdges>` | - | Fires when the strip reaches or leaves an end, and once when it is first measured. `ScrollerEdges { at_start, at_end }`, both `true` when nothing overflows. |
| `handle` | `ScrollerHandle` | - | From `use_scroller()`. Its `step_forward()` and `step_back()` move the strip as the controls do, for buttons of your own. |
| `children` | `Element` | required | The strip. |

Like every component, `Scroller` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes, which go on the root.

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
