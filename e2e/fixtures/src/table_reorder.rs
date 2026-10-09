//! `Table` row reorder and column order (1156-4b, 4c): the rows' and the
//! columns' order printed below.

use dioxus::prelude::*;
use libero::{
    components::{Aggregate, Chip, Table, column},
    hooks::{SortableMove, use_localization_handle},
    localization::Localization,
    sx::sx,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-reorder", || rsx! { ReorderTablePage {} }),
    ("/table-reorder-de", || rsx! { ReorderGermanPage {} }),
    ("/table-reorder-detail", || rsx! { ReorderDetailPage {} }),
    ("/table-reorder-windowed", || rsx! { ReorderWindowedPage {} }),
    ("/table-reorder-windowed-footer", || rsx! { ReorderWindowedFooterPage {} }),
    ("/table-column-drag", || rsx! { ColumnDragPage {} }),
    ("/table-column-scroll", || rsx! { ColumnScrollPage {} }),
];

#[derive(Clone, PartialEq)]
struct Fruit {
    name: &'static str,
    stock: u32,
    origin: &'static str,
}

fn fruit() -> Vec<Fruit> {
    ["Cherry", "Apple", "Banana", "Damson"]
        .into_iter()
        .zip([3, 12, 0, 7])
        .zip(["Turkey", "Kazakhstan", "India", "Syria"])
        .map(|((name, stock), origin)| Fruit {
            name,
            stock,
            origin,
        })
        .collect()
}

/// The same table in German, for the column menu's announcements (todo 2049).
#[component]
fn ReorderGermanPage() -> Element {
    let localization = use_localization_handle();
    use_effect(move || localization.set(&Localization::GERMAN));
    rsx! { ReorderTablePage {} }
}

#[component]
fn ReorderTablePage() -> Element {
    let mut rows = use_signal(fruit);
    let mut columns = use_signal(Vec::<String>::new);
    let mut moves = use_signal(String::new);
    let order: Vec<&str> = rows.read().iter().map(|fruit| fruit.name).collect();
    rsx! {
        Table {
            aria_label: "Fruit",
            data: rows(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock).sortable(),
                column("Origin").value(|fruit: &Fruit| fruit.origin.to_string()),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            onrowreorder: move |step: SortableMove| {
                moves.write().push_str(&format!("{}>{} ", step.from, step.to));
                step.apply(&mut rows.write());
            },
            column_menu: true,
            column_order: columns(),
            oncolumnorderchange: move |next| columns.set(next),
        }
        p { id: "order", {order.join(" ")} }
        p { id: "moves", "{moves}" }
        p { id: "columns", {columns.read().join(" ")} }
    }
}

/// Todo 1408: 200 windowed rows, `Row 1` to `Row 200`; `#moves` lists each reorder,
/// `#ends` the first and last rows.
#[component]
fn ReorderWindowedPage() -> Element {
    rsx! { WindowedRows {} }
}

/// The windowed rows under a sticky footer that counts them (todo 2771).
#[component]
fn ReorderWindowedFooterPage() -> Element {
    rsx! { WindowedRows { footer: true } }
}

#[component]
fn WindowedRows(#[props(default)] footer: bool) -> Element {
    let mut rows = use_signal(|| (1..=200).map(|n| format!("Row {n}")).collect::<Vec<_>>());
    let mut moves = use_signal(String::new);
    let ends = {
        let rows = rows.read();
        format!("{} {}", rows[0], rows[rows.len() - 1])
    };
    let name = column("Name").value(|row: &String| row.clone()).row_header();
    rsx! {
        Table {
            aria_label: "Rows",
            max_height: "240px",
            virtual_row_height: 40.0,
            data: rows(),
            columns: vec![if footer { name.aggregate(Aggregate::Count) } else { name }],
            row_key: |row: &String| row.clone(),
            onrowreorder: move |step: SortableMove| {
                moves.write().push_str(&format!("{}>{} ", step.from, step.to));
                step.apply(&mut rows.write());
            },
        }
        p { id: "moves", "{moves}" }
        p { id: "ends", "{ends}" }
    }
}

/// The docs demo's shape: the table holds the order itself, a caption, a rendered column.
#[component]
fn ColumnDragPage() -> Element {
    let mut columns = use_signal(String::new);
    rsx! {
        Table {
            caption: "Fruit",
            data: fruit(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock")
                    .value(|fruit: &Fruit| fruit.stock)
                    .sortable()
                    .render(|fruit: &Fruit| rsx! { Chip { size: "xs", "{fruit.stock}" } }),
                column("Origin").value(|fruit: &Fruit| fruit.origin.to_string()).sortable(),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            column_menu: true,
            oncolumnorderchange: move |next: Vec<String>| columns.set(next.join(" ")),
        }
        p { id: "columns", "{columns}" }
    }
}

/// A table wider than its 320px scroll region: Origin's end starts out of view.
#[component]
fn ColumnScrollPage() -> Element {
    let mut columns = use_signal(String::new);
    rsx! {
        div { style: "width: 320px",
            Table {
                aria_label: "Fruit",
                data: fruit(),
                columns: vec![
                    column("Name").value(|fruit: &Fruit| fruit.name.to_string()).row_header(),
                    column("Stock").value(|fruit: &Fruit| fruit.stock),
                    column("Origin").value(|fruit: &Fruit| fruit.origin.to_string()),
                ],
                row_key: |fruit: &Fruit| fruit.name.to_string(),
                column_menu: true,
                scroll: true,
                sx: sx().min_width("640px"),
                oncolumnorderchange: move |next: Vec<String>| columns.set(next.join(" ")),
            }
        }
        p { id: "columns", "{columns}" }
    }
}

/// Cherry's detail open, a tall one: it moves with its row (1397).
#[component]
fn ReorderDetailPage() -> Element {
    let mut rows = use_signal(fruit);
    let order: Vec<&str> = rows.read().iter().map(|fruit| fruit.name).collect();
    rsx! {
        Table {
            aria_label: "Fruit",
            data: rows(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            row_detail: |fruit: &Fruit| {
                let origin = fruit.origin;
                Some(rsx! {
                    p { style: "height: 80px; margin: 0", "From {origin}" }
                })
            },
            default_expanded: vec!["Cherry".to_string()],
            onrowreorder: move |step: SortableMove| step.apply(&mut rows.write()),
        }
        p { id: "order", {order.join(" ")} }
    }
}
