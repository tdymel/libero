# Box

Crate: `libero`
Import: `use libero::components::Box;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/box.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: The polymorphic primitive every other component is built on - renders as any tag via `component`, styled entirely through `sx`.

The polymorphic primitive every other component is built on - renders as any tag
via `component`, plus `sx`/`states` styling and escape-hatch attributes like
`href`/`src`, which follow whichever tag you picked.

## Usage

`Box` has no look of its own: everything visible comes from `sx`. See
[styling.md](styling.md) for the builder.

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

`component` swaps the rendered tag, and the attributes that only exist on that
tag follow it:

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

`component` takes any of the 111 HTML5 element names, and 83 of them - every
sectioning, text-level, list, table and form element - render on default
features. The other 28 (document metadata, embedded and media content,
`template`/`slot`, and the bidi and ruby set) need the `full-polymorphism`
feature; without it they render as a `div`, silently in a release build. See
[getting_started.md](getting_started.md#feature-flags) for the full list.

## Accessibility

`Box` adds no roles, so the semantics are whatever tag `component` names.
Picking a `div` for something clickable loses the keyboard behaviour a `button`
would have given for free.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variables` | `Variables` | - | Per-instance CSS custom properties on the `style` attribute, so `sx` can reference a varying value without a class per value. |
| `component` | `HtmlTag` | `div` | Which element to render as. |
| `framework_sx` | `&'static StaticSx` | - | Base styles of a component built on `Box`, on their own CSS layer - below `sx`, so a caller's override still wins. |
| `style` | `String` | - | Raw `style` declarations, merged after `variables` - not overwritten by it. |
| `alt` | `String` | - | The `img` attribute, a field rather than a passed-through attribute because `a` and `button` share the name. |
| `r#type` | `String` | - | The `button` attribute, a field for the same reason as `alt`. |
| `children` | `Element` | required | The element's content. |

Like every component, `Box` also takes the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes - here extended with the `img`, `a` and
`button` attribute sets, so `src`, `href`, `target` and friends pass through.

## Theme defaults

None. `Box` is unstyled by design; the theme only reaches it through the values
an `sx` resolves.

## CSS variables

None of its own. `variables` puts your own custom properties on the instance,
and `framework_sx` is how a component built on `Box` declares its base rules.

## Data attributes

Only what you pass: the `states` prop renders as `data-state`, matched by
`Sx::when`. `Box` adds no state tokens itself.
