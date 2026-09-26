# Flex

Crate: `libero`
Import: `use libero::components::Flex;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/flex.rs>
Index: [index.md](index.md) lists every other page
Description: A flexbox container with theme-aware direction, gap, alignment and wrapping.

A flexbox container with theme-aware direction, gap, alignment and wrapping. A
row and a column each have their own theme defaults, so a row wraps and centers
its children without naming a value.

## Usage

`align` and `justify` share out spare space, so the box is bigger than its
children.

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

`direction` picks the axis, and the other props follow it.

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
| `direction` | `FlexDirection` | `column` | Lays the children out in a row or a column. |
| `align` | `ThemeAwareValue` | `stretch` in a column, `center` in a row | Cross-axis alignment. |
| `justify` | `ThemeAwareValue` | `flex-start` | Main-axis alignment. |
| `gap` | `Size` | `md` | Space between children. |
| `wrap` | `FlexWrap` | `nowrap` in a column, `wrap` in a row | Whether children wrap onto new lines. Also takes a `bool`. |
| `children` | `Element` | required | The flex's children. |

`FlexDirection` is `row` or `column`, `FlexWrap` is `wrap` or `nowrap`. A row
wraps by default so it reflows on a narrow screen. `wrap: false` turns that off.

Like every component, `Flex` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `Flex` adds no roles and has no reverse direction, so tab and reading order
  match what is seen.
- A row wraps by default, so it reflows on a narrow screen.

### You must

- Add `role` and `aria-label` when the children form a group, such as
  `role: "group"` around related buttons.
- `Flex` always renders a `div`: for a list or a nav, use `Box` with
  `component: "ul"` or `"nav"`.

## Theme defaults

`FlexDefaults` on the theme holds one `FlexAxisDefaults` per axis.

| Field | Type | Description |
|---|---|---|
| `column` | `FlexAxisDefaults` | Defaults for a column. `align: "stretch"`, `justify: "flex-start"`, `spacing: Size::Md`, `wrap: false`. |
| `row` | `FlexAxisDefaults` | Defaults for a row. `align: "center"`, `justify: "flex-start"`, `spacing: Size::Md`, `wrap: true`. |

## CSS variables

The per-axis variables carry the theme defaults. The three without an axis come
from the props.

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
| `--lsx-flex-align` | The `align` prop, or the axis default. |
| `--lsx-flex-justify` | The `justify` prop, or the axis default. |
| `--lsx-flex-wrap` | The `wrap` prop, or the axis default. |

## Data attributes

State tokens on the root's `data-state`, space separated.

| Token | Condition |
|---|---|
| `row` | `direction` is `row`. Without it, a column. |
| `size-<size>` | The `gap` in effect, when `gap` is set. |
