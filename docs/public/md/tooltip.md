# Tooltip

Crate: `libero`
Import: `use libero::components::Tooltip;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/tooltip.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A label that appears while its child is hovered or focused by keyboard, portaled so nothing clips it.

A label that appears while its child is hovered or focused by keyboard. It
wraps the trigger in a `<span>` and portals the bubble to the document root, so
an `overflow: hidden` ancestor cannot clip it, and it flips when its side has no
room. `gap` is rendered as transparent padding, not empty space, so the
pointer can travel from the trigger into the bubble without it closing. `sx`,
`class`, `states` and spread attributes land on the bubble, not the wrapper.

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

Style the bubble - not the wrapper - through `sx`:

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

## Accessibility

Keyboard focus anywhere inside `Tooltip` shows the bubble; a click does not.
Escape hides it until the pointer or focus comes back.

The wrapper is not focusable, so an `aria-describedby` on it would never be
announced. Give the bubble an id with `label_id` and point your own trigger at
it instead.

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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Element` | required | The bubble's content. |
| `side` | `Side` | `top` | The preferred side of the trigger, centred on it. The bubble flips when that side has no room. |
| `gap` | `Size` | `xs` | Distance to the trigger, rendered as transparent padding so the pointer can cross it. |
| `size` | `Size` | `sm` | Font size of the bubble. |
| `z_index` | `ThemeAwareValue` | the popover layer | Overrides the stacking level, for a bubble that loses to a neighbouring overlay. |
| `open_delay` | `u32` | `0` | Milliseconds the pointer must rest before the bubble appears. |
| `close_delay` | `u32` | `0` | Milliseconds the bubble lingers after the pointer leaves. |
| `open` | `bool` | - | Forces the bubble open or closed; unset leaves it to hover and focus. A bubble forced open ignores Escape. |
| `disabled` | `bool` | `false` | Renders `children` bare - no wrapper, no bubble. |
| `label_id` | `String` | - | The bubble's `id`, so the trigger can carry `aria-describedby`. It resolves while the bubble is closed, too. |
| `children` | `Element` | required | The trigger. Note that `class`, `sx`, `states` and spread attributes style the *bubble*, not this. |

Every default above is the theme's, so changing `TooltipDefaults` changes them.

Like every component, `Tooltip` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes - all of which land on the
bubble.

## Theme defaults

`TooltipDefaults` on the theme; per-size font sizes live in its `font_size`
scale.

| Field | Type | Description |
|---|---|---|
| `side` | `Side` | Default `side`. |
| `gap` | `Size` | Default distance between trigger and bubble, bridged so the pointer can cross. |
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

The bubble sits on the popover layer, `--lsx-z-index-popover`, above modals;
`z_index` writes its `-override` twin.

## Data attributes

On the bubble:

| Token | Condition |
|---|---|
| `side-top` / `side-right` / `side-bottom` / `side-left` | The side it landed on, after any flip. |
| `size-<size>` | The `size` in effect. |
