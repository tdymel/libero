# Box

Crate: `libero`
Import: `use libero::components::Box;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/box.rs>
Index: [index.md](index.md) lists every other page
Description: The primitive every other component is built on, rendered as any tag via `component` and styled through `sx`.

The primitive every other component is built on. `component` picks the tag, and
attributes like `href` or `src` pass through to it. `Box` has no look of its
own. Everything visible comes from `sx` and `states`, see
[styling.md](styling.md).

Reach for `Box` when an element needs `sx` or `states`, or when the tag is
decided at runtime: it renders any tag via `component`. Markup that needs
neither stays a plain Dioxus element.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::Box;
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            sx: sx().padding("16px").background("muted.1").border_radius("md"),
            "Styled entirely via sx"
        }
    }
}
```

`component` swaps the tag, and that tag's attributes come with it.

```rust
use dioxus::prelude::*;
use libero::components::Box;
use libero::sx::sx;

#[component]
fn Demo() -> Element {
    rsx! {
        Box {
            component: "a",
            href: "https://dioxuslabs.com",
            target: "_blank",
            sx: sx().padding("16px").background("muted.1").border_radius("md"),
            "Styled entirely via sx"
        }
    }
}
```

`component` takes any of the 111 HTML5 element names. 83 of them render on
default features. The other 28 (document metadata, embedded and media content,
`template`, `slot`, bidi and ruby) need the `full-polymorphism` feature and
render as a `div` without it. See
[getting_started.md](getting_started.md#feature-flags).

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variables` | `Variables` | - | Custom properties set on the element's `style`, so `sx` can use a changing value without a new class per value. |
| `component` | `HtmlTag` | `div` | The element to render, any of the 111 HTML5 element names. The 28 outside the default set need the `full-polymorphism` feature and render as a `div` without it. |
| `framework_sx` | `&'static StaticSx` | - | Base styles for a component built on `Box`. They sit on a CSS layer below `sx`, so a caller's `sx` still wins. |
| `style` | `String` | - | Raw `style` declarations, applied after `variables`. |
| `alt` | `String` | - | The `img` alt text. |
| `r#type` | `String` | - | The `button` type, such as `"submit"`. |
| `children` | `Element` | required | The element's content. |

Like every component, `Box` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes. It also takes the `img`, `a` and
`button` attributes, such as `src`, `href` and `target`.

## Accessibility

### Libero handles

- `Box` adds no roles, so the semantics are whatever tag `component` names.

### You must

- Use `component: "button"` for something clickable: a clickable `div` has no
  keyboard support.
- Set `r#type: "button"` on a `button` inside a form, or it submits the form.
- Give an `img` an `alt`, empty for a decorative one.

## Theme defaults

None. The theme reaches a `Box` only through the values its `sx` names.

## CSS variables

None of its own. `variables` sets your own custom properties on the element.

## Data attributes

Only what you pass. The `states` prop renders as `data-state`, which `Sx::when`
matches. `Box` adds no state tokens itself.
