//! `Table` with master-detail rows: sortable, striped, clickable; the open
//! details and the last row click printed below.

use dioxus::prelude::*;
use libero::components::{Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[("/table-detail", || rsx! { DetailTablePage {} })];

#[derive(Clone, PartialEq)]
struct Fruit {
    name: &'static str,
    stock: u32,
    origin: Option<&'static str>,
}

fn fruit() -> Vec<Fruit> {
    vec![
        Fruit {
            name: "Cherry",
            stock: 3,
            origin: Some("Turkey"),
        },
        Fruit {
            name: "Apple",
            stock: 12,
            origin: Some("Kazakhstan"),
        },
        Fruit {
            name: "Banana",
            stock: 0,
            origin: None,
        },
    ]
}

/// Banana has no detail, so no toggle.
#[component]
fn DetailTablePage() -> Element {
    let mut expanded = use_signal(Vec::<String>::new);
    let mut clicked = use_signal(String::new);
    rsx! {
        Table {
            aria_label: "Fruit",
            striped: true,
            data: fruit(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock).sortable(),
            ],
            row_key: |fruit: &Fruit| fruit.name.to_string(),
            row_detail: |fruit: &Fruit| {
                let name = fruit.name;
                fruit.origin.map(|origin| rsx! {
                    p { "From {origin}" }
                    button { "data-order": name, "Order {name}" }
                })
            },
            expanded: expanded(),
            onexpandedchange: move |next| expanded.set(next),
            onrowclick: move |fruit: Fruit| clicked.set(fruit.name.to_string()),
        }
        p { id: "expanded", {expanded.read().join(",")} }
        p { id: "clicked", "{clicked}" }
    }
}
