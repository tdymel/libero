//! `Table` with a sortable text column and a custom-rendered one; a wide one
//! in its scroll region under a caption; keyed, clickable, striped rows; an empty one.

use dioxus::prelude::*;
use libero::components::{States, Table, column};
use libero::sx::sx;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table", || rsx! { TablePage {} }),
    ("/table/wide", || rsx! { WideTablePage {} }),
    ("/table/rows", || rsx! { RowsTablePage {} }),
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
