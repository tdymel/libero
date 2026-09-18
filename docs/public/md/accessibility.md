# Accessibility

Crate: `libero`
Index: [index.md](index.md) lists every other page
Description: What libero's accessibility support covers across the library and what it does not, from on and disabled states to forced colors, which is covered only in part.

Each component page has its own Accessibility section with the keys the
component answers and the props you must set, mostly a name for something
without a visible label. This page covers the library as a whole, including
what libero does not do.

## On and disabled states

A pressed, selected or current control never differs by colour alone. It also
carries a line in its own text colour. `Button`, `ActionIcon`, `Chip`,
`SegmentedControl`, the current `Pagination` page and the current `Stepper`
marker draw a thin ring just inside their edge. An active `NavLink` and a
selected row in a `Select`, `MultiSelect` or `Combobox` list draw a short 2px
bar at their start edge. A disabled control fades to half.

## Forced colors and Windows High Contrast

Libero supports forced colors only in part. The states above still show. An on
state takes the system's `Highlight` colours, and a disabled control's text
turns `GrayText`. A few components handle `(forced-colors: active)`
themselves. The rest of the library is untested in that mode.

In Windows High Contrast the system replaces every colour an author picked and
drops `box-shadow`. So anything libero draws with a shadow or a colour alone
can disappear. A `Paper` with `bordered: false` loses its edge, and a state
shown only by colour can no longer be told apart. Borders, text, outline focus
rings and the layout survive.

If your users run High Contrast, prefer a border over a shadow where an edge
matters, and add the media query yourself:

```rust
use dioxus::prelude::*;
use libero::components::Paper;
use libero::sx::sx;

#[component]
fn Card() -> Element {
    rsx! {
        Paper {
            // A border with no colour is currentColor, which the OS repaints.
            sx: sx().media("(forced-colors: active)", sx().border("1px solid")),
            "Still has an edge in High Contrast"
        }
    }
}
```

Forced colors is a Windows mode, not a WCAG success criterion, and covering it
properly means work on most of the library plus tests to keep it. If you need
more of it, open an issue. A real user asking is what would change this.
