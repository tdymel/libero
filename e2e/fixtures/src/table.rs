//! `Table` with a sortable text column and a custom-rendered one.

use dioxus::prelude::*;
use libero::components::{Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[("/table", || rsx! { TablePage {} })];

#[derive(Clone, PartialEq)]
struct Fruit {
    name: &'static str,
    stock: u32,
}

#[component]
pub fn TablePage() -> Element {
    let data = vec![
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
    ];
    rsx! {
        Table {
            aria_label: "Fruit",
            data,
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable(),
                column("Stock")
                    .value(|fruit: &Fruit| fruit.stock)
                    .render(|fruit: &Fruit| rsx! { b { "{fruit.stock} left" } }),
            ],
        }
    }
}
