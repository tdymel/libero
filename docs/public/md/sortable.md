# Sortable

Crate: `libero`
Import: `use libero::{components::{Sortable, SortableItem}, hooks::SortableMove};`
Feature: `sortable` (off by default, turns on `use-drag`, `use-icon`, `use-id`, `use-media-query`, `use-timers`, `visually-hidden`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/sortable>
Index: [index.md](index.md) lists every other page
Description: A list the user reorders by dragging a handle, by keyboard, or with move buttons. `use_sortable` and `use_sortable_item` do the same for your own markup.

A list the user reorders by dragging each item's handle, by keyboard, or with
each item's move buttons. The other items step aside while one drags. On drop
`onreorder` gets a `SortableMove` with the old and new index, and `apply` does
it to a `Vec`.

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
                SortableItem { key: "{name}", index, label: name, "{name}" }
            }
        }
    }
}
```

Key each item by its data, not its index. A reorder then moves the item's node,
and the handle keeps the focus.

## The hooks

`use_sortable(SortableOptions)` returns a `SortableHandle`: the list's
`element`, three pointer handlers for the element holding the items, and
`announcement`, the last lift, move, drop or cancel in words for a
`role="status"` region. Call `use_sortable_item(index)` in each item's own
component below it. Its `SortableItemHandle` has the item's `element`, the
grab `handle` with its `onpointerdown`, `onkeydown` and `onblur`, and
`style()`, the `transform` that moves the item while a drag is on and the
slide into its slot after a drop. `earlier`/`onearlier` and `later`/`onlater`
wire your move buttons, `first` and `last` say when to disable them.
`dragging` and `sorting` say whether this item, or any, is dragged. A hook
item is named by position ("Item 2") in the announcements, unless you call
`use_labelled_sortable_item(index, Some(name))` instead: it names the item as
your grab handle does. `instructions` is the
localized key help for the grab handle: render it in a hidden element with an
id of yours and point the handle's `aria-describedby` at that id. Where you
show the move buttons on a touch screen, use `touch_instructions` instead: a
tap lifts nothing, so it points to the buttons.

```rust
use dioxus::prelude::*;
use libero::{
    components::{Box, Orientation, VisuallyHidden},
    hooks::{SortableMove, SortableOptions, use_labelled_sortable_item, use_sortable},
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
            role: "list",
            onmounted: list.element.mount(),
            onpointermove: move |event| list.onpointermove.call(event),
            onpointerup: move |event| list.onpointerup.call(event),
            onpointercancel: move |event| list.onpointercancel.call(event),
            for (index, name) in fruit().into_iter().enumerate() {
                Fruit { key: "{name}", index, name }
            }
        }
        VisuallyHidden { role: "status", {list.announcement} }
    }
}

#[component]
fn Fruit(index: usize, name: &'static str) -> Element {
    let item = use_labelled_sortable_item(index, Some(name.to_string()));
    rsx! {
        Box {
            role: "listitem",
            onmounted: item.element.mount(),
            style: item.style(),
            div { id: "help-{name}", hidden: true, {item.instructions} }
            button {
                onmounted: item.handle.mount(),
                aria_describedby: "help-{name}",
                onpointerdown: move |event| item.onpointerdown.call(event),
                onkeydown: move |event| item.onkeydown.call(event),
                onblur: move |event| item.onblur.call(event),
                style: "touch-action: none",
                "Reorder {name}"
            }
            button {
                onmounted: item.earlier.mount(),
                aria_label: "Move {name} up",
                onclick: move |event| item.onearlier.call(event),
                disabled: (item.first)(),
                "Up"
            }
            button {
                onmounted: item.later.mount(),
                aria_label: "Move {name} down",
                onclick: move |event| item.onlater.call(event),
                disabled: (item.last)(),
                "Down"
            }
        }
    }
}
```

## Your own markup

For your own markup, `use_sortable` gives the list's handlers and
`use_sortable_item(index)` each item's handle, element, move buttons and
`style()`. The component is those two hooks plus a grip button, two move
buttons and a status region. `style()` is empty while an item sits in its slot
and has never moved, and `transform: none` once it has, so an idle item gets no
stacking context or fixed-position containing block from it.

## Props

### Sortable

| Prop | Type | Default | Description |
|---|---|---|---|
| `orientation` | `Orientation` | `vertical` | `vertical` stacks the items, `horizontal` puts them in a row. |
| `onreorder` | `EventHandler<SortableMove>` | required | Fires on drop, or on a move button, when an item changed place. Apply the move to your data with `SortableMove::apply`; until then the list keeps its old order. |
| `move_buttons` | `bool` | `true` | Each item's two buttons that move it one slot without dragging (WCAG 2.5.7). Off, only a drag and the handle's keys reorder: the pointer has no way but dragging (WCAG 2.5.7), so offer one yourself. |
| `children` | `Element` | required | The `SortableItem`s. |

### SortableItem

| Prop | Type | Default | Description |
|---|---|---|---|
| `index` | `usize` | required | The item's current position. The items run exactly 0 to n - 1: a gap stops every drag and lift, a repeat warns and some items never move, so number a filtered view or a page from 0. Key the item by its data, not by this. |
| `label` | `Option<String>` | - | Names the item in its controls and the announcements. Unset, the handle reads the item's content ("Reorder Apple"), and the move buttons `SortableLabels::item` with its current position, the announcements with the position it was lifted from. |
| `parts` | `Parts<SortableItemPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | The item's content, between the handle and the move buttons. |

Like every component, both also take the shared props `sx`, `class`, `states`,
and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `SortableItemPart::Handle` | `handle` | The drag handle. |
| `SortableItemPart::Content` | `content` | The wrapper round the item's content. |
| `SortableItemPart::MoveEarlier` | `move-earlier` | The move up or back button. |
| `SortableItemPart::MoveLater` | `move-later` | The move down or forward button. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Space` or `Enter` | On a handle, lifts its item. Lifted, drops it where it is. |
| `Up` or `Down` | Moves a lifted item one slot, in a vertical list. |
| `Left` or `Right` | Moves a lifted item one slot, in a horizontal list; mirrored right to left. |
| `Home` or `End` | Moves a lifted item to the first or last slot. |
| `Escape` | Puts a lifted or dragged item back where it was. |

### Libero handles

- Each item's handle is a button of at least 24px (WCAG 2.5.8) named with the
  item, "Reorder Apple", and described by how to move it with the keys. On a
  touch screen with the move buttons shown, the description points to them
  instead.
- Each item has Move up and Move down buttons (back and forward in a row),
  named with the item, "Move Apple up", so one pointer reorders without a drag
  (WCAG 2.5.7). At the list's ends the button that cannot move is disabled,
  and focus goes to the other one.
- A status region says each lift, move, drop and cancel with the item and its
  place. A key that would move an item past either end says it stays. The
  words come from `SortableLabels` in the active `Localization`.
- A horizontal list wider than its container scrolls inside itself and never
  widens the page (WCAG 1.4.10).
- A drag starts after 4px of pointer movement (8px on touch), so a click on
  the handle stays a click. Only the handle takes a touch: a swipe on the rest
  of an item scrolls the page.
- Focus leaving a lifted item's handle cancels the move. After a drop the
  handle keeps focus.
- A lifted item scrolls into view at each key, in a long list and in a row
  that scrolls inside itself. Holding Space or Enter lifts or drops once.
- A dropped item slides into its slot. Under reduced motion it lands at once,
  and the others jump instead of sliding.

### You must

- Give each `SortableItem` a `label`, or its controls and the announcements
  name it by position ("Item 2"). In your own markup, name each item with
  `use_labelled_sortable_item`, or the announcements say "Item 2" while your
  handle reads "Reorder Apple".
- With `move_buttons: false`, give the reader another way to reorder without
  dragging, such as a menu.

### Example

A fruit list where each `SortableItem` has `label: name`. Tab reaches "Reorder
Apple"; Space lifts it, Down moves it, and the status region says where Apple
is now; Space drops it, and focus stays on the handle.

### Limits

- A pointer drag does not scroll the page or a container at its edge. Keep a
  list within the view, and use the keys or the move buttons for a long one.

## Data attributes

State tokens on `data-state`.

| Token | On | Condition |
|---|---|---|
| `vertical` / `horizontal` | `Sortable` | The `orientation` in effect. |
| `sorting` | `Sortable`, `SortableItem` | An item is being dragged. |
| `dragging` | `SortableItem` | This item is the one dragged. |
