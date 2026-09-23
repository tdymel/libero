# Sortable

Crate: `libero`
Import: `use libero::{components::{Sortable, SortableItem}, hooks::SortableMove};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/sortable>
Index: [index.md](index.md) lists every other page
Description: A list the user reorders by dragging each item's handle. `use_sortable` and `use_sortable_item` do the same for your own markup.

A list the user reorders by dragging each item's handle. The other items step
aside while one drags. On drop `onreorder` gets a `SortableMove` with the old
and new index, and `apply` does it to a `Vec`.

For your own markup, `use_sortable` gives the list's handlers and
`use_sortable_item(index)` each item's handle, element and `style()`. The
component is those two hooks plus a grip button.

## Usage

```rust
use dioxus::prelude::*;
use libero::{components::{Sortable, SortableItem}, hooks::SortableMove};

#[component]
fn Demo() -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);

    rsx! {
        Sortable {
            onreorder: move |step: SortableMove| step.apply(&mut fruit.write()),
            for (index, name) in fruit().into_iter().enumerate() {
                SortableItem { key: "{name}", index, "{name}" }
            }
        }
    }
}
```

Key each item by its data, not its index. A reorder then moves the item's node,
and the handle keeps the focus.

## The hooks

`use_sortable(SortableOptions)` returns a `SortableHandle`: the list's
`element` and three pointer handlers for the element holding the items. Call
`use_sortable_item(index)` in each item's own component below it. Its
`SortableItemHandle` has the item's `element`, the grab `handle` with its
`onpointerdown`, and `style()`, the `transform` that moves the item while a
drag is on. `dragging` and `sorting` say whether this item, or any, is
dragged.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Orientation},
    hooks::{SortableMove, SortableOptions, drag_handle_sx, use_sortable, use_sortable_item},
};

#[component]
fn Demo() -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum"]);
    let list = use_sortable(SortableOptions {
        orientation: Orientation::Vertical,
        onreorder: Callback::new(move |step: SortableMove| step.apply(&mut fruit.write())),
    });

    rsx! {
        Box {
            onmounted: list.element.mount(),
            onpointermove: move |event| list.onpointermove.call(event),
            onpointerup: move |event| list.onpointerup.call(event),
            onpointercancel: move |event| list.onpointercancel.call(event),
            for (index, name) in fruit().into_iter().enumerate() {
                Fruit { key: "{name}", index, name }
            }
        }
    }
}

#[component]
fn Fruit(index: usize, name: &'static str) -> Element {
    let item = use_sortable_item(index);
    rsx! {
        Box { onmounted: item.element.mount(), style: item.style(),
            Box {
                onmounted: item.handle.mount(),
                onpointerdown: move |event| item.onpointerdown.call(event),
                sx: drag_handle_sx(),
                "Drag"
            }
            "{name}"
        }
    }
}
```

## Accessibility

### Libero handles

- Each item's handle is a button named by `SortableLabels::handle` ("Reorder"),
  at least 24px square (WCAG 2.5.8).
- A drag starts after the pointer moved 4px (8px for a touch), so a click on
  the handle stays a click.
- Only the handle takes a touch. Swiping the rest of an item scrolls the page.
- The handle keeps the focus after a drop.
- Under reduced motion the other items jump to their new place instead of
  sliding.

### You must

- Offer another way to reorder for now, such as move up and down buttons: the
  keyboard and announcements are not in yet.

## Props

### Sortable

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `vertical` | `vertical` stacks the items, `horizontal` puts them in a row. |
| `onreorder` | `EventHandler<SortableMove>` | required | Fires on drop when an item changed place. Apply the move to your data with `SortableMove::apply`; until then the list keeps its old order. |
| `children` | `Element` | required | The `SortableItem`s. |

### SortableItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `index` | `usize` | required | The item's current position, from 0. Key the item by its data, not by this. |
| `children` | `Element` | required | The item's content, after the handle. |

Like every component, both also take the shared props `sx`, `class`, `states`,
and any extra HTML attributes.

## Data attributes

State tokens on `data-state`.

| Token | On | Condition |
|---|---|---|
| `vertical` / `horizontal` | `Sortable` | The `orientation` in effect. |
| `sorting` | `Sortable`, `SortableItem` | An item is being dragged. |
| `dragging` | `SortableItem` | This item is the one dragged. |
