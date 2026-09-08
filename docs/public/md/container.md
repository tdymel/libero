# Container

Crate: `libero`
Import: `use libero::components::Container;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/container.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Centers content and caps its width at a breakpoint.

Centers content and caps its width at a breakpoint - wraps your main content,
not the whole page shell. `size` names a breakpoint (`xs` is 36rem, `xxl`
101rem), so the cap only bites once the surrounding area is wider than it;
`gutters` is the horizontal padding, from the spacing scale.

## Usage

A container is invisible without something to see its edges by, so this snippet
paints a background and vertical padding through `sx`.

```rust
use dioxus::prelude::*;
use libero::{components::Container, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Container {
            size: "lg",
            gutters: "md",
            component: "div",
            sx: sx().background("muted.1").padding_top("16px").padding_bottom("16px"),
            "Centered, width-capped content."
        }
    }
}
```

## Accessibility

Use `component: "main"` or `"section"` when the wrapped region is a landmark; a
`<section>` wants an accessible name (`aria-label` or `aria-labelledby`) to be
listed as one.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `component` | `HtmlTag` | `div` | Which element to render as. |
| `size` | `ThemeAwareValue` | `lg` | Max width, as a breakpoint (`xs` is 36rem, `xxl` 101rem) - the cap only bites once the surrounding area is wider than it. |
| `gutters` | `ThemeAwareValue` | `md` | Horizontal padding, from the spacing scale. |
| `children` | `Element` | required | The container's content. |

Both `size` and `gutters` also take a literal CSS length, since they are
`ThemeAwareValue`s rather than plain `Size`s.

Like every component, `Container` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`ContainerDefaults` on the theme - the breakpoint and spacing step a
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

Each has an `-override` twin (`--lsx-container-size-override`,
`--lsx-container-gutters-override`) that the `size` and `gutters` props write
into the element's `style` attribute, so a per-instance value never mints a new
class.

## Data attributes

`Container` sets no state tokens of its own; a `states` prop is passed through
unchanged.
