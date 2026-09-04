# Tooltip

Crate: `libero`
Import: `use libero::components::Tooltip;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/overlay/tooltip.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A CSS-only label that appears while its child is hovered or focused.

A label that appears while its child is hovered or focused. Pure CSS - it wraps
the trigger in a `<span>` and needs no state, so there are no open/close
callbacks. `gap` is rendered as transparent padding, not empty space, so the
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
            placement: "top",
            size: "sm",
            gap: "xs",
            Button { variant: "outlined", "Save" }
        }
    }
}
```

The bubble escapes the trigger's box, so leave it room: an ancestor with
`overflow: hidden` clips it.

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

Escape does not dismiss it, and an ancestor with `overflow: hidden` - a scroll
container, a card - clips it. Both need measurement and state; reach for a
popover there.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Element` | required | The bubble's content. |
| `placement` | `TooltipPlacement` | `top` | Which side of the trigger the bubble sits on, centred on that side. No viewport flipping. |
| `gap` | `Size` | `xs` | Distance to the trigger, rendered as transparent padding so the pointer can cross it. |
| `size` | `Size` | `sm` | Font size of the bubble. |
| `z_index` | `ThemeAwareValue` | the float layer | Overrides the stacking level, for a bubble that loses to a neighbouring overlay. |
| `open_delay` | `u32` | `0` | Milliseconds the pointer must rest before the bubble appears. |
| `close_delay` | `u32` | `0` | Milliseconds the bubble lingers after the pointer leaves. |
| `opened` | `bool` | - | Forces the bubble open or closed; unset leaves it to hover and focus. |
| `disabled` | `bool` | `false` | Renders `children` bare - no wrapper, no bubble. |
| `label_id` | `String` | - | The bubble's `id`, so the trigger can carry `aria-describedby`. |
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
| `placement` | `TooltipPlacement` | Default `placement`. |
| `gap` | `Size` | Default distance between trigger and bubble, bridged so the pointer can cross. |
| `size` | `Size` | Default `size`. |
| `open_delay` | `u32` | Default milliseconds before the bubble appears. |
| `close_delay` | `u32` | Default milliseconds before it disappears. |
| `duration` | `u32` | Fade duration, in milliseconds. |
| `font_size` | `Sizes<u16>` | Bubble font size per size step, in px. |
| `background` | `&'static str` | Bubble background. |
| `color` | `&'static str` | Bubble text color. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-tooltip-font-size-<size>` | Bubble `font-size` for that size step. |
| `--lsx-tooltip-background` | Bubble background. |
| `--lsx-tooltip-color` | Bubble text color. |
| `--lsx-tooltip-duration` | Fade duration of both directions. |
| `--lsx-tooltip-gap` | Distance to the trigger, written per instance from `gap`. |
| `--lsx-tooltip-open-delay` | Open delay, written per instance from `open_delay`. |
| `--lsx-tooltip-close-delay` | Close delay, written per instance from `close_delay`. |

The bubble sits on the shared float layer, `--lsx-z-index-float`; `z_index`
writes its `-override` twin.

## Data attributes

The wrapper and the bubble carry different tokens.

| Token | On | Condition |
|---|---|---|
| `opened` | wrapper | `opened` is `Some(true)` - forces the bubble visible. |
| `closed` | wrapper | `opened` is `Some(false)` - forces it hidden, hover included. |
| `placement-top` / `placement-right` / `placement-bottom` / `placement-left` | bubble | The `placement` in effect. |
| `size-<size>` | bubble | The `size` in effect. |

The bubble is also addressable as `[role="tooltip"]`, which is how the
wrapper's own hover rules reach it.
