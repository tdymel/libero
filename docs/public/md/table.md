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

`show_quick_filter` puts a search field above the table. A row stays when every
typed word occurs in one of its shown cells; `.filterable(false)` leaves a column
out. The rows are filtered, then sorted, then paged. Hold the text yourself with
`quick_filter` and `onquickfilterchange`, or filter on a server with
`manual_filter`.

`toolbar` puts your own controls in a row above the table, the search field at
its end. `loading` shows placeholder rows while there are none yet, and a
progress bar over the rows while new ones load. `empty` and `no_results`
replace the text of the row shown without data and without matches.

`column_menu`'s Filter entry, and `header_filters`' row of fields under the
headers, filter one column each, with operators for its type: contains or
starts with for text, greater than for numbers, yes or no for booleans. The
filters all apply, together with the quick filter. Hold them yourself with
`column_filters` and `oncolumnfilterschange`.

`column_menu` adds a menu to each header to sort, hide the column, or show and
hide the others. `.hideable(false)` keeps a column out of it. To hold the hidden
columns yourself, pass `hidden_columns` and update it from
`onhiddencolumnschange`.

`column("Q1").value(..).group("Revenue")` puts a column under a group header,
shared with its neighbours of the same group; call `group` again for a nested
one. `col_span` lets a row's cell cover the next columns, say a total row's
label.

`row_detail` gives a row a toggle that opens a full-width detail row under it,
as in a master-detail view. Rows the closure answers `None` for get no toggle.
The open details are `row_key`s, held yourself with `expanded` and
`onexpandedchange`.

`onrowreorder` gives each row a drag handle and move buttons. The table hands
you the move, by positions in `data`, and `step.apply(&mut rows.write())`
applies it. Sorted or filtered, the shown order is not your data's, so the
controls turn off. With `column_menu`, Move left and Move right reorder the
columns; hold the order yourself with `column_order` and
`oncolumnorderchange`. A sorted column stays sorted wherever it moves.

`table_csv(&columns, &rows)` writes the cells as CSV, each as its column shows
it as text, and `table_text` hands them over unjoined for another format.
Saving is yours. A cell that starts with `=` runs as a formula in a
spreadsheet, so neutralise text users typed first.

`scroll` wraps a table wider than its container in a `ScrollArea` that
scrolls sideways. The demo's switch also sets `sx().min_width("640px")`, so the
three columns overflow at any width. `max_height` caps a long table's height:
its rows scroll under a header that stays put.

For thousands of rows add `virtual_row_height` to `max_height`: only the rows
in view render, each that tall. Sorting, filtering, selection and pinning work
as before.

`default_pinned_columns` holds columns at the start or end edge while the rest
scroll under them, and the column menu pins and unpins them. Start and end
follow the page's direction. Give each pinned column but the outermost a
`width`, so the next one knows where to sit.

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

While rows load, `loading` fills an empty body with placeholder rows, or lays a
progress bar over the rows already shown. `toolbar` holds your controls above
the table, with the quick filter's field at its end.

```rust
use dioxus::prelude::*;
use libero::components::{Button, Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: String,
}

#[component]
fn Demo() -> Element {
    let people: Vec<Person> = Vec::new();
    let loading = use_signal(|| true);

    rsx! {
        Table {
            caption: "Team members",
            loading: loading(),
            show_quick_filter: true,
            toolbar: rsx! { Button { variant: "outlined", size: "sm", "Add member" } },
            no_results: rsx! { "No member matches." },
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

Ten thousand rows take `virtual_row_height` too: the table renders the dozen
in view and a few beyond each edge, each row exactly 40px tall with one line per
cell. `aria-rowcount` and each row's `aria-rowindex` tell a screen reader where
it is in the whole table, and the row holding focus stays rendered while it
scrolls away.

```rust
use dioxus::prelude::*;
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Stock {
    id: u32,
    name: String,
    count: u32,
}

#[component]
fn Demo() -> Element {
    let stock: Vec<Stock> = (1..=10_000)
        .map(|id| Stock { id, name: format!("Item {id}"), count: id * 7 % 1_000 })
        .collect();

    rsx! {
        Table {
            caption: "Stock",
            max_height: "320px",
            virtual_row_height: 40.0,
            selectable: true,
            row_key: |s: &Stock| s.id.to_string(),
            data: stock,
            columns: vec![
                column("Id").value(|s: &Stock| s.id).sortable().row_header(),
                column("Name").value(|s: &Stock| s.name.clone()).sortable(),
                column("Count").value(|s: &Stock| s.count).sortable(),
            ],
        }
    }
}
```

A wide table pins columns to its edges: here the ID and Name stay at the start
and Actions at the end while the rest scroll sideways. ID, pinned further out
than Name, has a `width`, so Name sits right after it. With `column_menu`, each
header's menu pins and unpins its column.

```rust
use dioxus::prelude::*;
use libero::components::{PinnedColumns, Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: String,
}

#[component]
fn Demo() -> Element {
    let people = vec![Person { id: 1, name: "Ada Lovelace".into() }];

    rsx! {
        Table {
            caption: "Team members",
            scroll: true,
            column_menu: true,
            default_pinned_columns: PinnedColumns::default().start(["ID", "Name"]).end(["Actions"]),
            data: people,
            columns: vec![
                column("ID").value(|p: &Person| p.id).width("4rem"),
                column("Name").value(|p: &Person| p.name.clone()).width("12rem"),
                column("Notes").value(|p: &Person| format!("{} wrote the first program", p.name)),
                column("Actions").value(|_: &Person| "Edit".to_string()),
            ],
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

A quick filter over the name and role, held by the caller; the id column is
not searched.

```rust
use dioxus::prelude::*;
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: String,
    role: String,
}

#[component]
fn Demo() -> Element {
    let people: Vec<Person> = Vec::new();
    let mut query = use_signal(String::new);

    rsx! {
        Table {
            caption: "Team members",
            data: people,
            columns: vec![
                column("Id").value(|p: &Person| p.id).filterable(false),
                column("Name").value(|p: &Person| p.name.clone()).row_header(),
                column("Role").value(|p: &Person| p.role.clone()),
            ],
            show_quick_filter: true,
            quick_filter: query(),
            onquickfilterchange: move |next| query.set(next),
        }
    }
}
```

`row_detail` with a controlled `expanded`, and the same rows as CSV:

```rust
use dioxus::prelude::*;
use libero::components::{Table, column, table_csv};

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: String,
    notes: Option<String>,
}

#[component]
fn Demo() -> Element {
    let people: Vec<Person> = Vec::new();
    let mut expanded = use_signal(Vec::<String>::new);
    let columns = || vec![column("Name").value(|p: &Person| p.name.clone()).row_header()];
    let csv = table_csv(&columns(), &people);

    rsx! {
        Table {
            caption: "Team members",
            data: people,
            columns: columns(),
            row_key: |p: &Person| p.id.to_string(),
            row_detail: |p: &Person| p.notes.clone().map(|notes| rsx! { "{notes}" }),
            expanded: expanded(),
            onexpandedchange: move |next| expanded.set(next),
        }
        pre { "{csv}" }
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
| `empty` | `Option<Element>` | `None` | Shown in one full-width row when `data` is empty. Unset, the row reads the localized `table.no_rows`, "No rows". When the quick filter or the column filters leave no rows, the row reads `table.no_results`, "No matching rows", instead. |
| `no_results` | `Option<Element>` | `None` | Shown in one full-width row when the quick filter or the column filters leave no rows, over the localized `table.no_results`. |
| `loading` | `bool` | `false` | Rows are on their way. While no row shows, placeholder rows fill the body, a page of them when paged, else five, and the table is `aria-busy`. With rows shown, they stay usable under a thin progress bar over the table's top edge, named by the localized `table.loading`, "Loading rows". The empty row waits until loading ends. |
| `toolbar` | `Option<Element>` | `None` | A row above the table for your own controls, say an export or add button. With `show_quick_filter` the search field joins it at the end. It wraps on a narrow screen and stays put while the table scrolls. |
| `scroll` | `bool` | `false` | Wraps the table in a `ScrollArea` that scrolls sideways. `class`, `sx` and `attributes` stay on the table. |
| `max_height` | `Option<String>` | `None` | Caps the table's height, any CSS length. The rows scroll in a `ScrollArea`, both ways, under a header that stays put. The header takes the page surface's colour: on another background, set it with `sx().selector("& thead th", ..)`. |
| `virtual_row_height` | `Option<f64>` | `None` | With `max_height`, renders only the rows in view plus a few beyond each edge, so ten thousand rows scroll like fifty. Every body row is exactly this tall in px: one line per cell, longer text ends in an ellipsis. The table then lays out fixed, columns without a `width` sharing the rest evenly. A row holding focus stays rendered while it scrolls away. Ignored with `row_detail` or `onrowreorder`, which render every row; a debug build warns. |
| `sort` | `Option<Vec<TableSort>>` | `None` | The sorted columns, empty for source order. Set, the sort is controlled: pair it with `onsortchange`. Without `multi_sort`, one column sorts, the first entry naming a sortable header. |
| `default_sort` | `Vec<TableSort>` | `[]` | Seeds the sort once. Ignored when `sort` is set. |
| `onsortchange` | `EventHandler<Vec<TableSort>>` | `None` | Called with the sort a header click asks for: ascending, then descending, then empty. |
| `multi_sort` | `bool` | `false` | Sorts by several columns, the first entry first. Shift, Ctrl or Cmd with a header click, or any tap on a touch screen, adds the column after the sorted ones, then flips and removes it. A plain click sorts by that column alone. Each sorted header shows its place. |
| `selectable` | `bool` | `false` | Adds a checkbox column, with a select-all box in its header. Set `row_key` with it, or the selection sticks to positions in `data`. |
| `selection` | `Option<Vec<String>>` | `None` | The selected rows' `row_key`s. Set, the selection is controlled: pair it with `onselectionchange`. It survives a sort. |
| `default_selection` | `Vec<String>` | `[]` | Seeds the selection once. Ignored when `selection` is set. |
| `onselectionchange` | `EventHandler<Vec<String>>` | `None` | Called with the selection a checkbox asks for. Select-all covers every row of `data` the quick filter keeps, and keeps keys of the other rows, say from another page of a server. |
| `row_key` | `RowFn<T, String>` | the row's index | A row's identity, unique per row, from a `\|row: &T\| ..` closure. Its DOM node follows it through a sort or a data change, so focus and state inside a row stay with it. |
| `onrowclick` | `EventHandler<T>` | `None` | Called with the clicked row. Pointer only: give keyboard users a button or link in a cell for the same action. |
| `row_states` | `RowFn<T, States>` | `None` | A row's states, rendered as its `data-state`. Style them with `sx().selector("& tbody tr", sx().when(..))`. |
| `row_attrs` | `RowFn<T, Vec<Attribute>>` | `None` | Extra attributes on a row's `tr`. |
| `row_detail` | `RowFn<T, Option<Element>>` | `None` | A row's detail, from a `\|row: &T\| ..` closure. `Some` gives the row a toggle in a leading column; open, the detail shows in a full-width row under it. Called for the shown rows on every render. Set `row_key` with it, or open details stick to positions in `data`. |
| `expanded` | `Option<Vec<String>>` | `None` | The `row_key`s of the rows whose detail shows. Set, it is controlled: pair it with `onexpandedchange`. It survives a sort. |
| `default_expanded` | `Vec<String>` | `[]` | Seeds the open details once. Ignored when `expanded` is set. |
| `onexpandedchange` | `EventHandler<Vec<String>>` | `None` | Called with the open details a toggle asks for. |
| `onrowreorder` | `EventHandler<SortableMove>` | `None` | Adds a leading column with a drag handle and Move up and Move down buttons per row. Called with a move by positions in `data`; apply it with `step.apply(&mut rows)`, the table shows the old order until you do. Off while the rows are sorted or the quick filter has text. Paged, a row moves within its page. Set `row_key` with it. |
| `size` | `Size` | theme (md) | Cell padding and font size. |
| `striped` | `bool` | `false` | Shades every other body row. |
| `page` | `Option<u32>` | `None` | The shown page, 1-based. Set, the page is controlled: pair it with `onpagechange`. A page past the end shows the last one. |
| `default_page` | `u32` | `1` | Seeds the page once. Ignored when `page` is set. |
| `onpagechange` | `EventHandler<u32>` | `None` | Called with the page a page button asks for. A sort or quick-filter change asks for page 1, and a new page size for the page that keeps the first shown row. |
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
| `quick_filter` | `Option<String>` | `None` | The quick filter's text. A row stays when every word occurs, ignoring case, in the text of one of its shown, `filterable` cells. Set, the filter is controlled: pair it with `onquickfilterchange`. |
| `default_quick_filter` | `String` | `""` | Seeds the quick filter once. Ignored when `quick_filter` is set. |
| `onquickfilterchange` | `EventHandler<String>` | `None` | Called with the text typed into the quick-filter field. |
| `show_quick_filter` | `bool` | `false` | Puts a search field above the table that drives the quick filter. |
| `manual_filter` | `bool` | `false` | `data` comes filtered, say from a server: the quick filter and the column filters only report through `onquickfilterchange` and `oncolumnfilterschange`. Pair it with `manual_pagination` and `row_count` when paged. |
| `column_filters` | `Option<Vec<ColumnFilter>>` | `None` | One filter per column, by header: `ColumnFilter::new("Age", FilterOperator::GreaterThan, "30")`. A row stays when it passes all of them and the quick filter. Text operators ignore case, `Equals` too; number operators compare the value, not its formatted text, and take `1,5` as 1.5. An empty value, a number that does not parse, or an operator the column's type does not offer keeps every row. A hidden column's filter keeps filtering. Set, the filters are controlled: pair them with `oncolumnfilterschange`. |
| `default_column_filters` | `Vec<ColumnFilter>` | `[]` | Seeds the column filters once. Ignored when `column_filters` is set. |
| `oncolumnfilterschange` | `EventHandler<Vec<ColumnFilter>>` | `None` | Called with the filters a filter popover or a header filter asks for. Typed values arrive once typing pauses for 300 ms, an operator or yes/no pick at once. |
| `header_filters` | `bool` | `false` | Adds a row of filter fields under the headers, one per `filterable` column: a text field, or Any/Yes/No for a boolean column. A field edits its column's filter with the operator the filter popover set, else the type's first: Contains for text, Equals for numbers. |
| `pinned_columns` | `Option<PinnedColumns>` | `None` | The columns held at the table's start and end edges while the rest scroll sideways, by header: `PinnedColumns::default().start([..]).end([..])`. Start is the left in a left-to-right page, the right in a right-to-left one. Set, pinning is controlled: pair it with `onpinnedcolumnschange`. Pair it with `scroll` or `max_height`, and give every pinned column but the outermost on its side a `width`, which it then keeps exactly. |
| `default_pinned_columns` | `PinnedColumns` | none pinned | Seeds the pinned columns once. Ignored when `pinned_columns` is set. |
| `onpinnedcolumnschange` | `EventHandler<PinnedColumns>` | `None` | Called with the pinned columns a column menu pick asks for. |
| `column_order` | `Option<Vec<String>>` | `None` | The headers in display order. Unlisted columns follow the listed ones in `columns` order, and pinned columns keep their pinned order. Set, the order is controlled: pair it with `oncolumnorderchange`. The sort, hidden and pinned columns name headers, so they follow a moved column. |
| `default_column_order` | `Vec<String>` | `[]` | Seeds the column order once. Ignored when `column_order` is set. |
| `oncolumnorderchange` | `EventHandler<Vec<String>>` | `None` | Called with the order a column menu's Move left or Move right asks for, every header listed. |
| `resizable_columns` | `bool` | `false` | Puts a drag grip on each header's end edge, and Widen column, Narrow column (50px steps, the menu stays open) and Reset width in its `column_menu`, the keyboard and drag-free way. Double-click a grip to reset. A column opts out with `.resizable(false)` and sets its range with `.resize_limits(min, max)` in px, 50 to unbounded by default. Auto layout never draws a column narrower than its content. Blitz: use the menu, the grip does not drag reliably there. |
| `column_widths` | `Option<ColumnWidths>` | `None` | Resized widths in px by header, a `BTreeMap<String, f64>`, over the columns' own `width`. Set, the widths are controlled: pair them with `oncolumnwidthschange`. |
| `default_column_widths` | `ColumnWidths` | `{}` | Seeds the widths once. Ignored when `column_widths` is set. |
| `oncolumnwidthschange` | `EventHandler<ColumnWidths>` | `None` | Called with the widths a grip drag asks for when it ends, and with each Widen, Narrow or Reset width. |
| `column_menu` | `bool` | `false` | Puts a menu button in each header: sort ascending or descending, unsort, add to the sort with `multi_sort`, filter a `filterable` column, move the column left or right past the next shown one, widen, narrow or reset it with `resizable_columns`, pin to the start or end or unpin, hide the column, and a Columns submenu that shows or hides the others. A pinned column has no move entries. Filter opens a popover with the operators of the column's type, a value and Clear; a filtered column's header then shows a filter button that reopens it. |
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
| `resizable` | `bool` | `true` | Whether the table's `resizable_columns` gives it a grip and menu entries. |
| `resize_limits` | `(f64, f64)` | `(50, unbounded)` | How narrow and how wide a resize takes the column, in px. `min_width` still floors it on screen. |
| `header_render` | `fn() -> Element` | `None` | Replaces the header's body, inside the sort button when sortable. The header text stays the column's name in `TableSort`. Capture signals, not values: the closure is not compared, so a changed value does not redraw the header. |
| `of` | `&ColumnType<V>` | `None` | Before `value`: starts the column from a shared `const` type, its alignment, widths and a `format` over the value. `V` must match `value`'s; the column's own settings win. |
| `row_header` | `bool` | `false` | Renders the column's cells as `th scope="row"`, so a screen reader names each row by it. One per table, usually the first. |
| `filterable` | `bool` | `true` | Whether the quick filter searches the column's cell text, and whether it takes a column filter. Off for ids and codes that would match by accident. |
| `hideable` | `bool` | `true` | Whether the column menu offers to hide the column. `hidden_columns` still hides it. |
| `group` | `String` | `None` | Puts the column under a group header, shared with the adjacent columns of the same groups. Call it once per level, outermost first. The same name under another parent is another group, and a hidden column leaves its group. |
| `col_span` | `fn(&T) -> usize` | `None` | How many shown columns a row's cell covers, from this one on, say a total row's label. The covered cells are left out; the span stops at the row's end. Capture signals, not values: the closure is not compared. |

`column()` is a builder, not a component, so it takes no shared props.

### table_csv()

| Prop | Type | Default | Description |
|---|---|---|---|
| `table_csv` | `fn(&[Column<T>], &[T]) -> String` | none | The header row and one line per row as CSV (RFC 4180, CRLF), each cell as its column's text, `format` applied and `render` ignored. Pass the rows and columns in the order you want; saving the file is yours. |
| `table_text` | `fn(&[Column<T>], &[T]) -> Vec<Vec<String>>` | none | The same cells unjoined, the header row first, for your own writer. |

## Accessibility

### Keyboard

| Key | Action |
|---|---|
| `Tab` | With `scroll` or `max_height`, when the table overflows and has no button in it: enters the scroll region, a tab stop. |
| `Left` or `Right` or `Up` or `Down` or `PageUp` or `PageDown` | In the scroll region, or on a header button inside it: scrolls the table. |
| `Enter` or `Space` | On a sortable header, a button: sorts by that column, flips it, then unsorts. |
| `Shift+Enter` or `Shift+Space` | With `multi_sort`, on a sortable header: adds that column after the sorted ones. |
| `Space` | On a row's checkbox: selects or deselects the row. On the header checkbox: selects or clears every row. |
| `Enter` or `Space` | With `row_detail`, on a row's toggle: opens or closes its detail. Tab then goes into the open detail. |
| `Enter` or `Space` or `Down` | With `column_menu`, on a header's menu button: opens the column menu, keyed like `Menu`. |
| `Space` or `Enter` | With `onrowreorder`, on a row's handle: lifts the row, then drops it. Up and Down move the lifted row, Home and End to the first or last place, Escape puts it back. |
| `Escape` | In a filter popover: closes it and returns focus to the column's menu button. |
| `Tab` or `Shift+Tab` | In a filter popover: moves between its fields; past either end it closes and Tab goes on from the menu button. |

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
- `selectable` or `row_detail` without `row_key` warns in a debug build.
- Each detail toggle is a button named "Details for" plus the row's name, like
  its checkbox, with `aria-expanded`, and `aria-controls` on the detail row
  while it is open. The toggle column's header reads "Details" to a screen
  reader only.
- `.row_header()` cells render as `th scope="row"`, so a screen reader reads
  that name as it moves down any other column. They look like the other cells.
- Paginated, the page buttons sit in a `nav` named after the caption, the
  page-size picker is labelled, and a page change announces the new range,
  "4–6 of 7", politely. The first render announces nothing.
- The quick-filter field is a labelled `type="search"` input, "Search", described by the table's caption. Once
  typing pauses for half a second, a polite live region says how many rows are
  left, "2 rows".
- With `column_menu`, each menu button is named after its column, "Age column
  options", and the header keeps its text as its name. The Columns submenu lists
  checkbox items, and the last shown column cannot be hidden.
- The filter popover is a `role="dialog"` named "Filter" plus the column, with
  labelled Operator and Value fields; focus moves to the value on open, or to
  the operator when it takes none. A filtered header's button reads "Age is
  filtered". Each header filter field is named "Filter" plus its column.
- A column filter change announces the rows left, "2 rows", in the same polite
  live region, once it settles.
- A group header is a `th scope="colgroup"` over its columns, so a screen reader
  reads it with each of their cells. A column outside any group, and the
  select-all box, span every header row.
- Pinned columns move to their edge in the DOM too, so Tab and a screen reader
  meet the cells in the order they are seen. The detail toggle and checkbox
  columns pin with the start ones.
- A pinned column past one without a `width` warns in a debug build: its offset
  is unknown, so it would overlap.
- A group or a `col_span` stops at a pin edge: a group over pinned and scrolled
  columns shows as two headers, and a group header never pins.
- With `onrowreorder`, each row has a drag handle named "Reorder" plus the
  row's name, described by the keyboard steps, and Move up and Move down
  buttons, so a single pointer reorders without a drag (WCAG 2.5.7). A touch
  drags only from the handle; elsewhere it scrolls. Each lift, move and drop is
  said in a polite live region. While sorted or filtered the controls are
  disabled.
- `onrowreorder` without `row_key` warns in a debug build.
- The column menu's Move left and Move right name the screen sides in either
  text direction. A moved column moves in the DOM too, so Tab and a screen
  reader follow it, and a column moved out of its group splits the group.
- `loading` without shown rows marks the table `aria-busy` and hides its
  placeholder rows from screen readers. With rows shown it adds a progress bar
  named "Loading rows" and leaves the table unbusy, as some screen readers hold
  back a busy table's rows.
- The `toolbar` is a plain row, not a `role="toolbar"`: Tab moves through its
  controls as anywhere else.
- With `virtual_row_height`, the table carries `aria-rowcount`, every row it
  holds, and each rendered row its `aria-rowindex`, so a screen reader says
  "row 5 001 of 10 001" though only a screenful is in the DOM. The
  scrolled-away rows leave no empty rows behind.
- With `virtual_row_height`, Tab and Shift+Tab walk the rows' controls past the
  rendered ones: the focused row scrolls into view and the next one renders.
  The row holding focus stays rendered when it scrolls away, Blitz included.

### You must

- Name every table. `caption` shows a title and names it, `aria_labelledby`
  points at a heading already on the page, and `aria_label` names it without
  text.
- Set `scroll: true` on a table wider than its container, or `max_height` on a
  long one.
- Mark the column that names a row with `.row_header()`.
- With `onrowclick`, also put a button or link for that action in a cell. A
  row is not a tab stop, so a keyboard cannot click it.
- With `selectable` or `row_detail`, give the rows a `.row_header()` column, so
  each checkbox and toggle is named by something unique.
- Before a CSV of text users typed goes to a spreadsheet, neutralise cells that
  start with `=`, `+`, `-` or `@` (CSV injection, OWASP). `table_text`'s docs
  show a three-line guard.

### Limits

- On Blitz, once a `max_height` table's rows scroll, a click on a header's sort
  or menu button misses: Blitz hit-tests the header where it sat before the
  scroll. Tab to the button and press Enter instead.
- On Blitz, the same holds for a pinned column's cells once the table scrolls
  sideways. In a right-to-left page Blitz cannot scroll a wide table at all
  (todo 707), so pinning shows no effect there.
- On Blitz, a `max_height` table with column groups keeps only its last header
  row in place; the group rows scroll away with the rows. The same holds for
  the `header_filters` row.
- On Blitz, a row's handle does not drag: Blitz paints no moved table row. The
  keyboard and the move buttons reorder there.
- Columns reorder from the column menu only, not by dragging a header.
- On Android, the filter popover cannot look into itself, so Tab does not close
  it at its ends and focus lands on the popover rather than its value field;
  Escape and Back close it.

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
| `data-empty` | On the body row that holds `empty`, `no_results` or their text. |
| `data-skeleton` | On each placeholder row of a `loading` table without shown rows. |
| `data-loading-bar` | On the box holding the progress bar of a `loading` table with shown rows. |
| `data-toolbar` | On the row above the table with `toolbar`; `data-toolbar-end` on the quick filter's box in it. |
| `data-detail-toggle` | On the toggle column's `th` and `td`s, with `row_detail`. |
| `data-detail` | On an open detail row, right after its row. |
| `data-stripe` | On every other body row with `striped`, detail rows not counted. |
| `data-reorder` | On the reorder column's `th` and `td`s, with `onrowreorder`. |
| `data-dragging` | On the body row being dragged or moved by keyboard. |
| `data-group` | On a column group's header cell. |
| `data-pin` | On a pinned column's `th` and `td`s, `start` or `end`. |
| `data-pin-edge` | On the pinned cells next to the scrolled columns, which draw a rule there. |
| `data-state` | On the table: `size-{size}`, plus `striped` and `row-click` when set, `pinned` with a pinned column and `pin-select` when the checkbox column pins too, `pin-detail` when the detail toggle column does, `pin-reorder` when the reorder column does. On a body row: its active `row_states`. |
| `data-slot="range"` | On the paginated table's range text, "1–10 of 95". |
