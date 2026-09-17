# Flex

Crate: `libero`
Import: `use libero::components::Flex;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/flex.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A flexbox container - direction, gap, align, justify and wrap, all theme-aware.

A flexbox container - direction, gap, align, justify and wrap, all theme-aware.
Each axis has its own theme defaults, so a `row` and a `column` can align
differently without either naming a value.

## Usage

`align` and `justify` distribute *spare* space, so the container needs a box
bigger than its children before either does anything visible.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Flex},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex {
            sx: sx().width("400px").height("200px").padding("8px").background("muted.1"),
            Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
            Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
            Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }
        }
    }
}
```

`direction` picks the axis, and every other prop reads against it:

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Flex},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Flex {
            direction: "row",
            gap: "lg",
            justify: "space-between",
            align: "center",
            wrap: "wrap",
            sx: sx().width("400px").height("200px").padding("8px").background("muted.1"),
            Box { sx: sx().padding("8px 16px").background("primary.1"), "One" }
            Box { sx: sx().padding("8px 16px").background("primary.1"), "Two" }
            Box { sx: sx().padding("8px 16px").background("primary.1"), "Three" }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `direction` | `FlexDirection` | `column` | Row or column layout. |
| `align` | `ThemeAwareValue` | follows `direction` - `stretch` for column, `center` for row | Cross-axis alignment. |
| `justify` | `ThemeAwareValue` | `flex-start` | Main-axis alignment. |
| `gap` | `Size` | `md` | Space between children. |
| `wrap` | `FlexWrap` | follows `direction` - `nowrap` for column, `wrap` for row | Whether children wrap onto new lines. Also accepts a `bool`. |
| `divider` | `Element` | - | Rendered between each child, not before the first or after the last. Not rendered: dioxus merges the children into one node. |
| `children` | `Element` | required | The flex's children. |

`FlexDirection` is `row` or `column`; `FlexWrap` is `wrap` or `nowrap`. A row
wraps by default so it reflows on a narrow screen; `wrap: false` opts out.

Like every component, `Flex` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`FlexDefaults` on the theme, one `FlexAxisDefaults` per axis - so the two
directions can differ without a prop.

| Field | Type | Description |
|---|---|---|
| `column` | `FlexAxisDefaults` | Defaults for `direction: "column"` - `align: "stretch"`, `justify: "flex-start"`, `spacing: Size::Md`, `wrap: false`. |
| `row` | `FlexAxisDefaults` | Defaults for `direction: "row"` - `align: "center"`, `justify: "flex-start"`, `spacing: Size::Md`, `wrap: true`. |

## CSS variables

The per-axis vars carry the theme defaults; the three unprefixed ones are set in
the element's `style` from the props, so an override mints no new class.

| Variable | Description |
|---|---|
| `--lsx-flex-column-align` | Default `align-items` for a column. |
| `--lsx-flex-column-justify` | Default `justify-content` for a column. |
| `--lsx-flex-column-spacing` | Default `gap` for a column. |
| `--lsx-flex-column-wrap` | Default `flex-wrap` for a column. |
| `--lsx-flex-row-align` | Default `align-items` for a row. |
| `--lsx-flex-row-justify` | Default `justify-content` for a row. |
| `--lsx-flex-row-spacing` | Default `gap` for a row. |
| `--lsx-flex-row-wrap` | Default `flex-wrap` for a row. |
| `--lsx-flex-align` | Set from the `align` prop; falls back to the axis default. |
| `--lsx-flex-justify` | Set from the `justify` prop; falls back to the axis default. |
| `--lsx-flex-wrap` | Set from the `wrap` prop; falls back to the axis default. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `row` | `direction` is `row`; its absence is a column. |
| `size-<size>` | The `gap` in effect, when `gap` is set. |
