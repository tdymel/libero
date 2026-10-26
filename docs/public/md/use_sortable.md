# Sortable list

Crate: `libero`
Import: `use libero::hooks::{SortableMove, SortableOptions, drag_handle_sx, use_labelled_sortable_item, use_sortable, use_sortable_item};`
Feature: `sortable` (off by default, turns on `use-drag`, `use-icon`, `use-id`, `use-media-query`, `use-timers`, `visually-hidden`; [features.md](features.md))
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/sortable/use_sortable.rs>
Index: [index.md](index.md) lists every other page
Description: Makes your own markup a list the user reorders by dragging a handle, by keyboard, or with move buttons.

`use_sortable(options: SortableOptions) -> SortableHandle` makes your own markup
a list the user reorders by dragging, as `Sortable` does. Spread its element and
pointer handlers on the list, and call `use_sortable_item(index)` in each item's
component for its grab handle, element, move buttons and `style()`. On drop
`onreorder` gets a `SortableMove`, and `apply` does it to a `Vec`.

## Usage

```rust
use dioxus::prelude::*;
use libero::{
    components::{ActionIcon, Box, Orientation, Pictogram, VisuallyHidden},
    hooks::{
        SortableMove, SortableOptions, drag_handle_sx, use_id, use_labelled_sortable_item,
        use_sortable,
    },
    sx::sx,
};
use pictogram_icons_lucide as lucide;

#[component]
pub fn Fruit(#[props(default, into)] orientation: Orientation) -> Element {
    let mut fruit = use_signal(|| vec!["Apple", "Pear", "Plum", "Cherry"]);
    let list = use_sortable(SortableOptions {
        orientation,
        onreorder: Callback::new(move |step: SortableMove| step.apply(&mut fruit.write())),
    });
    let horizontal = orientation == Orientation::Horizontal;

    rsx! {
        Box {
            role: "list",
            onmounted: list.element.mount(),
            onpointermove: move |event| list.onpointermove.call(event),
            onpointerup: move |event| list.onpointerup.call(event),
            onpointercancel: move |event| list.onpointercancel.call(event),
            sx: sx().display("flex").gap("xs").flex_direction(if horizontal { "row" } else { "column" }),
            for (index, name) in fruit().into_iter().enumerate() {
                Item { key: "{name}", index, name, horizontal }
            }
        }
        VisuallyHidden { role: "status", {list.announcement} }
    }
}

#[component]
fn Item(index: usize, name: &'static str, horizontal: bool) -> Element {
    let item = use_labelled_sortable_item(index, Some(name.to_string()));
    let help = use_id();
    let (earlier, later) = match horizontal {
        true => (
            lucide::chevron_left::outlined,
            lucide::chevron_right::outlined,
        ),
        false => (lucide::chevron_up::outlined, lucide::chevron_down::outlined),
    };
    let mut row = sx()
        .display("flex")
        .align_items("center")
        .gap("xs")
        .padding("xs")
        .border_radius("sm")
        .background("muted.1")
        .position("relative");
    if (item.dragging)() {
        row = row.z_index("1").box_shadow("0 4px 12px rgb(0 0 0 / 25%)");
    } else if (item.sorting)() {
        row = row.transition("transform 150ms ease");
    }

    rsx! {
        Box {
            role: "listitem",
            onmounted: item.element.mount(),
            style: item.style(),
            sx: row,
            Box {
                component: "button",
                r#type: "button",
                aria_label: "Reorder {name}",
                aria_describedby: "{help}",
                onmounted: item.handle.mount(),
                onpointerdown: move |event| item.onpointerdown.call(event),
                onkeydown: move |event| item.onkeydown.call(event),
                onblur: move |event| item.onblur.call(event),
                sx: drag_handle_sx().padding("0").border("0").background("transparent")
                    .color("inherit").cursor("grab"),
                Pictogram { icon: lucide::grip_vertical::outlined, width: "1em", height: "1em" }
            }
            Box { id: "{help}", hidden: true, {item.instructions} }
            Box { component: "span", "{name}" }
            ActionIcon {
                aria_label: "Move {name} earlier",
                size: "sm",
                icon: earlier,
                disabled: (item.first)(),
                onclick: move |event| item.onearlier.call(event),
            }
            ActionIcon {
                aria_label: "Move {name} later",
                size: "sm",
                icon: later,
                disabled: (item.last)(),
                onclick: move |event| item.onlater.call(event),
            }
        }
    }
}
```

## API

```rust,ignore
pub fn use_sortable(options: SortableOptions) -> SortableHandle
pub fn use_sortable_item(index: usize) -> SortableItemHandle
pub fn use_labelled_sortable_item(index: usize, label: Option<String>) -> SortableItemHandle
```

`SortableOptions`:

| Field | Type | Description |
|---|---|---|
| `orientation` | `Orientation` | `Vertical` stacks the items, `Horizontal` lines them up in a row. |
| `onreorder` | `Callback<SortableMove>` | Called once on drop, only when the item changed place. Apply it to your data with `SortableMove::apply`: the list shows its old order until you do. |

`SortableHandle` is `Copy`:

| Field | Type | Description |
|---|---|---|
| `element` | `ElementHandle` | The list: takes the pointer capture while an item drags. |
| `sorting` | `Memo<bool>` | An item is being dragged, by pointer or keyboard. |
| `announcement` | `Signal<String>` | The last lift, move, drop or cancel, in words. Show it in a `role="status"` region. |
| `onpointermove`, `onpointerup`, `onpointercancel` | `Callback<Event<PointerData>>` | Spread onto the element holding the items. |

`SortableItemHandle` is `Copy`. `use_sortable_item(index)` names the item in the
announcements by position ("Item 2"); `use_labelled_sortable_item` names it
`label`. The indices run exactly 0 to n - 1, and the item is keyed by its data,
not its index.

| Field | Type | Description |
|---|---|---|
| `element` | `ElementHandle` | The item: measured when a drag starts, and moved by `style()`. |
| `handle` | `ElementHandle` | The grab handle. Give it `drag_handle_sx()`, or a touch scrolls instead. |
| `onpointerdown`, `onkeydown`, `onblur` | `Callback<Event<..>>` | The grab handle's: Space or Enter lifts and drops, the arrows move, Escape cancels, and focus leaving it cancels a keyboard move. |
| `instructions` | `&'static str` | The localized key instructions for the grab handle. Render them in a hidden element with an id of yours and point the handle's `aria-describedby` at it. |
| `touch_instructions` | `&'static str` | For a touch screen that shows the move buttons: a tap lifts nothing, so it points to them. Use it instead of `instructions` there. |
| `earlier`, `later` | `ElementHandle` | The buttons moving the item one slot up or down (back or forward in a row), without a drag. |
| `onearlier`, `onlater` | `Callback<Event<MouseData>>` | Their click handlers. |
| `first`, `last` | `Memo<bool>` | The item has no slot before or after it. |
| `dragging` | `Memo<bool>` | This item is the one being dragged. |
| `sorting` | `Memo<bool>` | Some item of the list is being dragged. |

`offset()` is how far the item sits from its slot, in px along the list's axis.
`style()` is the item's inline `style`: a `transform` moving it by `offset()`
while it is off its slot (empty while it sits in its slot and has never moved,
`transform: none` once it has, so an idle item gets no stacking context), and
just after a drop the slide into its slot.

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

- A drag starts after 4px of pointer movement (8px on touch), so a click on the
  handle stays a click.
- Focus leaving a lifted item's handle cancels the move. After a drop the handle
  keeps focus.
- A lifted item scrolls into view at each key. Holding Space or Enter lifts or
  drops once.
- `announcement` says each lift, move, drop and cancel with the item and its
  place, in the active `Localization`. A key that would move an item past either
  end says it stays.
- `instructions` and `touch_instructions` are the localized key help for the
  grab handle's description.

### You must

- Make the grab handle a focusable button of at least 24px (WCAG 2.5.8) with
  `drag_handle_sx()`, named with the item, as "Reorder Apple", and described by
  `instructions`.
- Render `announcement` in a `role="status"` region.
- Offer a way to reorder without dragging (WCAG 2.5.7): the move buttons,
  disabled by `first` and `last` at the list's ends, or a menu.
- Name each item with `use_labelled_sortable_item`, or the announcements say
  "Item 2" while your handle reads "Reorder Apple".
- Number the items exactly 0 to n - 1 and key each by its data, not its index,
  so a reorder moves the node and the handle keeps its focus.

### Example

A fruit list whose items call `use_labelled_sortable_item(index, Some(name))`.
Tab reaches "Reorder Apple"; Space lifts it, Down moves it, and the status
region says where Apple is now; Space drops it, and focus stays on the handle.

### Limits

- A pointer drag does not scroll the page or a container at its edge. Keep a
  list within the view, and use the keys or the move buttons for a long one.
