# Table

Crate: `libero`
Import: `use libero::components::{Table, column};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/table>
Index: [index.md](index.md) lists every other page
Description: A sortable data table built from a row type and a list of column definitions.

A table built from `data` and `columns`. Each column comes from `column(..)`,
with a header and a `value` that reads one cell out of a row. The cell's type
sets the sort order and alignment, so a numeric column sorts numerically and
aligns right on its own.

`sortable` turns a header into a button. The first click sorts ascending, the
next flips it, and a third restores source order. `render` changes only what a
cell draws, so a Role column rendered as a [Chip](chip.md) still sorts by its
text. `format` does the same for the cell's text: the Bonus column prints a
percent and still sorts by number.

`default_sort` sorts the first render. To hold the sort yourself, for a
server-side query or a saved view, pass `sort` and update it from
`onsortchange`.

`row_key` gives each row an identity, so its DOM node follows it when rows are
added, removed or sorted. `size` sets the cell padding and font size, and
`striped` shades every other row.

## Usage

```rust
use dioxus::prelude::*;
use libero::components::{Chip, Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    role: String,
    bonus: Option<f64>,
}

#[component]
fn Demo() -> Element {
    let people = vec![
        Person { name: "Ada Lovelace".into(), role: "Owner".into(), bonus: Some(12.5) },
        Person { name: "Grace Hopper".into(), role: "Admin".into(), bonus: Some(8.0) },
        Person { name: "Alan Turing".into(), role: "Viewer".into(), bonus: None },
    ];

    rsx! {
        Table {
            caption: "Team members",
            data: people,
            columns: vec![
                column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
                column("Role")
                    .value(|p: &Person| p.role.clone())
                    .sortable()
                    .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
                column("Bonus")
                    .value(|p: &Person| p.bonus)
                    .format(|p: &Person| p.bonus.map(|b| format!("{b:.1} %")).unwrap_or_default())
                    .sortable(),
            ],
        }
    }
}
```

Strings sort as text and align left. Every integer and float sorts numerically
and aligns right. `bool` prints `true` or `false`. `Option<V>` keeps the inner
type's alignment, renders `None` empty and sorts it last in both directions.

```rust,ignore
|p: &Person| p.name.clone()  // String      -> text sort, start-aligned
|p: &Person| p.name.len()   // usize       -> numeric sort, end-aligned
|p: &Person| p.bonus        // Option<f64> -> None renders empty, sorts last
```

Your own type joins them with one `impl CellValue`. `cell_text` is required,
and `sort_key` and `align` have defaults.

```rust
use dioxus::prelude::*;
use libero::components::{CellAlign, CellValue, SortKey, Table, column};

#[derive(Clone, PartialEq)]
struct Money(i64);

impl CellValue for Money {
    fn cell_text(&self) -> String {
        format!("{}.{:02}", self.0 / 100, self.0 % 100)
    }

    fn sort_key(&self) -> SortKey {
        SortKey::num(self.0 as f64)
    }

    // An associated function, since alignment belongs to the type.
    fn align() -> CellAlign {
        CellAlign::End
    }
}

#[derive(Clone, PartialEq)]
struct Row {
    label: String,
    total: Money,
}

#[component]
fn Demo() -> Element {
    let rows = vec![
        Row { label: "January".into(), total: Money(125_00) },
        Row { label: "February".into(), total: Money(98_50) },
    ];

    rsx! {
        Table {
            aria_label: "Monthly totals",
            data: rows,
            columns: vec![
                column("Month").value(|r: &Row| r.label.clone()).sortable(),
                column("Total").value(|r: &Row| r.total.clone()).sortable(),
            ],
        }
    }
}
```

`align` on the column overrides the alignment the cell type chose.

A controlled sort lives in your state. A `TableSort` names its column by the
header text; an empty `Vec` is source order.

```rust
use dioxus::prelude::*;
use libero::components::{SortDirection, Table, TableSort, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
    age: u32,
}

#[component]
fn Demo() -> Element {
    let mut sort = use_signal(|| vec![TableSort::new("Age", SortDirection::Descending)]);
    let people = vec![
        Person { name: "Ada".into(), age: 36 },
        Person { name: "Grace".into(), age: 45 },
    ];

    rsx! {
        Table {
            aria_label: "People",
            data: people,
            columns: vec![
                column("Name").value(|p: &Person| p.name.clone()).sortable(),
                column("Age").value(|p: &Person| p.age).sortable(),
            ],
            sort: sort(),
            onsortchange: move |next| sort.set(next),
        }
    }
}
```

Rows keyed by an id, a click on a row, and a state that styles some rows.
`onrowclick` is for the pointer only: keep a button or link in a cell for the
keyboard.

```rust
use dioxus::prelude::*;
use libero::components::{States, Table, column};
use libero::sx::sx;

#[derive(Clone, PartialEq)]
struct Order {
    id: u64,
    item: String,
    late: bool,
}

#[component]
fn Demo() -> Element {
    let mut open = use_signal(|| None::<u64>);
    let orders = vec![
        Order { id: 41, item: "Desk".into(), late: false },
        Order { id: 42, item: "Lamp".into(), late: true },
    ];

    rsx! {
        Table {
            aria_label: "Orders",
            size: "sm",
            striped: true,
            data: orders,
            columns: vec![
                column("Order").value(|o: &Order| o.id).row_header(),
                column("Item").value(|o: &Order| o.item.clone()).sortable(),
            ],
            row_key: |o: &Order| o.id.to_string(),
            row_states: |o: &Order| States::new().with("late", o.late),
            onrowclick: move |o: Order| open.set(Some(o.id)),
            sx: sx().selector("& tbody tr", sx().when("late", sx().color("red.7"))),
        }
    }
}
```

A table wider than its container takes `scroll: true`, and `empty` fills the
body while `data` has no rows. Without `empty` that row reads the localized
`table.no_rows`, "No rows".

```rust
use dioxus::prelude::*;
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
}

#[component]
fn Demo() -> Element {
    let people: Vec<Person> = Vec::new();

    rsx! {
        Table {
            caption: "Team members",
            scroll: true,
            empty: rsx! { "No team members yet." },
            data: people,
            columns: vec![column("Name").value(|p: &Person| p.name.clone()).sortable()],
        }
    }
}
```

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | With `scroll: true`: enters the scroll region, a tab stop. |
| `Left` or `Right` or `Up` or `Down` | In the scroll region: scrolls the table. |
| `Enter` or `Space` | On a sortable header, a button: sorts by that column, flips it, then unsorts. |

### Libero handles

- An unnamed table warns in a debug build.
- With `scroll: true` the wrapper is a `role="region"` named like the table.
- Only the sorted header carries `aria-sort`.
- `.row_header()` cells render as `th scope="row"`, so a screen reader reads
  that name as it moves down any other column. They look like the other cells.

### You must

- Name every table. `caption` shows a title and names it, `aria_labelledby`
  points at a heading already on the page, and `aria_label` names it without
  text.
- Set `scroll: true` on a table wider than its container.
- Mark the column that names a row with `.row_header()`.
- With `onrowclick`, also put a button or link for that action in a cell. A
  row is not a tab stop, so a keyboard cannot click it.

## Props

### Table

| Prop | Type | Default | Description |
|---|---|---|---|
| `data` | `Vec<T>` | required | One row each, in source order until a column is sorted. |
| `columns` | `Vec<Column<T>>` | required | Built with `column(..)`. |
| `caption` | `Option<String>` | `None` | A visible title above the header row, and the table's accessible name. |
| `empty` | `Option<Element>` | `None` | Shown in one full-width row when `data` is empty. Unset, the row reads the localized `table.no_rows`, "No rows". |
| `scroll` | `bool` | `false` | Wraps the table in a named, focusable region that scrolls sideways. `class`, `sx` and `attributes` stay on the table. |
| `sort` | `Option<Vec<TableSort>>` | `None` | The sorted column, empty for source order. Set, the sort is controlled: pair it with `onsortchange`. One column sorts, the first entry naming a sortable header. |
| `default_sort` | `Vec<TableSort>` | `[]` | Seeds the sort once. Ignored when `sort` is set. |
| `onsortchange` | `EventHandler<Vec<TableSort>>` | `None` | Called with the sort a header click asks for: ascending, then descending, then empty. |
| `row_key` | `RowFn<T, String>` | the row's index | A row's identity, unique per row, from a `\|row: &T\| ..` closure. Its DOM node follows it through a sort or a data change, so focus and state inside a row stay with it. |
| `onrowclick` | `EventHandler<T>` | `None` | Called with the clicked row. Pointer only: give keyboard users a button or link in a cell for the same action. |
| `row_states` | `RowFn<T, States>` | `None` | A row's states, rendered as its `data-state`. Style them with `sx().selector("& tbody tr", sx().when(..))`. |
| `row_attrs` | `RowFn<T, Vec<Attribute>>` | `None` | Extra attributes on a row's `tr`. |
| `size` | `Size` | theme (md) | Cell padding and font size. |
| `striped` | `bool` | `false` | Shades every other body row. |

Like every component, `Table` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes, `aria_label` among them.

### column()

| Prop | Type | Default | Description |
|---|---|---|---|
| `header` | `String` | required | The column's title, the argument to `column(..)`. |
| `value` | `fn(&T) -> V` | required | Reads one cell out of a row. `V`'s `CellValue` impl sets the sort order and alignment. |
| `sortable` | `bool` | `false` | Turns the header into a sort button. |
| `render` | `fn(&T) -> Element` | `None` | Replaces the cell body. Sorting still uses `value`. |
| `format` | `fn(&T) -> String` | `None` | Replaces the cell text, say a price with its currency. Sorting and alignment still follow `value`. |
| `align` | `CellAlign` | follows the cell type | Overrides the alignment the cell type chose. |
| `row_header` | `bool` | `false` | Renders the column's cells as `th scope="row"`, so a screen reader names each row by it. One per table, usually the first. |

`column()` is a builder, not a component, so it takes no shared props.

## Theme defaults

`TableDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | The size when the prop is unset, `Md`. |
| `sizes` | `Sizes<TableSizeLevel>` | Per size: `font_size`, `padding_x` and `padding_y` of every cell. `Md` is 14px, 12px and 10px. |
| `border_color` | `ColorValue` | Color of the header and row rules. |
| `hover_color` | `ColorValue` | Row background while hovered. |
| `stripe_color` | `ColorValue` | Every other body row's background with `striped`. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-table-font-size-{size}` | `font-size` of the table at that size. |
| `--lsx-table-padding-x-{size}` | Horizontal padding on every `th`/`td` at that size. |
| `--lsx-table-padding-y-{size}` | Vertical padding on every `th`/`td` at that size. |
| `--lsx-table-pad-x`, `--lsx-table-pad-y` | The padding resolved on the table for its size. |
| `--lsx-table-border-color` | Color of the `1px` header and row rules. |
| `--lsx-table-hover` | `background` of a hovered body row. |
| `--lsx-table-stripe` | `background` of every other body row with `striped`. |

## Data attributes

`data-align` on a `th`/`td` carries the cell alignment, and is omitted for the
default `start`.

| Attribute | Condition |
|---|---|
| `data-align="center"` | The column's alignment is `center`. |
| `data-align="end"` | The column's alignment is `end`, as for every numeric cell type. |
| `data-sortable` | On a sortable header. |
| `aria-sort` | On the sorted header only, `ascending` or `descending`. |
| `data-empty` | On the body row that holds `empty`. |
| `data-state` | On the table: `size-{size}`, plus `striped` and `row-click` when set. On a body row: its active `row_states`. |
