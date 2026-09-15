# Visually Hidden

Crate: `libero`
Import: `use libero::components::VisuallyHidden;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/a11y/visually_hidden.rs>
Index: [index.md](index.md) - every other component's markdown page
Description: A `span` whose content is read by screen readers but removed from sighted layout - extra context for something ambiguous on its own.

Content available to screen readers but removed from sighted layout - e.g. extra
context for a link that's ambiguous out of context. It renders a `<span>`
clipped to a 1px box off-flow, so the text stays in the accessibility tree while
occupying no space.

## Usage

The preview reads "Read more"; a screen reader reads "Read more about focus
management".

```rust
use dioxus::prelude::*;
use libero::components::{Anchor, Text, VisuallyHidden};

#[component]
fn Demo() -> Element {
    rsx! {
        Text {
            Anchor {
                to: "https://example.com",
                "Read more"
                VisuallyHidden { " about focus management" }
            }
        }
    }
}
```

## Accessibility

The content stays in reading order, so place it where it should be *read*:
inside the link, not next to it.

Use it for text that is genuinely missing for a non-sighted reader - a link's
target, a table column header, a live-region announcement. Do not use it to hide
something from sighted users that they need, and prefer `aria_label` on the
control itself when the whole accessible name is being replaced rather than
extended.

A focusable element inside a visually hidden span is a trap for a keyboard user:
focus moves somewhere invisible. Keep the children to text, or set `focusable`.

## Skip link

`focusable` shows the content, on a paper background, while focus is inside it
(2.4.7). It stays `fixed` at its place in the flow, so put a skip link first on
the page. Further down, host it in a positioned parent and set
`position: absolute` through `sx`, or Tab never scrolls it into view.

```rust,ignore
VisuallyHidden {
    focusable: true,
    Anchor { to: "#main", "Skip to content" }
}
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `focusable` | `bool` | `false` | Shows the content while focus is inside it, for a skip link. |
| `children` | `Element` | required | The screen-reader-only content. |

Like every component, `VisuallyHidden` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Theme defaults

None - the clipping rules are fixed, not themed.

## CSS variables

None.

## Data attributes

None.
