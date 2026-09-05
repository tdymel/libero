# Divider

Crate: `libero`
Import: `use libero::components::Divider;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/divider.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A horizontal or vertical rule, with an optional label sitting in the line.

A horizontal or vertical rule, with an optional centered/positioned label.

## Usage

`spacing` is a margin and the rule has no size of its own, so both examples give
it neighbours to be measured against.

```rust
use dioxus::prelude::*;
use libero::components::{Box, Divider, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().width("240px").text_align("center"),
            Text { "Above" }
            Divider { size: "xs", spacing: "md", "OR" }
            Text { "Below" }
        }
    }
}
```

Vertical, between two inline items - it stretches to the row's height:

```rust
use dioxus::prelude::*;
use libero::components::{Box, Divider, Text};
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().display("flex").align_items("center").justify_content("center").height("64px"),
            Text { "Left" }
            Divider { orientation: "vertical", spacing: "md" }
            Text { "Right" }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `horizontal` | Horizontal or vertical rule. |
| `size` | `Size` | `xs` | Line thickness. |
| `label_position` | `LabelPosition` | `center` | Where the label sits along the rule: `center`, `start` or `end`. |
| `spacing` | `ThemeAwareValue` | `none` | Margin on either side of the rule, from the spacing scale. |
| `color` | `ThemeAwareValue` | `grey.4` | Line color. A bare theme color is tinted to shade 3. |
| `children` | `Element` | - | The optional centered/positioned label. |

Like every component, `Divider` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`DividerDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `spacing` | `Option<Size>` | Default margin either side of the rule; `None` means `0`. |
| `thicknesses` | `Sizes<u16>` | Line thickness in pixels per size step. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-divider-thickness-<size>` | Line thickness for that size step. |
| `--lsx-divider-line` | The thickness in effect, resolved from the `size` state. |
| `--lsx-divider-spacing` | Margin either side of the rule - declared by the theme, overwritten per instance by the `spacing` prop. |
| `--lsx-divider-color` | Line color; unset leaves the base `grey.4`. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `horizontal` / `vertical` | The `orientation` in effect. |
| `label` | `children` is present. |
| `label-start` | `label_position` is `start`. |
| `label-end` | `label_position` is `end`. |
