//! `Table` with a sortable text column and a custom-rendered one; a wide one
//! in its scroll region under a caption; keyed, clickable, striped rows; selectable,
//! multi-sorted rows; an empty one; a paged one; sized columns; column menus.

use dioxus::prelude::*;
use libero::components::{ColumnDefaults, SortDirection, States, Table, TableSort, column};
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
];

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
