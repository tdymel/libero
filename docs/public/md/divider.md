# Divider

Crate: `libero`
Import: `use libero::components::Divider;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/divider.rs>
Index: [index.md](index.md) lists every other page
Description: A horizontal or vertical rule, with an optional label sitting in the line.

A horizontal or vertical rule, with an optional label sitting in the line.

## Usage

The rule has no size of its own, so both examples give it neighbours.

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
            Divider { spacing: "md", "OR" }
            Text { "Below" }
        }
    }
}
```

A vertical rule stretches to the row's height.

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
| `orientation` | `Orientation` | `horizontal` | The direction of the rule. |
| `size` | `Size` | `xs` | Line thickness. |
| `label_position` | `LabelPosition` | `center` | Where the label sits along the rule: `center`, `start` or `end`. |
| `spacing` | `ThemeAwareValue` | `none` | Margin on both sides of the rule, a spacing step or a CSS length. |
| `color` | `ThemeAwareValue` | `muted.4` | Line color. A bare theme color like `primary` resolves to its shade 3. |
| `children` | `Element` | - | An optional label in the line. |
| `parts` | `Parts<DividerPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |

Like every component, `Divider` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `DividerPart::Label` | `label` | The label between the two line halves, with `children` only. |

## Accessibility

### Libero handles

- The rule is a `separator`, named by its label. Your own `aria-label` or
  `aria-labelledby` wins.

### You must

- Pass `role: "none"` for a purely visual rule.

## Theme defaults

`DividerDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size`, `xs`. |
| `spacing` | `Option<Size>` | Default margin on both sides of the rule. `None` means `0`. |
| `thicknesses` | `Sizes<u16>` | Line thickness in pixels per size step. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-divider-thickness-<size>` | Line thickness for that size step. |
| `--lsx-divider-line` | The thickness in effect, resolved from the `size` state. |
| `--lsx-divider-spacing` | Margin on both sides of the rule. |
| `--lsx-divider-color` | Line color. Unset leaves `muted.4`. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect. |
| `horizontal` / `vertical` | The `orientation` in effect. |
| `label` | `children` is present. |
| `label-start` | `label_position` is `start`. |
| `label-end` | `label_position` is `end`. |
