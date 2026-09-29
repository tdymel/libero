//! `Table` toolbar pieces (1416, 1448): Columns, Density, Filters and Export over
//! a paged, sorted table; the exported CSV and the picked density printed below.

use dioxus::prelude::*;
use libero::components::{
    SortDirection, Table, TableColumnsButton, TableDensityButton, TableExportButton,
    TableFilterButton, TableSort, column,
};

use crate::Routes;

pub const ROUTES: Routes = &[("/table-toolbar", || rsx! { ToolbarTablePage {} })];

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

/// Two rows a page, so an export over every page shows.
#[component]
fn ToolbarTablePage() -> Element {
    let mut csv = use_signal(String::new);
    let mut density = use_signal(String::new);
    rsx! {
        Table {
            aria_label: "Fruit",
            toolbar: rsx! {
                TableColumnsButton {}
                TableDensityButton {}
                TableFilterButton {}
                TableExportButton { onexport: move |text: String| csv.set(text.replace("\r\n", "|")) }
            },
            filter_panel: true,
            data: fruit(),
            columns: vec![
                column("Name").value(|fruit: &Fruit| fruit.name.to_string()).sortable().row_header(),
                column("Stock").value(|fruit: &Fruit| fruit.stock),
            ],
            default_sort: vec![TableSort::new("Name", SortDirection::Ascending)],
            default_page_size: 2,
            ondensitychange: move |size| density.set(format!("{size:?}")),
        }
        p { id: "csv", "{csv}" }
        p { id: "density", "{density}" }
    }
}
