# Paper

Crate: `libero`
Import: `use libero::components::Paper;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/paper.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The library's surface, with a background, a corner radius, an elevation and an optional hairline border, and no semantics of its own.

A surface with a background, a corner radius, an elevation and an optional
hairline border. Every card, panel and popup in the library sits on one, and
[`Dialog`](dialog.md) is a `Paper` with a role. It has no ARIA of its own.
Padding comes from your `sx`.

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

`radius` and `shadow` are steps on the shared scales. Anything off the scale,
like a `2px` corner or no shadow, goes through `sx`.

```rust,ignore
Paper { sx: sx().padding("lg").box_shadow("none"), "Flat" }
```

## Building on it

A component that is a surface itself, rather than one holding a `Paper`, starts
its base style from `paper_sx()` (`use libero::components::paper_sx;`), adds its
own rules and passes the result as `framework_sx`. `Dialog` works this way.

## Accessibility

A `Paper` rendered as a `section` or `aside` is a landmark and needs your
`aria-label`. As an `a` the whole surface is one link, named by its contents, so
nothing interactive belongs inside it.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `radius` | `Size` | `md` | Corner radius, a step on the shared radius scale. |
| `shadow` | `Size` | `sm` | Elevation, a step on the shared shadow scale. For a flat surface use `sx().box_shadow("none")`. |
| `bordered` | `bool` | `false` | A hairline border in the theme's surface border colour. Works together with a shadow. |
| `component` | `HtmlTag` | `div` | The element to render, such as `section`, `article`, `aside`, or `a` for a clickable card. A `section` or `aside` is a landmark and needs your `aria-label`. |
| `variables` | `Variables` | - | Custom properties set on the element's `style`, for a component built on `Paper`. |
| `framework_sx` | `&'static StaticSx` | - | Base styles for a component built on `Paper`. They replace `Paper`'s own, so start from `paper_sx()`. |
| `children` | `Element` | required | The surface's contents. |

Like every component, `Paper` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes, `href`, `target` and `rel`
included for the `component: "a"` form. A caller's `role` or `aria-label`
reaches the element untouched.

## Theme defaults

`PaperDefaults` on the theme, as `theme.paper`.

| Field | Type | Default | Description |
|---|---|---|---|
| `radius` | `Size` | `Md` | Corner radius of a `Paper` that names none. |
| `shadow` | `Size` | `Sm` | Elevation of a `Paper` that names none. |
| `background` | `&'static str` | `#fff` | The surface colour. A dark theme changes this value, not any component. |
| `contrast` | `ColorValue` | `black` | The text colour on `background`. Change both together. |
| `border_color` | `ColorValue` | `muted.3` | The `bordered` hairline. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-paper-background` | The surface colour. |
| `--lsx-paper-contrast` | The text colour on the surface. A focus ring inside the surface contrasts against it. |
| `--lsx-paper-border-color` | The `bordered` hairline colour. |
| `--lsx-paper-radius` | The theme's corner radius. |
| `--lsx-paper-shadow` | The theme's elevation. |

## Data attributes

| `data-state` token | When |
|---|---|
| `radius-<size>` | The caller named a `radius`. Absent otherwise. |
| `shadow-<size>` | The caller named a `shadow`. Absent otherwise. |
| `bordered` | `bordered` is on. |
