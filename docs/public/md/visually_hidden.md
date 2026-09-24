# VisuallyHidden

Crate: `libero`
Import: `use libero::components::VisuallyHidden;`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/accessibility/visually_hidden.rs>
Index: [index.md](index.md) lists every other page
Description: A `span` read by screen readers but hidden from sighted layout, for extra context on something vague on its own.

Content for screen readers only, such as extra context for a link that is
vague on its own. With `focusable` on it is a skip link.

## Usage

Sighted users see "Read more", and a screen reader reads "Read more about
focus management".

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

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `focusable` | `bool` | `false` | Shows the content while focus is inside it, for a skip link. |
| `children` | `Element` | required | The screen-reader-only content. |

Like every component, `VisuallyHidden` also takes the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Accessibility

### Libero handles

- With `focusable`, the content shows while focus is inside it.

### You must

- Place the text where it should be read, inside the link and not next to it.
- Use it for text a screen reader user is missing, never to hide something
  sighted users need. To replace a control's whole name, use `aria_label`
  instead.
- Keep the children to text unless `focusable` is set, or keyboard focus lands
  somewhere invisible.
- A skip link shows at its place in the flow, so put it first on the page.
  Further down, put it in a positioned parent and set `position: absolute`
  through `sx`, or Tab never scrolls it into view.

```rust,ignore
VisuallyHidden {
    focusable: true,
    Anchor { to: "#main", "Skip to content" }
}
```

## Theme defaults

None. The hiding rules are fixed.

## CSS variables

None.

## Data attributes

None.
