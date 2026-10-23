# Flex

Crate: `libero`
Import: `use libero::components::Flex;`
Feature: `flex` (on by default, turns on `box`; [features.md](features.md))
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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `direction` | `Responsive<FlexDirection>` | `column` | Lays the children out in a row or a column, or one per breakpoint, `direction: responsive(FlexDirection::Column).md(FlexDirection::Row)`. Each breakpoint takes that axis's theme defaults; an explicit `gap` stays. `row-reverse` and `column-reverse` draw the children backwards without moving them in the reading or Tab order. |
| `align` | `ThemeAwareValue` | `stretch` in a column, `center` in a row | Cross-axis alignment. |
| `justify` | `ThemeAwareValue` | `flex-start` | Main-axis alignment. |
| `gap` | `Responsive<ThemeAwareValue>` | `md` | Space between children: a size, any CSS such as `"0"`, or one per breakpoint, `gap: responsive(Size::Xs).md(Size::Xl)`. Breakpoints follow the window, not the parent. |
| `wrap` | `FlexWrap` | `nowrap` in a column, `wrap` in a row | Whether children wrap onto new lines. Also takes a `bool`. |
| `children` | `Element` | required | The flex's children. |

`FlexDirection` is `row`, `column`, `row-reverse` or `column-reverse`, `FlexWrap` is `wrap` or `nowrap`. A row
wraps by default so it reflows on a narrow screen. `wrap: false` turns that off.

Like every component, `Flex` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `Flex` adds no roles, and with `row` or `column` the tab and reading order
  match what is seen.
- A row wraps by default, so it reflows on a narrow screen.

### You must

- Avoid `row-reverse` and `column-reverse` for anything read or tabbed through:
  the screen shows the children backwards while a screen reader and Tab keep the
  source order (WCAG 1.3.2, 2.4.3). Reorder the children in the code instead.
- Add `role` and `aria-label` when the children form a group, such as
  `role: "group"` around related buttons.
- `Flex` always renders a `div`: for a list or a nav, use `Box` with
  `component: "ul"` or `"nav"`.

### Example

Save and Cancel in a `Flex` with `role: "group"` and `"aria-label": "Form
actions"`: a screen reader reads both as one group, in the order they are
shown.

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
| `row` | `direction` is `row` or `row-reverse` below every breakpoint. Without it, a column. |
| `row-reverse` / `column-reverse` | `direction` is that value below every breakpoint. |
| `size-<size>` | The `gap` below every breakpoint, when it is a size. |
