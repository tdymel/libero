# DirectionToggle

Crate: `libero`
Import: `use libero::components::DirectionToggle;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/buttons/direction_toggle.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: An icon button that turns the app's text between left to right and right to left.

An icon button that turns the app's text between left to right and right to
left. It sets the document's `dir`, so every component and every overlay turns
with it, and the web keeps the choice in `localStorage` for the next visit.
The arrow shows where the next press goes.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::DirectionToggle;

#[component]
fn Demo() -> Element {
    rsx! {
        DirectionToggle {}
    }
}
```

## Your own control

Place it and you are done: it needs no state of its own. For your own control,
build on `use_direction()`; the start direction is `LiberoProvider { direction }`,
and a kept choice wins over it. The button names come from the
[localization](localization.md).

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `Variant` | `outlined` | Visual style, as on `ActionIcon`. |
| `color` | `ThemeAwareValue` | `muted` | Accent color. A theme color name or any CSS color. |
| `size` | `ThemeAwareValue` | `md` | Button size. The icon takes 55% of it. |
| `radius` | `ThemeAwareValue` | `sm` | Corner radius, independent of `size`. |
| `label` | `Callback<Direction, String>` | - | Replaces the two built-in names. Gets the direction a press turns the text to and returns what the press does. |
| `disabled` | `bool` | `false` | Disables and dims the button. |

Like every component, `DirectionToggle` also takes the shared props `sx`,
`class`, `style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- The button's name says what a press does, from the localization's
  `DirectionToggleLabels`: `to_rtl` or `to_ltr`.
- It sets the document's `dir`, so a screen reader and every component follow
  the new direction.

### You must

- With `label`, return what the press does, not the current direction.

## Theme defaults

`DirectionToggleDefaults` on the theme, as `direction_toggle`.

| Field | Type | Description |
|---|---|---|
| `variant` | `Variant` | Default `variant` when the prop is omitted (`outlined`). |
| `color` | `Color` | Default `color` when the prop is omitted (`muted`). |

The names are `DirectionToggleLabels` in the localization: `to_rtl` and
`to_ltr`.

## CSS variables

None of its own; it renders an `ActionIcon`, whose variables apply.

## Data attributes

`ActionIcon`'s, on the root's `data-state`.
