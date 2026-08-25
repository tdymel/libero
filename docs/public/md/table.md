# Table

Crate: `libero`
Import: `use libero::components::{Table, column};`
Source: <https://github.com/tdymel/libero/tree/main/libero/src/components/data_display/table>
Index: [index.md](index.md) - every other component's markdown page
Description: A sortable data table built from a row type and a list of column definitions.

Data in, table out. Each column is built with `column(..)` - a header, a `value`
that reads one cell out of a row, and whatever else that column needs. `Table`
itself takes only `data` and `columns`.

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
    age: u32,
    bonus: Option<f64>,
}

#[component]
fn Demo() -> Element {
    let people = vec![
        Person { name: "Ada Lovelace".into(), role: "Owner".into(), age: 36, bonus: Some(12.5) },
        Person { name: "Grace Hopper".into(), role: "Admin".into(), age: 45, bonus: Some(8.0) },
        Person { name: "Alan Turing".into(), role: "Viewer".into(), age: 9, bonus: None },
    ];

    rsx! {
        Table {
            aria_label: "Team members",
            data: people,
            columns: vec![
                column("Name").value(|p: &Person| p.name.clone()).sortable(),
                column("Role")
                    .value(|p: &Person| p.role.clone())
                    .render(|p: &Person| rsx! { Chip { size: "xs", "{p.role}" } }),
                column("Age").value(|p: &Person| p.age).sortable(),
                column("Bonus").value(|p: &Person| p.bonus).sortable(),
            ],
        }
    }
}
```

## The cell type decides

Strings sort as text and align left. Every integer and float sorts numerically
and aligns right. `bool` prints `true`/`false`. `Option<V>` keeps the inner
type's alignment, renders `None` as empty, and sorts it last in both directions.

```rust
|p: &Person| p.name.clone()  // String      -> text sort, start-aligned
|p: &Person| p.age          // u32         -> numeric sort, end-aligned
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

## Accessibility

The root is a real `<table>` with a `<thead>`/`<tbody>` and `scope="col"` on
every header cell, so a screen reader announces column headers with each cell.
Give it an accessible name with `aria_label` when the surrounding text doesn't
already provide one - it passes straight through to the `<table>`.

A sortable header is a `<button>` inside its `th`, so it is reachable by Tab and
activated with Space or Enter. Only sortable headers carry `aria-sort`
(`none`/`ascending`/`descending`), which is both the announced sort state and
what fades and flips the arrow. The arrow itself is `aria-hidden`.

## Props

### Table

| Prop | Type | Default | Description |
|---|---|---|---|
| `data` | `Vec<T>` | required | One row each, in source order until a column is sorted. |
| `columns` | `Vec<Column<T>>` | required | Built with `column(..)`. |

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
| `aria-sort` | On sortable headers only: `none`, `ascending`, or `descending`. |
