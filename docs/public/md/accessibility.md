# Accessibility

Crate: `libero`
Index: [index.md](index.md) - every other component's markdown page
Description: What libero's accessibility support covers across the whole library, and what it does not - including how on and disabled states read, and forced colors, which is covered only in part.

Every component page has its own Accessibility section: the keys that component
answers, and the props you have to set because it cannot work them out - a name
for something that has no visible label, mostly. This page holds what applies to
the library as a whole, starting with what libero does not do, so you can decide
what to do about it.

## On and disabled states

A pressed, selected or current control never differs by colour alone. It
carries a short 2px line in its own text colour: `Button`, `ActionIcon`,
`Chip`, `SegmentedControl`, the current `Pagination` page and the current
`Stepper` marker draw it centred under the content, an active `NavLink` and a
selected row in a `Select`, `MultiSelect` or `Combobox` list upright at the
start edge. A disabled control fades to half.

## Forced colors and Windows High Contrast

Libero supports forced colors only in part. The states above hold: an on state
paints the system's `Highlight` pair and a disabled control's text turns
`GrayText`. A few components carry their own `(forced-colors: active)` arm; the
rest of the library is not tested in that mode.

Windows High Contrast is the mode this affects. In it the operating system
replaces every colour an author picked and drops `box-shadow` entirely.
Anything libero draws with a shadow or a themed colour alone can therefore
disappear: a `Paper` that relies on its shadow for its edge (`bordered: false`)
has no visible boundary at all, and a state carried only by colour is no longer
distinguishable. Borders, text, focus rings drawn as outlines and the layout
itself survive, because the system repaints them.

If your users run High Contrast, prefer a border over a shadow where a boundary
matters, and write the arm yourself - `sx` takes any media query:

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

The reason we stop there: forced colors is a Windows platform mode, not a WCAG
success criterion at any conformance level, and covering it properly means an
arm on most of the library plus a test tier to keep it honest. If you need more
of it, open an issue - a real user asking is what would change this.
