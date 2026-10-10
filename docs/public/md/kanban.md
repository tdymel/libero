# Kanban

Crate: `libero`
Import: `use libero::components::{Kanban, KanbanCard, KanbanColumn, KanbanColumnMove, KanbanMove};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/kanban>
Index: [index.md](index.md) lists every other page
Description: A board of columns whose cards move by drag in a column or to another, by keyboard or move buttons in a column, and to another column by a Move to menu.

A board of columns. A card moves by dragging its handle, in its column or to
another; by keyboard in its column or to the next one; with its move buttons in
its column; and to any column with its Move to menu. `onmove` gets a `KanbanMove` with the old and new
column and index, and `apply` does it to a `Vec<Vec<T>>`. With `oncolumnmove` the
columns reorder too, by a handle and two buttons in each header.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Kanban, KanbanCard, KanbanColumn, KanbanMove};

#[component]
fn Demo() -> Element {
    let columns = ["To do", "Doing", "Done"];
    let mut cards = use_signal(|| vec![
        vec!["Write the brief", "Draw the board"],
        vec!["Review the API"],
        vec![],
    ]);

    rsx! {
        Kanban {
            onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
            for (column, label) in columns.into_iter().enumerate() {
                KanbanColumn { key: "{label}", index: column, label,
                    for (index, card) in cards()[column].clone().into_iter().enumerate() {
                        KanbanCard { key: "{card}", index, label: card, "{card}" }
                    }
                }
            }
        }
    }
}
```

Key each column and card by its data, not its index. A drag to another column
lands the card where it was let go; the Move to menu lands it at that column's
end.

### Column reorder

Set `oncolumnmove` and each column header gets a reorder handle and, with
`move_buttons`, two move buttons. Keep the column order as data next to the
cards and apply each `KanbanColumnMove` to both:

```rust,ignore
let mut names = use_signal(|| vec!["To do", "Doing", "Done"]);
// in the Kanban:
oncolumnmove: move |step: KanbanColumnMove| {
    step.apply(&mut names.write());
    step.apply(&mut cards.write());
},
```

## Cards

The cards are your content: the board draws no card look. The docs demo builds
Jira-style issue cards from `Text`, `Badge`, `Icon` and `Avatar`, with a count
in each column's `header`.

## Props

### Kanban

| Prop | Type | Default | Description |
|---|---|---|---|
| `onmove` | `EventHandler<KanbanMove>` | required | Fires when a card changed place: on a drop, a move button or a Move to entry. Apply it to your data with `KanbanMove::apply`; until then the board keeps its old order. |
| `oncolumnmove` | `Option<EventHandler<KanbanColumnMove>>` | `None` | Fires when a column changed place, by its header's reorder handle or move buttons. Set, each column header shows them. Apply it with `KanbanColumnMove::apply` to the column names and to the cards, so the `index` of each column follows. |
| `move_buttons` | `bool` | `true` | Each card's two buttons that move it one slot in its column without dragging (WCAG 2.5.7), and with `oncolumnmove` each column's two that move it one place on the board. |
| `children` | `Element` | required | The `KanbanColumn`s. |

### KanbanColumn

| Prop | Type | Default | Description |
|---|---|---|---|
| `index` | `usize` | required | The column's position on the board, from 0. Key the column by its data, not by this. |
| `label` | `String` | required | The column's name: the header's text, the card list's accessible name, its entry in every Move to menu and the announcements. |
| `header` | `Option<Element>` | `None` | The header's content instead of `label`, e.g. with a count. `label` stays the list's name. |
| `parts` | `Parts<KanbanColumnPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | The column's `KanbanCard`s. |

### KanbanCard

| Prop | Type | Default | Description |
|---|---|---|---|
| `index` | `usize` | required | The card's position in its column's data, from 0. Key the card by its data, not by this. A filtered column may skip indices: the drag, keys, move buttons and Move to go by the cards it shows. |
| `label` | `Option<String>` | `"Item {n}"` | Names the card in its controls and the announcements. Unset, the handle reads the card's content, and the other controls and announcements `SortableLabels::item` with its position. |
| `parts` | `Parts<KanbanCardPart>` | - | Styles for the inner parts in the Style API tab, under `sx`. |
| `children` | `Element` | required | The card's content, between the handle and the move buttons. |

Like every component, all three also take the shared props `sx`, `class`,
`states`, and any extra HTML attributes.

## Style API

Style a part with the `parts` prop, or address it as `[data-slot='…']` in your
own CSS. The names are stable. [Styling](styling.md#style-api) explains how
parts work.

| Part | `data-slot` | Description |
|---|---|---|
| `KanbanColumnPart::Header` | `header` | The header naming the column. |
| `KanbanColumnPart::List` | `list` | The list holding the cards. |
| `KanbanColumnPart::Handle` | `handle` | The header's reorder handle, with `oncolumnmove`. |
| `KanbanColumnPart::MoveEarlier` | `move-earlier` | The header's move back button, with `oncolumnmove` and `move_buttons`. |
| `KanbanColumnPart::MoveLater` | `move-later` | The header's move forward button. |
| `KanbanCardPart::Handle` | `handle` | The drag handle. |
| `KanbanCardPart::Content` | `content` | The wrapper round the card's content. |
| `KanbanCardPart::MoveEarlier` | `move-earlier` | The move up button. |
| `KanbanCardPart::MoveLater` | `move-later` | The move down button. |
| `KanbanCardPart::MoveTo` | `move-to` | The Move to column menu's trigger. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Space` or `Enter` | On a handle, lifts its card. Lifted, drops it where it is. |
| `Up` or `Down` | Moves a lifted card one slot in its column. |
| `Home` or `End` | Moves a lifted card to the top or bottom of its column. |
| `Left` or `Right` | Puts a lifted card in the next column on that side (right to left, mirrored), at the slot it was lifted at, or last if that column has fewer cards. The lift ends there and the handle keeps focus. |
| `Escape` | Puts a lifted or dragged card back where it was. |
| `Space` or `Enter` | On a column's reorder handle, lifts the column; lifted, drops it where it is. |
| `Left` or `Right` | Moves a lifted column one place on the board (right to left, mirrored). Home and End move it first or last; Escape puts it back. |

### Libero handles

- Each column is a list named by its `label`, and each card a list item.
- Each card's handle is a button of at least 24px (WCAG 2.5.8) named with the
  card, "Reorder Write", and described by how to move it with the keys. On a
  touch screen with `move_buttons` on, the description points to the move
  buttons and the Move to column menu instead, from
  `KanbanLabels::touch_instructions`; with `move_buttons` off, to the menu alone,
  from `KanbanLabels::touch_move_to`.
- Each card has Move up and Move down buttons and a Move to column menu, named
  with the card, "Move Write up", "Move Write to column", so nothing needs a
  drag (WCAG 2.5.7). The menu lists every column, the card's own disabled.
- A card sent to another column by the menu lands at its end, and focus
  follows to its Move to button. A dragged card lands where it was let go, and
  its handle keeps focus.
- One status region for the board says each lift, move, drop and cancel, with
  the column and the card's place when the column changes. A drop or Move to is
  said once the card shows at its new slot, so a move `onmove` refuses is not
  announced. The words come from `SortableLabels` and `KanbanLabels` in the
  active `Localization`.
- A drag starts after 4px of pointer movement (8px on touch). Only the handle
  takes a touch: a swipe on the rest of a card scrolls. The column under the
  card's centre takes it; let go off the board, it goes back.
- On a board wider than its container, or with a height of its own that its
  columns outgrow, a card dragged near an edge scrolls the board that way. Up
  and down only once the card has moved toward that edge.
- In a narrow column the move buttons wrap below the card's content.
- With `oncolumnmove`, each column header has a reorder handle named "Reorder
  To do column" and described by the column keys, and with `move_buttons` a
  Move backward and a Move forward button, "Move To do column forward". The
  status region says the column's lift, moves and drop, "To do column moved to
  position 2 of 3.", and focus stays on the handle or button used.

### You must

- Give each `KanbanCard` a `label`, or its controls and the announcements name
  it by position ("Item 2").

### Example

A task board where each `KanbanCard` has `label: title`. On the "Write" card,
Move to column opens a menu of the columns; picking Done moves the card to the
end of Done, the status region says so, and focus lands on the card's Move to
button there.

### Limits

- Left and Right move a lifted card one column per press: it is dropped there, so
  lift it again for the next column. On Android a right to left board's Left and
  Right are not mirrored yet.
- A drag scrolls the board, not the page: to reach a card or column beyond the
  window, use the Move to menu.
- A board with a height of its own scrolls its cards by wheel and touch; the
  scrollbar's gutter stays reserved, so a drag never changes the board's width.
- A column moves by its handle's keys and its move buttons only; dragging a
  column by pointer is not built yet.
- In a filtered column a dragged card lands just before the next card shown,
  or just after the last one; Move to lands after the last card shown, before
  any hidden ones that follow.

## Data attributes

State tokens on `data-state`.

| Token | On | Condition |
|---|---|---|
| `sorting` | `KanbanColumn`, `KanbanCard` | A card is being dragged: by keyboard in this column, by pointer anywhere on the board. On `KanbanColumn` also while a column is lifted. |
| `dragging` | `KanbanCard`, `KanbanColumn` | This card, or this column by its reorder handle, is the one lifted. |
| `target` | `KanbanColumn` | A card dragged by pointer would land in this column. |
