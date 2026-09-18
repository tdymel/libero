# DataList

Crate: `libero`
Import: `use libero::components::{DataList, DataListItem};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/data_list>
Index: [index.md](index.md) lists every other page
Description: A `<dl>` of term/description pairs, where one term can carry several descriptions.

A `<dl>` of terms and descriptions. One term can carry several descriptions,
which share one `<dd>`.

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
                    div { "{phone}" }
                }
            }
        }
    }
}
```

`orientation: "horizontal"` makes the `<dl>` a two-column grid, with terms in
the first column and every description in the second.

## Accessibility

Put each `DataListItem` directly inside the `DataList`. A wrapper element
between them breaks the pairing of term and description.

## Props

### DataList

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `vertical` | `horizontal` puts each description beside its term, `vertical` below it. |
| `gap` | `Size` | `md` | Row gap. Other values go through `sx`. |
| `children` | `Element` | required | `DataListItem`s, or any `dt` and `dd` content. |

### DataListItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `Element` | required | The term, a `<dt>`. `sx`, `class` and `states` style the term only. |
| `children` | `Element` | required | The term's descriptions, in one `<dd>`. |

Like every component, both also take the shared props `sx`, `class`, `style`,
`states`, and any extra HTML attributes.

## Theme defaults

`DataListDefaults` on the theme. The gap scale is in pixels.

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
| `size-<size>` | The `gap` step in effect, which selects the gap variable. |
| `horizontal` | `orientation` is `horizontal`, which makes the `<dl>` a grid. |

`DataListItem` sets no state tokens of its own. Its `states` prop lands on the
`<dt>`.
