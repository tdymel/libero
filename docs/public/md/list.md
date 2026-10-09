# List

Crate: `libero`
Import: `use libero::components::{List, ListItem};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/list>
Index: [index.md](index.md) lists every other page
Description: A `<ul>` of `<li>` items without the browser's list styling, with themed gaps and nested indent.

A `<ul>` of `<li>` items without the browser's list styling. A nested list
indents from its own content. Set `size` on the outer list only, since the
parent sets a nested list's indent. A nested ordered list keeps a fixed 2em,
room for its numbers. On an inner `List`, `size` changes only the gap.
`ordered: true` renders an `<ol>` with visible numbers.

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

With icons:

```rust
use dioxus::prelude::*;
use libero::components::{Icon, List, ListItem};

#[component]
fn Checklist() -> Element {
    rsx! {
        List { icon: rsx! { Icon { variant: "standard", color: "primary", size: "sm", "✓" } },
            ListItem { "Tests pass" }
            ListItem { "Docs updated" }
            ListItem { icon: rsx! { Icon { variant: "standard", color: "error", size: "sm", "✗" } },
                "Changelog missing"
            }
        }
    }
}
```

## Icons

An `icon` on the list marks every item. A `ListItem`'s own `icon` replaces it
for that item. A nested list does not take its parent's icon. In an ordered
list an icon replaces its item's number and the gutter beside it.

## Ordered lists

```rust,ignore
List { ordered: true,
    ListItem { "Install" }
    ListItem { "Configure" }
}
```

## Props

### List

| Prop | Type | Default | Description |
|---|---|---|---|
| `size` | `Size` | `md` | Item gap and nested-list indent, together. A nested ordered list keeps the fixed `2em` its numbers need. |
| `ordered` | `bool` | `false` | An `ol` with visible numbers, for items whose order matters. |
| `icon` | `Option<Element>` | `None` | Shown at the start of every item, beside its first line. Hidden from screen readers. |
| `children` | `Element` | required | The list's items. |

### ListItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `icon` | `Option<Element>` | `None` | This item's own icon, in place of the list's. |
| `parts` | `Parts<ListItemPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | The item's content. |

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `ListItemPart::Icon` | `icon` | The icon wrapper, with an icon only. |
| `ListItemPart::Body` | `body` | The content beside the icon, with an icon only. |

## Accessibility

### Libero handles

- Icons are hidden from screen readers.

### You must

- Keep a `List`'s children to `ListItem`s. A stray element between them breaks
  the list and its item count for a screen reader.
- When an icon carries meaning, such as done or missing, say it in the item's
  text too.

### Example

A checklist of `ListItem`s with check icons: a screen reader says the item
count and reads each item's text. The item says "done" in words, since the
icon is not read.

## Theme defaults

`ListDefaults` on the theme. The gap and indent scales are separate, both in
pixels.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Default `size` when the prop is omitted. |
| `gaps` | `Sizes<u8>` | Gap between items, per size, in px. |
| `indents` | `Sizes<u8>` | Left padding of a nested list, per size, in px. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-list-gap-<size>` | Gap between items for that size step. |
| `--lsx-list-indent-<size>` | Nested-list left padding for that size step. |

## Data attributes

State tokens on the `<ul>`'s (or `<ol>`'s) `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `size` in effect, which selects the gap and indent variables. |
| `ordered` | `ordered` is set. Decimal markers in a `2em` gutter. |

On a `ListItem`, `with-icon` is set while it shows an icon. The icon then sits
in a `data-slot="icon"` span and the content in a `data-slot="body"` div.
