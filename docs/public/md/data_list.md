# DataList

Crate: `libero`
Import: `use libero::components::{DataList, DataListItem};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/data_list>
Index: [index.md](index.md) - every other component's markdown page
Description: A `<dl>` of term/description pairs, where one term can carry several descriptions.

Renders a `<dl>` of term/description pairs. Unlike [list.md](list.md), a term
can have more than one description - `DataListItem`'s `children` is a
`Vec<Element>`, not a single `Element`, so writing more than one child gives
each one its own `<dd>` with no extra ceremony over a single description -
including from a `for` loop, which flattens the same way.

The split needs the `dioxus-fork` build: against upstream dioxus main,
`children` is a single `Element` and the descriptions collapse into one `<dd>`.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Chip, DataList, DataListItem};

#[component]
fn Demo() -> Element {
    let phones = vec!["555-1234", "555-5678"];

    rsx! {
        DataList { orientation: "vertical", gap: "md",
            DataListItem {
                label: rsx! { "Status" },
                Chip { variant: "filled", color: "success", size: "xs", "Active" }
            }
            DataListItem { label: rsx! { "Owner" }, "Jamie Chen" }
            DataListItem {
                label: rsx! { "Phone" },
                for phone in &phones {
                    "{phone}"
                }
            }
        }
    }
}
```

`orientation: "horizontal"` switches the `<dl>` to a two-column grid, terms in
the first column and every description in the second - pinned explicitly, so a
term's second `<dd>` cannot flow back into the term column.

## Accessibility

Keep the `DataListItem`s directly inside the `DataList` - an intervening wrapper
element breaks the term/description pairing.

## Props

### DataList

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `vertical` | `horizontal` puts each description beside its term; `vertical` stacks it below. |
| `gap` | `Size` | `md` | Row gap. Off-scale values go through `sx`. |
| `children` | `Element` | required | `DataListItem`s, or any `dt`/`dd` content. |

### DataListItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Element` | required | The term (`<dt>`). `sx`/`class`/`states` decorate this element only - nothing wraps a term together with its descriptions. |
| `children` | `Vec<Element>` | required | Descriptions for `label`. Under the `dioxus-fork` build this is `Vec<Element>`, so each child gets its own `<dd>`; against upstream main they collapse into one. |

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`DataListDefaults` on the theme; the gap scale is in pixels.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | Size step `gap` falls back to when the prop is omitted. |
| `gaps` | `Sizes<u8>` | Row gap per size step, in px. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-data-list-gap-<size>` | Row gap for that size step. |

## Data attributes

State tokens on the `<dl>`'s `data-state`, space separated.

| Token | Condition |
|---|---|
| `size-<size>` | The `gap` step in effect - this is what selects the gap variable. |
| `horizontal` | `orientation` is `horizontal`; this is what switches the `<dl>` to a grid. |

`DataListItem` sets no state tokens of its own; its `states` prop lands on the
`<dt>`.
