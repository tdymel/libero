//! `Table` with a sortable text column and a custom-rendered one; a wide one
//! in its scroll region under a caption; keyed, clickable, striped rows; selectable,
//! multi-sorted rows; an empty one; a paged one; sized columns; column menus; a
//! height-capped one with a sticky header; a wide one with pinned columns, also RTL;
//! column filters through menus and header filters; a windowed one of 10k rows;
//! one that loads more rows at its bottom.

use std::time::Duration;

use dioxus::prelude::*;
use libero::chrono::NaiveDate;
use libero::components::{
    ColumnDefaults, ColumnFilter, PinnedColumns, SortDirection, States, Table, TableSort, column,
};
use libero::platform::{TimerSubscription, timer};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table", || rsx! { TablePage {} }),
    ("/table/wide", || rsx! { WideTablePage {} }),
    ("/table/rows", || rsx! { RowsTablePage {} }),
    ("/table/empty", || rsx! { EmptyTablePage {} }),
    ("/table/select", || rsx! { SelectTablePage {} }),
    ("/table/paged", || rsx! { PagedTablePage {} }),
    ("/table/widths", || rsx! { WidthsTablePage {} }),
    ("/table/menu", || rsx! { MenuTablePage {} }),
    (
        "/table/sticky",
        || rsx! { StickyTablePage { interactive: true } },
    ),
    (
        "/table/sticky-plain",
        || rsx! { StickyTablePage { interactive: false } },
    ),
    ("/table/filter", || rsx! { FilterTablePage {} }),
    ("/table/column-filter", || rsx! { ColumnFilterTablePage {} }),
    ("/table/date-filter", || rsx! { DateFilterTablePage {} }),
    ("/table/pinned", || rsx! { PinnedTablePage { rtl: false } }),
    (
        "/table/pinned-rtl",
        || rsx! { PinnedTablePage { rtl: true } },
    ),
    (
        "/table/windowed",
        || rsx! { WindowedTablePage { pinned: false } },
    ),
    (
        "/table/windowed-pinned",
        || rsx! { WindowedTablePage { pinned: true } },
    ),
    ("/table/infinite", || rsx! { InfiniteTablePage {} }),
];

/// The seven paged fruits with a quick-filter field; `#query` and `#page` echo
/// their change handlers.
#[component]
fn FilterTablePage() -> Element {
    let mut query = use_signal(String::new);
    let mut page = use_signal(|| 0u32);
    let data: Vec<Fruit> = ["Fig", "Apple", "Grape", "Cherry", "Elder", "Banana", "Date"]
        .into_iter()
        .zip(1..)
        .map(|(name, stock)| Fruit { name, stock })
        .collect();
    rsx! {
        Table {
            caption: "Fruit",
            data,
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock).filterable(false),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            default_page_size: 3usize,
            show_quick_filter: true,
            onquickfilterchange: move |next| query.set(next),
            onpagechange: move |next| page.set(next),
        }
        p { id: "query", "{query}" }
        p { id: "page", "{page}" }
    }
}

/// The seven fruits with column menus and header filters; `#filters` echoes
/// `oncolumnfilterschange`. Unsorted, so a menu's first item is Filter.
#[component]
fn ColumnFilterTablePage() -> Element {
    let mut filters = use_signal(String::new);
    let data: Vec<Fruit> = ["Fig", "Apple", "Grape", "Cherry", "Elder", "Banana", "Date"]
        .into_iter()
        .zip(1..)
        .map(|(name, stock)| Fruit { name, stock })
        .collect();
    rsx! {
        Table {
            caption: "Fruit",
            data,
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            column_menu: true,
            header_filters: true,
            oncolumnfilterschange: move |next: Vec<ColumnFilter>| {
                let text: Vec<String> = next
                    .iter()
                    .map(|filter| format!("{} {:?} {}", filter.column, filter.operator, filter.value))
                    .collect();
                filters.set(text.join("; "));
            },
        }
        p { id: "filters", "{filters}" }
    }
}

/// Five deliveries due on the 1st, 5th, 10th, 15th and 20th of March 2024, with
/// a date column's menu and header filter (1401); `#filters` echoes the change.
#[component]
fn DateFilterTablePage() -> Element {
    #[derive(Clone, PartialEq)]
    struct Delivery {
        name: &'static str,
        due: NaiveDate,
    }
    let mut filters = use_signal(String::new);
    let data: Vec<Delivery> = ["Fig", "Apple", "Grape", "Cherry", "Elder"]
        .into_iter()
        .zip([1, 5, 10, 15, 20])
        .filter_map(|(name, day)| {
            Some(Delivery {
                name,
                due: NaiveDate::from_ymd_opt(2024, 3, day)?,
            })
        })
        .collect();
    rsx! {
        Table {
            caption: "Deliveries",
            data,
            columns: vec![
                column("Name").value(|row: &Delivery| row.name.to_string()).row_header(),
                column("Due").value(|row: &Delivery| row.due),
            ],
            row_key: |row: &Delivery| row.name.to_string(),
            column_menu: true,
            header_filters: true,
            oncolumnfilterschange: move |next: Vec<ColumnFilter>| {
                let text: Vec<String> = next
                    .iter()
                    .map(|filter| format!("{:?} {}..{}", filter.operator, filter.value, filter.value_to))
                    .collect();
                filters.set(text.join("; "));
            },
        }
        p { id: "filters", "{filters}" }
    }
}

/// Column menus over a multi-sorted table; the name column can't be hidden.
#[component]
fn MenuTablePage() -> Element {
    rsx! {
        Table {
            aria_label: "Fruit",
            column_menu: true,
            multi_sort: true,
            data: fruit(),
            columns: vec![
                column("Name")
                    .value(|fruit: &Fruit| fruit.name.to_string())
                    .sortable()
                    .row_header()
                    .hideable(false),
                column("Stock").value(|fruit: &Fruit| fruit.stock).sortable(),
            ],
        }
    }
}

/// A 200px first column; a sortable one with a rendered header.
#[component]
fn WidthsTablePage() -> Element {
    rsx! {
        div { width: "600px",
            Table {
                aria_label: "Fruit",
                column_defaults: ColumnDefaults::new().min_width("4rem"),
                data: fruit(),
                columns: vec![
                    column("Name").value(|fruit: &Fruit| fruit.name.to_string()).width("200px"),
                    column("Stock")
                        .value(|fruit: &Fruit| fruit.stock)
                        .sortable()
                        .header_render(|| rsx! { "Stock " small { "(boxes)" } }),
                ],
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct Fruit {
    name: &'static str,
    stock: u32,
}

fn fruit() -> Vec<Fruit> {
    vec![
        Fruit {
            name: "Cherry",
            stock: 3,
        },
        Fruit {
            name: "Apple",
            stock: 12,
        },
        Fruit {
            name: "Banana",
            stock: 0,
        },
    ]
}

#[component]
pub fn TablePage() -> Element {
    rsx! {
        Table {
            aria_label: "Fruit",
            data: fruit(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock")
                    .value(|fruit: &Fruit| fruit.stock)
                    .render(|fruit: &Fruit| rsx! { b { "{fruit.stock} left" } }),
            ],
        }
    }
}

/// Eight sortable columns of unbreakable text: wider than the desktop viewport.
#[component]
fn WideTablePage() -> Element {
    let headers = [
        "Name", "Origin", "Season", "Colour", "Taste", "Storage", "Price", "Supplier",
    ];
    let columns = headers
        .into_iter()
        .map(|header| {
            column(header)
                .value(move |fruit: &Fruit| format!("{}_{header}_description", fruit.name))
                .sortable()
        })
        .collect();
    rsx! {
        Table { caption: "Fruit catalogue", scroll: true, data: fruit(), columns }
    }
}

/// Keyed, clickable, striped rows at `sm`; the sold-out row marked by a state.
/// `#prepend` adds a row on top, which the keys keep off the others' nodes.
#[component]
fn RowsTablePage() -> Element {
    let mut clicked = use_signal(String::new);
    let mut data = use_signal(fruit);
    rsx! {
        button {
            id: "prepend",
            onclick: move |_| data.write().insert(0, Fruit { name: "Apricot", stock: 7 }),
            "Prepend"
        }
        Table {
            aria_label: "Fruit",
            size: "sm",
            striped: true,
            data: data(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock")
                    .value(|fruit: &Fruit| fruit.stock)
                    .format(|fruit: &Fruit| format!("{} kg", fruit.stock))
                    .sortable(),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            row_states: |fruit: &Fruit| States::new().with("sold-out", fruit.stock == 0),
            row_attrs: |fruit: &Fruit| vec![Attribute::new("data-name", fruit.name, None, false)],
            onrowclick: move |fruit: Fruit| clicked.set(fruit.name.to_string()),
            sx: sx().selector("& tbody tr", sx().when("sold-out", sx().color("red"))),
        }
        p { id: "clicked", "{clicked}" }
    }
}

/// Selectable, multi-sortable rows with stock ties; the selection, sort and last
/// row click printed below.
#[component]
fn SelectTablePage() -> Element {
    let mut selection = use_signal(|| vec!["Apple".to_string()]);
    let mut sort = use_signal(Vec::<TableSort>::new);
    let mut clicked = use_signal(String::new);
    let data = || {
        let mut data = fruit();
        data.push(Fruit {
            name: "Date",
            stock: 3,
        });
        data.push(Fruit {
            name: "Elder",
            stock: 12,
        });
        data
    };
    let sorted: Vec<String> = sort
        .read()
        .iter()
        .map(|entry| {
            let direction = match entry.direction {
                SortDirection::Ascending => "ascending",
                SortDirection::Descending => "descending",
            };
            format!("{} {direction}", entry.column)
        })
        .collect();
    rsx! {
        Table {
            aria_label: "Fruit",
            selectable: true,
            selection: selection(),
            onselectionchange: move |next| selection.set(next),
            multi_sort: true,
            sort: sort(),
            onsortchange: move |next| sort.set(next),
            data: data(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock).sortable(),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            onrowclick: move |fruit: Fruit| clicked.set(fruit.name.to_string()),
        }
        p { id: "selection", {selection.read().join(",")} }
        p { id: "sort", {sorted.join(",")} }
        p { id: "clicked", "{clicked}" }
    }
}

/// Seven fruits, three a page, with a page-size picker; `#page` echoes `onpagechange`.
#[component]
fn PagedTablePage() -> Element {
    let mut page = use_signal(|| 0u32);
    let data: Vec<Fruit> = ["Fig", "Apple", "Grape", "Cherry", "Elder", "Banana", "Date"]
        .into_iter()
        .zip(1..)
        .map(|(name, stock)| Fruit { name, stock })
        .collect();
    rsx! {
        Table {
            caption: "Fruit",
            data,
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            page_sizes: vec![3, 5],
            onpagechange: move |next| page.set(next),
        }
        p { id: "page", "{page}" }
    }
}

/// Forty fruits under a 200px `max_height`; sortable with column menus, or plain
/// text, which leaves the scroll area the only tab stop.
#[component]
fn StickyTablePage(interactive: bool) -> Element {
    let data: Vec<Fruit> = (1..=40)
        .map(|stock| Fruit {
            name: ["Apple", "Banana", "Cherry", "Date"][stock as usize % 4],
            stock,
        })
        .collect();
    let name = column("Name").value(|fruit: &Fruit| fruit.name.to_string());
    let stock = column("Stock").value(|fruit: &Fruit| fruit.stock);
    let columns = match interactive {
        true => vec![name.sortable(), stock.sortable()],
        false => vec![name, stock],
    };
    rsx! {
        Table {
            caption: "Fruit stock",
            max_height: "200px",
            column_menu: interactive,
            striped: true,
            data,
            columns,
        }
    }
}

/// The wide table, selectable with column menus: Name pinned to the start,
/// Supplier to the end; `#pinned` echoes `onpinnedcolumnschange`. `rtl` sets
/// `dir="rtl"` around it.
#[component]
fn PinnedTablePage(rtl: bool) -> Element {
    let mut pinned = use_signal(String::new);
    let headers = [
        "Name", "Origin", "Season", "Colour", "Taste", "Storage", "Price", "Supplier",
    ];
    let columns = headers
        .into_iter()
        .map(|header| {
            column(header)
                .value(move |fruit: &Fruit| format!("{}_{header}_description", fruit.name))
                .width("240px")
        })
        .collect();
    rsx! {
        div { dir: if rtl { "rtl" } else { "ltr" },
            Table {
                caption: "Fruit catalogue",
                scroll: true,
                selectable: true,
                column_menu: true,
                striped: true,
                row_key: |fruit: &Fruit| fruit.name.to_string(),
                default_pinned_columns: PinnedColumns::default().start(["Name"]).end(["Supplier"]),
                onpinnedcolumnschange: move |next: PinnedColumns| {
                    pinned.set(format!("{}|{}", next.start.join(","), next.end.join(",")))
                },
                data: fruit(),
                columns,
            }
            p { id: "pinned", "{pinned}" }
        }
    }
}

/// Ten thousand selectable rows, 40px each under a 300px `max_height`: only
/// the rows in view render. `pinned` adds six 240px columns, Name pinned to the
/// start and Supplier to the end.
#[component]
fn WindowedTablePage(pinned: bool) -> Element {
    let mut selection = use_signal(Vec::<String>::new);
    // Longer names at the end: auto layout would widen the column there.
    let data: Vec<Fruit> = (1..=10_000)
        .map(|stock| Fruit {
            name: match stock > 9_950 {
                true => "Elderberry from the far north",
                false => ["Apple", "Banana", "Cherry", "Date"][stock as usize % 4],
            },
            stock,
        })
        .collect();
    let mut columns = vec![
        column("Name")
            .value(|fruit: &Fruit| fruit.name.to_string())
            .sortable(),
        column("Stock")
            .value(|fruit: &Fruit| fruit.stock)
            .sortable()
            .row_header(),
    ];
    if pinned {
        columns[0] = columns[0].clone().width("240px");
        columns.extend(
            ["Origin", "Season", "Colour", "Taste", "Storage", "Supplier"].map(|header| {
                column(header)
                    .value(move |fruit: &Fruit| format!("{}_{header}", fruit.name))
                    .width("240px")
            }),
        );
    }
    rsx! {
        Table {
            caption: "Fruit stock",
            max_height: "300px",
            virtual_row_height: 40.0,
            selectable: true,
            selection: selection(),
            onselectionchange: move |next| selection.set(next),
            default_pinned_columns: match pinned {
                true => PinnedColumns::default().start(["Name"]).end(["Supplier"]),
                false => PinnedColumns::default(),
            },
            data,
            columns,
            row_key: |fruit: &Fruit| fruit.stock.to_string(),
        }
        p { id: "selection", {selection.read().join(",")} }
    }
}

/// Fifty windowed rows a batch, the next one 300 ms after the bottom is reached,
/// three batches in all. `#asks` counts the calls.
#[component]
fn InfiniteTablePage() -> Element {
    let batch = |from: u32| -> Vec<Fruit> {
        (from + 1..=from + 50)
            .map(|stock| Fruit {
                name: ["Apple", "Banana", "Cherry", "Date"][stock as usize % 4],
                stock,
            })
            .collect()
    };
    let mut rows = use_signal(|| batch(0));
    let mut loading = use_signal(|| false);
    let mut asks = use_signal(|| 0u32);
    let mut pending = use_signal(|| None::<Box<dyn TimerSubscription>>);
    use_drop(move || pending.set(None));
    rsx! {
        Table {
            caption: "Fruit stock",
            max_height: "300px",
            virtual_row_height: 40.0,
            loading: loading(),
            onbottomreached: move |_| {
                asks += 1;
                let from = rows.peek().len() as u32;
                if from >= 150 {
                    return;
                }
                loading.set(true);
                pending.set(timer().map(|timer| {
                    timer.after(
                        Duration::from_millis(300),
                        Box::new(move || {
                            rows.write().extend(batch(from));
                            loading.set(false);
                        }),
                    )
                }));
            },
            data: rows(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()),
                column("Stock").value(|fruit: &Fruit| fruit.stock).row_header(),
            ],
            row_key: |fruit: &Fruit| fruit.stock.to_string(),
        }
        p { id: "asks", "{asks}" }
    }
}

#[component]
fn EmptyTablePage() -> Element {
    rsx! {
        Table {
            aria_label: "Fruit",
            empty: rsx! { "No fruit" },
            data: Vec::<Fruit>::new(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()),
                column("Stock").value(|fruit: &Fruit| fruit.stock),
            ],
        }
    }
}
