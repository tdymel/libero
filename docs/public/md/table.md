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

`width` and `min_width` size a column, `header_render` draws its header, and
`column_defaults` sets what every column starts from, say a minimum width.
Columns of one kind, say prices, share a `const ColumnType` with
`column("Price").of(&MONEY).value(..)`.

`selectable` adds a checkbox per row and a select-all box. The selection is a
list of `row_key`s, so it stays with its rows through a sort. Hold it yourself
with `selection` and `onselectionchange`. `multi_sort` lets Shift-click, or a
tap on a touch screen, sort by one more column.

`page_sizes` pages the rows after sorting them, with a page-size picker, the
shown range and page buttons under the table. `page` and `page_size` hold the
state yourself, as `sort` does. For server-side data set `manual_sort` and
`manual_pagination`, pass the current page as `data` and the total as
`row_count`. Select-all covers the rows on every page.

`column_menu` adds a menu to each header to sort, hide the column, or show and
hide the others. `.hideable(false)` keeps a column out of it. To hold the hidden
columns yourself, pass `hidden_columns` and update it from
`onhiddencolumnschange`.

`scroll` wraps a table wider than its container in a `ScrollArea` that
scrolls sideways. The demo's switch also sets `sx().min_width("640px")`, so the
three columns overflow at any width. `max_height` caps a long table's height:
its rows scroll under a header that stays put.

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

Ten rows a page, with a picker for 10, 25 or 50. A sort goes back to page 1,
and a new page size keeps the first shown row in view.

```rust
use dioxus::prelude::*;
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
}

#[component]
fn Demo() -> Element {
    let items: Vec<Item> = (1..=95).map(|id| Item { id }).collect();

    rsx! {
        Table {
            caption: "Items",
            data: items,
            columns: vec![column("Id").value(|i: &Item| i.id).sortable()],
            page_sizes: vec![10, 25, 50],
        }
    }
}
```

Server-side data: the table shows `data` as given and only reports what the
reader asks for. Fetch the page for `sort`, `page` and `page_size`, and pass
the total as `row_count`.

```rust
use dioxus::prelude::*;
use libero::components::{Table, TableSort, column};

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
}

#[component]
fn Demo() -> Element {
    let mut sort = use_signal(Vec::<TableSort>::new);
    let mut page = use_signal(|| 1u32);
    let mut page_size = use_signal(|| 25usize);
    // Your fetch, keyed by `sort()`, `page()` and `page_size()`.
    let rows: Vec<Item> = Vec::new();

    rsx! {
        Table {
            caption: "Items",
            data: rows,
            columns: vec![column("Id").value(|i: &Item| i.id).sortable()],
            manual_sort: true,
            manual_pagination: true,
            row_count: 1200,
            sort: sort(),
            onsortchange: move |next| sort.set(next),
            page: page(),
            onpagechange: move |next| page.set(next),
            page_size: page_size(),
            onpagesizechange: move |next| page_size.set(next),
            page_sizes: vec![25, 50, 100],
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

A long table takes `max_height`: the rows scroll, both ways, in a `ScrollArea`
under a header that stays put. The header takes the page surface's colour; on
another background, set it with `sx().selector("& thead th", ..)`.

```rust
use dioxus::prelude::*;
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
}

#[component]
fn Demo() -> Element {
    let people: Vec<Person> = (1..=50).map(|i| Person { name: format!("Person {i}") }).collect();

    rsx! {
        Table {
            caption: "Team members",
            max_height: "320px",
            data: people,
            columns: vec![column("Name").value(|p: &Person| p.name.clone()).sortable()],
        }
    }
}
```

`selectable` with a controlled `selection` of `row_key`s, and `multi_sort`:
Shift-click a second header to break ties of the first. Each sorted header
shows its place.

```rust
use dioxus::prelude::*;
use libero::components::{SortDirection, Table, TableSort, column};

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: String,
    role: String,
}

#[component]
fn Demo() -> Element {
    let people: Vec<Person> = Vec::new();
    let mut selection = use_signal(Vec::<String>::new);

    rsx! {
        Table {
            caption: "Team members",
            data: people,
            columns: vec![
                column("Name").value(|p: &Person| p.name.clone()).sortable().row_header(),
                column("Role").value(|p: &Person| p.role.clone()).sortable(),
            ],
            row_key: |p: &Person| p.id.to_string(),
            selectable: true,
            selection: selection(),
            onselectionchange: move |next| selection.set(next),
            multi_sort: true,
            default_sort: vec![
                TableSort::new("Role", SortDirection::Ascending),
                TableSort::new("Name", SortDirection::Ascending),
            ],
        }
    }
}
```

## Props

### Table

| Prop | Type | Default | Description |
|---|---|---|---|
| `data` | `Vec<T>` | required | One row each, in source order until a column is sorted. |
| `columns` | `Vec<Column<T>>` | required | Built with `column(..)`. |
| `column_defaults` | `ColumnDefaults` | none | Settings every column starts from: `ColumnDefaults::new().align(..).width(..).min_width(..)`. A column's own setting wins. |
| `caption` | `Option<String>` | `None` | A visible title above the header row, and the table's accessible name. |
| `empty` | `Option<Element>` | `None` | Shown in one full-width row when `data` is empty. Unset, the row reads the localized `table.no_rows`, "No rows". |
| `scroll` | `bool` | `false` | Wraps the table in a `ScrollArea` that scrolls sideways. `class`, `sx` and `attributes` stay on the table. |
| `max_height` | `Option<String>` | `None` | Caps the table's height, any CSS length. The rows scroll in a `ScrollArea`, both ways, under a header that stays put. The header takes the page surface's colour: on another background, set it with `sx().selector("& thead th", ..)`. |
| `sort` | `Option<Vec<TableSort>>` | `None` | The sorted columns, empty for source order. Set, the sort is controlled: pair it with `onsortchange`. Without `multi_sort`, one column sorts, the first entry naming a sortable header. |
| `default_sort` | `Vec<TableSort>` | `[]` | Seeds the sort once. Ignored when `sort` is set. |
| `onsortchange` | `EventHandler<Vec<TableSort>>` | `None` | Called with the sort a header click asks for: ascending, then descending, then empty. |
| `multi_sort` | `bool` | `false` | Sorts by several columns, the first entry first. Shift, Ctrl or Cmd with a header click, or any tap on a touch screen, adds the column after the sorted ones, then flips and removes it. A plain click sorts by that column alone. Each sorted header shows its place. |
| `selectable` | `bool` | `false` | Adds a checkbox column, with a select-all box in its header. Set `row_key` with it, or the selection sticks to positions in `data`. |
| `selection` | `Option<Vec<String>>` | `None` | The selected rows' `row_key`s. Set, the selection is controlled: pair it with `onselectionchange`. It survives a sort. |
| `default_selection` | `Vec<String>` | `[]` | Seeds the selection once. Ignored when `selection` is set. |
| `onselectionchange` | `EventHandler<Vec<String>>` | `None` | Called with the selection a checkbox asks for. Select-all covers every row of `data` and keeps keys of rows not in it, say from another page of a server. |
| `row_key` | `RowFn<T, String>` | the row's index | A row's identity, unique per row, from a `\|row: &T\| ..` closure. Its DOM node follows it through a sort or a data change, so focus and state inside a row stay with it. |
| `onrowclick` | `EventHandler<T>` | `None` | Called with the clicked row. Pointer only: give keyboard users a button or link in a cell for the same action. |
| `row_states` | `RowFn<T, States>` | `None` | A row's states, rendered as its `data-state`. Style them with `sx().selector("& tbody tr", sx().when(..))`. |
| `row_attrs` | `RowFn<T, Vec<Attribute>>` | `None` | Extra attributes on a row's `tr`. |
| `size` | `Size` | theme (md) | Cell padding and font size. |
| `striped` | `bool` | `false` | Shades every other body row. |
| `page` | `Option<u32>` | `None` | The shown page, 1-based. Set, the page is controlled: pair it with `onpagechange`. A page past the end shows the last one. |
| `default_page` | `u32` | `1` | Seeds the page once. Ignored when `page` is set. |
| `onpagechange` | `EventHandler<u32>` | `None` | Called with the page a page button asks for. A sort change asks for page 1, and a new page size for the page that keeps the first shown row. |
| `page_size` | `Option<usize>` | `None` | Rows per page. Set, the size is controlled: pair it with `onpagesizechange`. Turns on pagination. |
| `default_page_size` | `Option<usize>` | `None` | Seeds the page size once. Turns on pagination. |
| `onpagesizechange` | `EventHandler<usize>` | `None` | Called with the size picked in the page-size picker. |
| `page_sizes` | `Vec<usize>` | `[]` | The page-size picker's choices; empty hides the picker. Turns on pagination, the first one seeding the size. No cap. |
| `manual_sort` | `bool` | `false` | `data` comes sorted, say from a server: a header click only reports through `onsortchange`. |
| `manual_pagination` | `bool` | `false` | `data` is the current page only: the table draws the page controls and leaves the slicing to you. |
| `row_count` | `Option<usize>` | data's length | Rows over all pages with `manual_pagination`, for the page count and the range text. |
| `hidden_columns` | `Option<Vec<String>>` | `None` | The hidden columns' headers. Set, visibility is controlled: pair it with `onhiddencolumnschange`. A hidden sorted column keeps sorting. |
| `default_hidden_columns` | `Vec<String>` | `[]` | Seeds the hidden columns once. Ignored when `hidden_columns` is set. |
| `onhiddencolumnschange` | `EventHandler<Vec<String>>` | `None` | Called with the hidden columns a column menu pick asks for. |
| `column_menu` | `bool` | `false` | Puts a menu button in each header: sort ascending or descending, unsort, add to the sort with `multi_sort`, hide the column, and a Columns submenu that shows or hides the others. |
| `column_menu_parts` | `Parts<MenuPart>` | none | The column menus' `parts`, the `Menu` page's Style API. The menus open in a portal, out of the table's `sx`. |

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
| `align` | `CellAlign` | follows the cell type | Overrides the alignment the cell type chose and `column_defaults`. |
| `width` | `String` | `None` | The column's width, any CSS length. Columns without one share the rest. |
| `min_width` | `String` | `None` | The narrowest the column gets, any CSS length. |
| `header_render` | `fn() -> Element` | `None` | Replaces the header's body, inside the sort button when sortable. The header text stays the column's name in `TableSort`. Capture signals, not values: the closure is not compared, so a changed value does not redraw the header. |
| `of` | `&ColumnType<V>` | `None` | Before `value`: starts the column from a shared `const` type, its alignment, widths and a `format` over the value. `V` must match `value`'s; the column's own settings win. |
| `row_header` | `bool` | `false` | Renders the column's cells as `th scope="row"`, so a screen reader names each row by it. One per table, usually the first. |
| `hideable` | `bool` | `true` | Whether the column menu offers to hide the column. `hidden_columns` still hides it. |

`column()` is a builder, not a component, so it takes no shared props.

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | With `scroll` or `max_height`, when the table overflows and has no button in it: enters the scroll region, a tab stop. |
| `Left` or `Right` or `Up` or `Down` or `PageUp` or `PageDown` | In the scroll region, or on a header button inside it: scrolls the table. |
| `Enter` or `Space` | On a sortable header, a button: sorts by that column, flips it, then unsorts. |
| `Shift+Enter` or `Shift+Space` | With `multi_sort`, on a sortable header: adds that column after the sorted ones. |
| `Space` | On a row's checkbox: selects or deselects the row. On the header checkbox: selects or clears every row. |
| `Enter` or `Space` or `Down` | With `column_menu`, on a header's menu button: opens the column menu, keyed like `Menu`. |

### Libero handles

- An unnamed table warns in a debug build.
- With `scroll` or `max_height`, an overflowing scroll area with no button in
  it is a tab stop, a `role="region"` named like the table. With sort or menu
  buttons in the header it is no stop: the arrows scroll it from a focused
  button.
- The column menu button shows on its header's hover or focus with a mouse,
  always on a touch screen. An end-aligned header puts it first, in the DOM
  too, so Tab follows what is seen.
- Only sorted headers carry `aria-sort`. With several, each sort button's name
  adds its place, "sort order 2".
- Each row's checkbox is named "Select" plus its row header's text, else its
  first cell's. The header checkbox reads mixed while some rows are selected.
- A selected row carries `aria-selected="true"`, and a polite live region says
  the new count, "2 rows selected", after each change.
- A tap on a touch screen has no Shift key, so with `multi_sort` a tap always
  adds the column.
- `selectable` without `row_key` warns in a debug build.
- `.row_header()` cells render as `th scope="row"`, so a screen reader reads
  that name as it moves down any other column. They look like the other cells.
- Paginated, the page buttons sit in a `nav` named after the caption, the
  page-size picker is labelled, and a page change announces the new range,
  "4–6 of 7", politely. The first render announces nothing.
- With `column_menu`, each menu button is named after its column, "Age column
  options", and the header keeps its text as its name. The Columns submenu lists
  checkbox items, and the last shown column cannot be hidden.

### You must

- Name every table. `caption` shows a title and names it, `aria_labelledby`
  points at a heading already on the page, and `aria_label` names it without
  text.
- Set `scroll: true` on a table wider than its container, or `max_height` on a
  long one.
- Mark the column that names a row with `.row_header()`.
- With `onrowclick`, also put a button or link for that action in a cell. A
  row is not a tab stop, so a keyboard cannot click it.
- With `selectable`, give the rows a `.row_header()` column, so each checkbox
  is named by something unique.

## Theme defaults

`TableDefaults` on the theme.

| Field | Type | Description |
|---|---|---|
| `size` | `Size` | The size when the prop is unset, `Md`. |
| `sizes` | `Sizes<TableSizeLevel>` | Per size: `font_size`, `padding_x` and `padding_y` of every cell. `Md` is 14px, 12px and 10px. |
| `border_color` | `ColorValue` | Color of the header and row rules. |
| `hover_color` | `ColorValue` | Row background while hovered, `muted.2`: one step past the stripe. |
| `stripe_color` | `ColorValue` | Every other body row's background with `striped`, `muted.1`. |
| `selected_color` | `ColorValue` | A selected row's background, hovered or not, `primary.1`. |

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
| `--lsx-table-selected` | `background` of a selected body row. |

## Data attributes

`data-align` on a `th`/`td` carries the cell alignment, and is omitted for the
default `start`.

| Attribute | Condition |
|---|---|
| `data-align="center"` | The column's alignment is `center`. |
| `data-align="end"` | The column's alignment is `end`, as for every numeric cell type. |
| `data-sortable` | On a sortable header. |
| `aria-sort` | On sorted headers only, `ascending` or `descending`. |
| `data-sort-order` | On the place badge inside a sorted header's button, with two or more sorted columns. |
| `data-select` | On the checkbox column's `th` and `td`s. |
| `aria-selected` | On every body row of a `selectable` table, `true` or `false`. |
| `data-empty` | On the body row that holds `empty`. |
| `data-state` | On the table: `size-{size}`, plus `striped` and `row-click` when set. On a body row: its active `row_states`. |
| `data-slot="range"` | On the paginated table's range text, "1–10 of 95". |
