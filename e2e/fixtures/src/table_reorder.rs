//! `Table` row reorder and column order (1156-4b, 4c): the rows' and the
//! columns' order printed below.

use dioxus::prelude::*;
use libero::{
    components::{Table, column},
    hooks::SortableMove,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-reorder", || rsx! { ReorderTablePage {} }),
    ("/table-reorder-detail", || rsx! { ReorderDetailPage {} }),
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
