//! `Table` with a sortable text column and a custom-rendered one; a wide one
//! in its scroll region under a caption; an empty one.

use dioxus::prelude::*;
use libero::components::{Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table", || rsx! { TablePage {} }),
    ("/table/wide", || rsx! { WideTablePage {} }),
    ("/table/empty", || rsx! { EmptyTablePage {} }),
];

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
