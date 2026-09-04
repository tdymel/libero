# List

Crate: `libero`
Import: `use libero::components::{List, ListItem};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/list>
Index: [index.md](index.md) - every other component's markdown page
Description: An unstyled `<ul>`/`<li>` pair with themed gaps and nested indent.

Renders a `<ul>`/`<li>` pair with the browser's default list styling removed -
nested lists indent relative to their own content. `size` (`xs`-`xxl`, default
`md`) controls item gap and nested-list indent together. Only the outer list
carries `size`: a nested list's indent comes from the parent's own `& ul` rule,
so setting `size` again on the inner `List` changes its items' gap, not its
indent.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{List, ListItem};

#[component]
fn Demo() -> Element {
    rsx! {
        List { size: "md",
            ListItem { "First item" }
            ListItem { "Second item" }
            ListItem {
                "Third item, with a nested list"
                List {
                    ListItem { "Nested one" }
                    ListItem { "Nested two" }
                }
            }
        }
    }
}
```

## Accessibility

Keep the children of a `List` to `ListItem`s - a stray element between them
breaks the list and its item count for a screen reader.

## Props

### List

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Item gap and nested-list indent, together. |
| `children` | `Element` | required | The list's items. |

### ListItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `children` | `Element` | required | The item's content. |

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`ListDefaults` on the theme; the gap and indent scales are separate, both in
pixels.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `gap` | `Sizes<u8>` | Gap between items, per size, in px. |
| `indent` | `Sizes<u8>` | Left padding of a nested list, per size, in px. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-list-gap-<size>` | Gap between items for that size step. |
| `--lsx-list-indent-<size>` | Nested-list left padding for that size step. |

## Data attributes

State tokens on the `<ul>`'s `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect - this is what selects the gap and indent variables. |

`ListItem` sets no state tokens of its own.
