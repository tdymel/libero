# Kanban

Crate: `libero`
Import: `use libero::components::{Kanban, KanbanCard, KanbanColumn, KanbanMove};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/kanban>
Index: [index.md](index.md) lists every other page
Description: A board of columns whose cards move by drag in a column or to another, by keyboard or move buttons in a column, and to another column by a Move to menu.

A board of columns. A card moves by dragging its handle, in its column or to
another; by keyboard or with its move buttons in its column; and to another
column with its Move to menu. `onmove` gets a `KanbanMove` with the old and new
column and index, and `apply` does it to a `Vec<Vec<T>>`. The cards are your
content: the board draws no card look. The docs demo builds Jira-style issue
cards from `Text`, `Badge`, `Icon` and `Avatar`, with a count in each column's
`header`.

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

## Props

### Kanban

| Prop | Type | Default | Description |
|---|---|---|---|
| `onmove` | `EventHandler<KanbanMove>` | required | Fires when a card changed place: on a drop, a move button or a Move to entry. Apply it to your data with `KanbanMove::apply`; until then the board keeps its old order. |
| `move_buttons` | `bool` | `true` | Each card's two buttons that move it one slot in its column without dragging (WCAG 2.5.7). |
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
| `index` | `usize` | required | The card's position in its column, from 0. Key the card by its data, not by this. |
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
| `Escape` | Puts a lifted or dragged card back where it was. |

### Libero handles

- Each column is a list named by its `label`; each card is a list item.
- Each card's handle is a button named by `SortableLabels::handle` with the
  card's name ("Reorder Write"), at least 24px square (WCAG 2.5.8), described
  by `SortableLabels::instructions`.
- Each card has a move up and a move down button in its column, and a Move to
  column menu button named by `KanbanLabels::move_to`, each naming the card
  ("Move Write up", "Move Write to column"). The menu lists every column by
  `label`, the card's own disabled. Neither needs a drag (WCAG 2.5.7).
- A card moved to another column by its Move to menu lands at that column's
  end, and its Move to button takes the focus there. One dragged there lands
  where it was let go, and its handle keeps the focus.
- One `role="status"` region for the board announces each lift, move, drop and
  cancel, and a move or a drop in another column with the column's name and the card's
  position, from the `SortableLabels` and `KanbanLabels` templates of the
  active `Localization`.
- A drag starts after the pointer moved 4px (8px for a touch); only the handle
  takes a touch, so a swipe on the rest of a card scrolls. The column under the
  dragged card's centre takes it, marked `target`; let go off the board, the
  card goes back.
- On a board wider than its container, a card dragged within 48px of a side
  edge scrolls the board that way, faster the closer, until that edge's column
  is in view.
- In a narrow column, where the content would get less than 8rem, the move
  buttons wrap below it.

### You must

- Give each `KanbanCard` a `label`, or its controls and the announcements name
  it by position ("Item 2").

### Limits

- The keyboard drag moves a card within its column only. Moving to another
  column is the Move to menu.
- A drag scrolls the board sideways only, not the page: to reach a card or
  column above or below the window, use the Move to menu.
- Columns keep their place: the board has no column reorder.

## Data attributes

State tokens on `data-state`.

| Token | On | Condition |
|---|---|---|
| `sorting` | `KanbanColumn`, `KanbanCard` | A card is being dragged: by keyboard in this column, by pointer anywhere on the board. |
| `dragging` | `KanbanCard` | This card is the one dragged. |
| `target` | `KanbanColumn` | A card dragged by pointer would land in this column. |
