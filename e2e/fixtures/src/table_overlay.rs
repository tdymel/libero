//! `Table` loading, empty states and toolbar (1156-3d): the toolbar's buttons
//! toggle `loading` and empty the rows.

use dioxus::prelude::*;
use libero::components::{Button, RowFn, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-overlay", || rsx! { OverlayTablePage {} }),
    ("/table-overlay-bare", || rsx! { BareTablePage {} }),
];

#[derive(Clone, PartialEq)]
struct Fruit {
    name: &'static str,
    stock: u32,
}

const FRUIT: [Fruit; 3] = [
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
        stock: 7,
    },
];

#[component]
fn OverlayTablePage() -> Element {
    let mut loading = use_signal(|| false);
    let mut empty = use_signal(|| false);
    rsx! {
        div { style: "width: 480px",
            Table {
                aria_label: "Fruit",
                loading: loading(),
                show_quick_filter: true,
                no_results: rsx! { span { id: "no-results", "No fruit matches" } },
                toolbar: rsx! {
                    Button {
                        id: "toggle-loading",
                        variant: "outlined",
                        aria_pressed: loading(),
                        onclick: move |_| loading.toggle(),
                        "Loading"
                    }
                    Button {
                        id: "toggle-rows",
                        variant: "outlined",
                        aria_pressed: empty(),
                        onclick: move |_| empty.toggle(),
                        "No rows"
                    }
                },
                data: if empty() { Vec::new() } else { FRUIT.to_vec() },
                columns: vec![
                    column("Name").value(|fruit: &Fruit| fruit.name.to_string()).row_header(),
                    column("Stock").value(|fruit: &Fruit| fruit.stock).sortable(),
                ],
                row_key: |fruit: &Fruit| fruit.name.to_string(),
            }
        }
    }
}

/// No toolbar, in a flex row as the docs preview is: the buttons outside switch
/// `loading` and `row_detail` on (todos 1440, 1461).
#[component]
fn BareTablePage() -> Element {
    let mut loading = use_signal(|| false);
    let mut detail = use_signal(|| false);
    rsx! {
        button { id: "toggle-loading", onclick: move |_| loading.toggle(), "Loading" }
        button { id: "toggle-detail", onclick: move |_| detail.toggle(), "Detail" }
        div { style: "display: flex; width: 480px",
            Table {
                aria_label: "Fruit",
                loading: loading(),
                row_detail: match detail() {
                    true => (|fruit: &Fruit| Some(rsx! { "{fruit.name} in stock" })).into(),
                    false => RowFn::default(),
                },
                data: FRUIT.to_vec(),
                columns: vec![
                    column("Name").value(|fruit: &Fruit| fruit.name.to_string()).row_header(),
                    column("Stock").value(|fruit: &Fruit| fruit.stock),
                ],
                row_key: |fruit: &Fruit| fruit.name.to_string(),
            }
        }
    }
}
