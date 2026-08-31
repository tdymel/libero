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
            Title { size: "md", "Invoice #4021" }
            Text { "Due 30 September." }
        }
    }
}
```

`radius` and `shadow` are steps on the shared size scales, so anything off the
scale - a `2px` corner, or no shadow at all - is an `sx` override rather than a
prop value:

```rust
Paper { sx: sx().padding("lg").box_shadow("none"), "Flat" }
```

`bordered` and a shadow are legal together. That is a design choice, not a
misuse, and nothing warns about it.

## As a link

The base is `display: block` with `text-decoration: none`, which is what lets a
whole card be one link. `href` and `target` are available on any `Paper`; the
card takes its accessible name from its contents, like any other link.

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
            component: "a",
            href: "https://dioxuslabs.com",
            target: "_blank",
            bordered: true,
            sx: sx().padding("lg"),
            Title { size: "md", "Dioxus" }
            Text { "The framework Libero is built on." }
        }
    }
}
```

## Building on it

A component that renders a surface as part of its own element - rather than
nesting a `Paper` inside itself - builds its base style from the crate-internal
`paper_sx()` and chains its own declarations on top. `Dialog` is the worked
example: its static is `paper_sx()` plus the dialog chrome, handed back to
`Paper` as `framework_sx`, so there is one element, one CSS class and one
definition of what a surface is. Chaining works because `Paper` emits a
`data-state` token only for a step a caller explicitly names, so a base's own
`border-radius` or `box-shadow` is not outranked by an unasked-for
`[data-state]` rule.

## Accessibility

`Paper` renders no `role` and no ARIA, ever. A surface is presentational, and
the contents are what a reader interacts with. A `Paper` rendered as a
`section` or `aside` becomes a landmark, and the caller owns the `aria-label`
that names it - the same rule the rest of the library follows for elements it
does not name itself. Nothing in a `Paper` is focusable unless the caller makes
it so; as an `<a href>` it is focusable and carries the library's shared
`:focus-visible` ring.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `radius` | `Size` | `md` | Corner radius, a step on the shared radius scale. |
| `shadow` | `Size` | `sm` | Elevation, a step on the shared shadow scale. A flat surface is `sx: sx().box_shadow("none")`. |
| `bordered` | `bool` | `false` | A hairline border in the themed surface border colour. Legal together with a shadow. |
| `component` | `HtmlTag` | `div` | Which element to render as - `div`, `section`, `article`, `aside`, or `a` for a clickable card. |
| `variables` | `Variables` | - | Per-instance CSS custom properties, for a component built on `Paper`. |
| `children` | `Element` | required | The surface's contents. |

Like every component, `Paper` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes - `href` and `target`
included, for the link form.

## Theme defaults

`PaperDefaults` on the theme, as `theme.paper`.

| Field | Type | Default | Description |
|---|---|---|---|
| `radius` | `Size` | `Md` | Corner radius of a `Paper` that names none. |
| `shadow` | `Size` | `Sm` | Elevation of a `Paper` that names none. |
| `background` | `&'static str` | `#fff` | The surface colour itself. A dark theme changes this value, not any component. |
| `border_color` | `ColorValue` | `grey.3` | The `bordered` hairline. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-paper-background` | The surface colour. Anything that has to disappear against a surface reads this. |
| `--lsx-paper-border-color` | The `bordered` hairline colour. |
| `--lsx-paper-radius` | The themed corner radius, as a reference into the radius scale. |
| `--lsx-paper-shadow` | The themed elevation, as a reference into the shadow scale. |

## Data attributes

| `data-state` token | When |
|---|---|
| `radius-<size>` | The caller named a `radius`. Absent otherwise, so the themed default applies. |
| `shadow-<size>` | The caller named a `shadow`. Absent otherwise. |
| `bordered` | `bordered` is on. |
