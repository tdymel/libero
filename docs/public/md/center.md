# Center

Crate: `libero`
Import: `use libero::components::Center;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/center.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: Centers its child horizontally and vertically.

Centers its child both horizontally and vertically. `inline` switches it from
`flex` to `inline-flex`, so it shrinks to its child instead of filling the
parent's width. It has no width or height of its own, so give it one - or let
the parent do it - before either mode is visible.

## Usage

The outer `Box` is the parent whose width `Center` either fills or shrinks
inside; the inner one is the centered child.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Center}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box { sx: sx().width("260px").background("grey.2"),
            Center { sx: sx().height("120px").background("primary.1"),
                Box { sx: sx().padding("8px 16px").background("primary"), "Centered" }
            }
        }
    }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `inline` | `bool` | `false` | `inline-flex` instead of `flex`, so it doesn't stretch to the parent's width. |
| `children` | `Element` | required | The centered content. |

`inline` is an `Option<bool>`: leaving it unset defers to the theme's own
default, while `Some(false)` pins `flex`.

Like every component, `Center` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Theme defaults

`CenterDefaults` on the theme - one field, the display mode every `Center`
starts from.

| Field | Type | Description |
|---|---|---|
| `inline` | `bool` | `true` makes the default `inline-flex`, `false` makes it `flex`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-center-display` | `display` for every `Center`: `flex` or `inline-flex`. |

The `inline` prop writes `--lsx-center-display-override` into the element's
`style` attribute, so a per-instance mode never mints a new class.

## Data attributes

`Center` sets no state tokens of its own; a `states` prop is passed through
unchanged.
