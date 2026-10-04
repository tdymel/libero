# Tooltip

Crate: `libero`
Import: `use libero::components::Tooltip;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/tooltip.rs>
Index: [index.md](index.md) lists every other page
Description: A label that appears while its child is hovered or focused by keyboard, portaled so nothing clips it.

A label that appears while its child is hovered or focused by keyboard. The
bubble is portaled, so no `overflow: hidden` ancestor clips it, and it flips
when its side has no room. `sx`, `class`, `states` and extra attributes land on
the bubble, not the trigger.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Button, Tooltip};

#[component]
fn Demo() -> Element {
    rsx! {
        Tooltip {
            label: rsx! { "Saves the current draft" },
            side: "top",
            size: "sm",
            gap: "xs",
            Button { variant: "outlined", "Save" }
        }
    }
}
```

Style the bubble through `sx`.

```rust
use dioxus::prelude::*;
use libero::{components::{Button, Tooltip}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Tooltip {
            label: rsx! { "Saves the current draft" },
            sx: sx().background("primary.6").white_space("normal").max_width("12rem"),
            Button { variant: "outlined", "Save" }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Element` | required | The bubble's content. |
| `side` | `Side` | `top` | The preferred side of the trigger. The bubble flips when that side has no room. `Start`/`End` are logical: `Start` is the left under `dir="ltr"`, the right under `rtl`. |
| `gap` | `Size` | `xs` | Distance to the trigger. The pointer can cross it without closing the bubble. |
| `size` | `Size` | `sm` | Font size of the bubble. |
| `z_index` | `ThemeAwareValue` | the popover layer | Overrides the stacking level, for a bubble hidden by another overlay. |
| `open_delay` | `u32` | `0` | Milliseconds the pointer must rest before the bubble appears. |
| `close_delay` | `u32` | `0` | Milliseconds the bubble stays after the pointer leaves. While it counts down, the bubble carries `data-closing`. |
| `open` | `bool` | unset | Forces the bubble open or closed. Unset, hover and focus decide. A bubble forced open ignores Escape. |
| `disabled` | `bool` | `false` | Renders `children` alone, with no wrapper and no bubble. |
| `label_id` | `String` | - | The bubble's `id`, for the trigger's `aria-describedby`. It exists while the bubble is closed, too. |
| `children` | `Element` | required | The trigger. |

The defaults come from `TooltipDefaults` on the theme.

Like every component, `Tooltip` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes. They land on the bubble.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Escape` | Hides the bubble until the pointer or focus comes back. |

### Libero handles

- Keyboard focus anywhere inside `Tooltip` shows the bubble, a click does not.
- On touch, a tap shows nothing: a 500 ms hold shows the bubble, and it stays
  1.5 s after the release.

### You must

- Give the bubble an id with `label_id` and point your trigger's
  `aria-describedby` at it, so a screen reader reads the label.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Tooltip};

#[component]
fn Demo() -> Element {
    rsx! {
        Tooltip {
            label_id: "save-tip",
            label: rsx! { "Saves the current draft" },
            Button { aria_describedby: "save-tip", "Save" }
        }
    }
}
```

### Example

A Save icon button with a "Save (Ctrl+S)" tooltip and `label_id: "save-tip"`:
the button's `aria-describedby: "save-tip"` makes a screen reader read the
tip, Tab to it shows the bubble, and Escape hides it.

## Theme defaults

`TooltipDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `side` | `Side` | Default `side`. |
| `gap` | `Size` | Default distance between trigger and bubble. |
| `size` | `Size` | Default `size`. |
| `open_delay` | `u32` | Default milliseconds before the bubble appears. |
| `close_delay` | `u32` | Default milliseconds before it disappears. |
| `duration` | `u32` | Fade-in duration, in milliseconds. |
| `font_sizes` | `Sizes<u16>` | Bubble font size per size step, in px. |
| `background` | `&'static str` | Bubble background. |
| `color` | `&'static str` | Bubble text color. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-tooltip-font-size-<size>` | Bubble `font-size` for that size step. |
| `--lsx-tooltip-background` | Bubble background. |
| `--lsx-tooltip-color` | Bubble text color. |
| `--lsx-tooltip-duration` | Fade-in duration. The bubble unmounts on close, with no fade out. |
| `--lsx-tooltip-gap` | Distance to the trigger, written per instance from `gap`. |

The bubble sits on the popover layer, `--lsx-z-index-popover`, above modals.
`z_index` overrides it.

## Data attributes

On the bubble:

| Token | Condition |
|---|---|
| `side-top` / `side-end` / `side-bottom` / `side-start` | The side it landed on, after any flip. |
| `size-<size>` | The `size` in effect. |
