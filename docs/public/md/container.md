# Container

Crate: `libero`
Import: `use libero::components::Container;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/container.rs>
Index: [index.md](index.md) lists every other page
Description: Centers content and caps its width at a breakpoint.

Centers content and caps its width at a breakpoint. Wrap your main content in
it, not the whole page shell. The cap shows only once the space around it is
wider than `size`.

## Usage

The `sx` background and padding only make the edges visible.

```rust
use dioxus::prelude::*;
use libero::{components::Container, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Container {
            sx: sx().background("muted.1").padding_top("16px").padding_bottom("16px"),
            "Centered, width-capped content."
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `component` | `HtmlTag` | `div` | The element to render. |
| `size` | `ThemeAwareValue` | `lg` | Max width, a breakpoint (`xs` is 36rem, `xxl` 101rem) or a CSS length. |
| `gutters` | `ThemeAwareValue` | `md` | Horizontal padding, a spacing step or a CSS length. |
| `children` | `Element` | required | The container's content. |

Like every component, `Container` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- A focused container draws its focus ring inside its edges, so a full-width
  one keeps the ring on screen.

### You must

- Set `component: "main"` or `"section"` when the region is a landmark.
- Name a `section` with `aria-label` or `aria-labelledby`: without a name it
  is no landmark.

### Example

A skip-link target: `Container { component: "main", id: "content", tabindex:
"-1", .. }`. A link to `#content` moves focus there, and screen readers list
the region as the main landmark.

## Theme defaults

`ContainerDefaults` on the theme holds the breakpoint and spacing step a
`Container` falls back to.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Breakpoint the max width defaults to. |
| `gutters` | `Size` | Spacing step the horizontal padding defaults to. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-container-size` | The default max width, resolved from the theme's breakpoint scale. |
| `--lsx-container-gutters` | The default horizontal padding, resolved from the spacing scale. |

## Data attributes

`Container` sets no state tokens of its own. A `states` prop passes through
unchanged.
