# Table

Crate: `libero`
Import: `use libero::components::{Table, column};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/table>
Index: [index.md](index.md) - every other component's markdown page
Description: A sortable data table built from a row type and a list of column definitions.

Data in, table out. Each column is built with `column(..)` - a header, a `value`
that reads one cell out of a row, and whatever else that column needs. `Table`
itself needs only `data` and `columns`.

The cell's type does the quiet work: `value` reads it for the column's sort order
and alignment, then erases it, which is why columns over different cell types
live in one `Vec`. A numeric column sorts numerically and aligns right without
being told.

`sortable` turns a header into a button - the first click sorts ascending, the
next flips it, and the sort state stays inside `Table`. `render` changes only
what a cell draws, so a Role column rendered as a [Chip](chip.md) still sorts by
its text and not by its chip.

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
                column("Bonus").value(|p: &Person| p.bonus).sortable(),
            ],
        }
    }
}
```

Strings sort as text and align left. Every integer and float sorts numerically
and aligns right. `bool` prints `true`/`false`. `Option<V>` keeps the inner
type's alignment, renders `None` as empty, and sorts it last in both directions.

```rust,ignore
|p: &Person| p.name.clone()  // String      -> text sort, start-aligned
|p: &Person| p.name.len()   // usize       -> numeric sort, end-aligned
|p: &Person| p.bonus        // Option<f64> -> None renders empty, sorts last
```

Your own type joins them with one `impl CellValue`: `cell_text` is required,
`sort_key` and `align` have defaults.

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

    // An associated function, not a method: alignment is a property of the
    // type, not of one cell.
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

`align` on the column overrides whatever the cell type chose, for the case where
one column should not follow its type.

A table wider than its container takes `scroll: true`, and `empty` fills the
body while `data` has no rows:

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

Name every table: `caption` shows a title and names it, `aria_labelledby`
points at a heading already on the page, or `aria_label` names it without text.
An unnamed table warns in the console in debug builds.

A table wider than its container needs `scroll: true`: the wrapper is a
`role="region"` named like the table and a tab stop, so a keyboard user can
scroll it with the arrow keys. A sortable header is a button; Enter or Space
sorts. Only the sorted header carries `aria-sort`.

Mark the column that names a row with `.row_header()`: its cells render as
`th scope="row"`, so a screen reader reads that name as it moves down any other
column. It looks like the other cells; the change is semantic only.

## Props

### Table

| Prop | Type | Default | Description |
|---|---|---|---|
| `data` | `Vec<T>` | required | One row each, in source order until a column is sorted. |
| `columns` | `Vec<Column<T>>` | required | Built with `column(..)`. |
| `caption` | `Option<String>` | - | A visible title above the header row, and the table's accessible name. |
| `empty` | `Option<Element>` | - | Shown in one full-width row when `data` is empty. |
| `scroll` | `bool` | `false` | Wraps the table in a named, focusable `role="region"` that scrolls sideways. `class`, `sx` and `attributes` stay on the table. |

Like every component, `Table` also takes the shared props `sx`, `class`,
`style`, `states`, and any extra HTML attributes - `aria_label` among them.

### column()

| Prop | Type | Default | Description |
|---|---|---|---|
| `header` | `String` | required | The column's title, given as the argument to `column(..)`. |
| `value` | `fn(&T) -> V` | required | Reads one cell out of a row. `V`'s `CellValue` impl decides sort order and alignment, then is erased. |
| `sortable` | `bool` | `false` | Turns the header into a sort button. |
| `render` | `fn(&T) -> Element` | - | Replaces the cell body. Sorting still uses `value`. |
| `align` | `CellAlign` | follows the cell type | Overrides the alignment `value`'s type chose. |
| `row_header` | `bool` | `false` | Renders the column's cells as `th scope="row"`, so a screen reader names each row by it. One per table, usually the first. |

`column()` is a builder, not a component - it takes no shared props.

## Theme defaults

`TableDefaults` on the theme. Flat values, not a `Sizes` scale: `Table` has no
`size` prop, so a per-size scale would be six numbers expressing one.

| Field | Type | Description |
|---|---|---|
| `padding_x` | `u8` | Horizontal cell padding, in px. |
| `padding_y` | `u8` | Vertical cell padding, in px. |
| `font_size` | `u16` | Table font size, in px. |
| `border_color` | `ColorValue` | Color of the header and row rules. |
| `hover_color` | `ColorValue` | Row background while hovered. |

## CSS variables

| Variable | Description |
|---|---|
| `--lsx-table-padding-x` | Horizontal padding on every `th`/`td`. |
| `--lsx-table-padding-y` | Vertical padding on every `th`/`td`. |
| `--lsx-table-font-size` | `font-size` of the table. |
| `--lsx-table-border-color` | Color of the `1px` header and row rules. |
| `--lsx-table-hover` | `background` of a hovered body row. |

## Data attributes

`data-align` on a `th`/`td` carries the cell alignment, and is omitted for the
default `start`.

| Attribute | Condition |
|---|---|
| `data-align="center"` | The column's alignment is `center`. |
| `data-align="end"` | The column's alignment is `end` - what every numeric cell type picks. |
| `data-sortable` | On a sortable header. |
| `aria-sort` | On the sorted header only: `ascending` or `descending`. |
| `data-empty` | On the body row that holds `empty`. |
