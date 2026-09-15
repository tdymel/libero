# Paper

Crate: `libero`
Import: `use libero::components::Paper;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/surface/paper.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The library's surface - a background, a corner radius, an elevation and an optional hairline border, with no semantics of its own.

A surface: a background, a corner radius, an elevation and an optional hairline
border, on a `div` by default. It is what every card, panel and popup in the
library sits on - [`Dialog`](dialog.md) is a `Paper` with a role. It renders no
ARIA of its own, because a surface is presentational; a `Paper` that becomes a
landmark owns its own name. Padding is yours, through `sx`.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{Paper, Text, Title},
    sx::sx,
};

#[component]
fn Demo() -> Element {
    rsx! {
        Paper {
            shadow: "sm",
            sx: sx().padding("lg"),
            Title { size: "md", component: "h2", "Invoice #4021" }
            Text { "Due 30 September." }
        }
    }
}
```

`radius` and `shadow` are steps on the shared size scales, so anything off the
scale - a `2px` corner, or no shadow at all - is an `sx` override rather than a
prop value:

```rust,ignore
Paper { sx: sx().padding("lg").box_shadow("none"), "Flat" }
```

`bordered` and a shadow are legal together. That is a design choice, not a
misuse, and nothing warns about it.

## Building on it

A component that renders a surface as part of its own element - rather than
nesting a `Paper` inside itself - builds its base style from `paper_sx()`
(`use libero::components::paper_sx;`) and chains its own declarations on top. `Dialog` is the worked
example: its static is `paper_sx()` plus the dialog chrome, handed back to
`Paper` as `framework_sx`, so there is one element, one CSS class and one
definition of what a surface is. Chaining works because `Paper` emits a
`data-state` token only for a step a caller explicitly names, so a base's own
`border-radius` or `box-shadow` is not outranked by an unasked-for
`[data-state]` rule.

## Accessibility

A `Paper` rendered as a `section` or `aside` becomes a landmark, and the caller
owns the `aria-label` that names it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `radius` | `Size` | `md` | Corner radius, a step on the shared radius scale. |
| `shadow` | `Size` | `sm` | Elevation, a step on the shared shadow scale. A flat surface is `sx: sx().box_shadow("none")`. |
| `bordered` | `bool` | `false` | A hairline border in the themed surface border colour. Legal together with a shadow. |
| `component` | `HtmlTag` | `div` | Which element to render as - `div`, `section`, `article`, `aside`, or `a` for a clickable card. `section` and `aside` are landmarks; the caller owns the `aria-label` that names them. |
| `variables` | `Variables` | - | Per-instance CSS custom properties, for a component built on `Paper`. |
| `framework_sx` | `Option<&'static StaticSx>` | - | Base styles for a component built on `Paper`, on the framework layer. It **replaces** `Paper`'s own base, so build it from `paper_sx()`. |
| `children` | `Element` | required | The surface's contents. |

Like every component, `Paper` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes - `href`, `target` and `rel`
included, though only the `component: "a"` form does anything with them.
That form makes the whole surface one link: it takes its accessible name from
its contents, and nothing interactive belongs inside it.
`Paper` sets no `role` and no ARIA of its own and
forwards what it is given untouched, so a caller's `role` or `aria-label`
reaches the element.

## Theme defaults

`PaperDefaults` on the theme, as `theme.paper`.

| Field | Type | Default | Description |
|---|---|---|---|
| `radius` | `Size` | `Md` | Corner radius of a `Paper` that names none. |
| `shadow` | `Size` | `Sm` | Elevation of a `Paper` that names none. |
| `background` | `&'static str` | `#fff` | The surface colour itself. A dark theme changes this value, not any component. |
| `contrast` | `ColorValue` | `black` | What reads against `background`. Change one and change the other. |
| `border_color` | `ColorValue` | `muted.3` | The `bordered` hairline. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-paper-background` | The surface colour. Anything that has to disappear against a surface reads this. |
| `--lsx-paper-contrast` | What reads against the background. Published as `--lsx-focus-contrast`, so a focus ring inside a surface contrasts against it. The background is published as `--lsx-focus-ring-halo` beside it, as `sx().background()` does for a palette shade or hex. |
| `--lsx-paper-border-color` | The `bordered` hairline colour. |
| `--lsx-paper-radius` | The themed corner radius, as a reference into the radius scale. |
| `--lsx-paper-shadow` | The themed elevation, as a reference into the shadow scale. |

## Data attributes

| `data-state` token | When |
|---|---|
| `radius-<size>` | The caller named a `radius`. Absent otherwise, so the themed default applies. |
| `shadow-<size>` | The caller named a `shadow`. Absent otherwise. |
| `bordered` | `bordered` is on. |
