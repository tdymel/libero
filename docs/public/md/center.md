# Center

Crate: `libero`
Import: `use libero::components::Center;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/layout/center.rs>
Index: [index.md](index.md) lists every other page
Description: Centers its child horizontally and vertically.

Centers its child horizontally and vertically. It fills the parent's width and
has no height of its own. With `inline` it shrinks to its child.

## Usage

The outer `Box` is the parent, the inner one the centered child.

```rust
use dioxus::prelude::*;
use libero::{components::{Box, Center}, sx::sx};

#[component]
fn Demo() -> Element {
    rsx! {
        Box { sx: sx().width("260px").background("muted.2"),
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
| `inline` | `bool` | `false` | Shrinks to the child instead of filling the parent's width. |
| `children` | `Element` | required | The centered content. |

Leave `inline` unset to use the theme's default.

Like every component, `Center` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- `Center` adds no roles and moves nothing, so tab and reading order match the
  code.
- A child larger than the box spills toward the end, where scrolling reaches it,
  never past the start.

### You must

- `Center` always renders a `div`: for a list or a nav, put a `Box` with
  `component: "ul"` or `"nav"` inside.

### Example

An empty state in `Center`, with an icon, a line of text and a "New project"
button: a screen reader and Tab meet them in that order, the order of the
code.

## Theme defaults

`CenterDefaults` on the theme holds the display mode every `Center` starts
from.

| Field | Type | Description |
|---|---|---|
| `inline` | `bool` | `true` makes the default `inline-flex`, `false` makes it `flex`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-center-display` | `display` for every `Center`: `flex` or `inline-flex`. |

## Data attributes

`Center` sets no state tokens of its own. A `states` prop passes through
unchanged.
