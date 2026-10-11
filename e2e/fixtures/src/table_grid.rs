//! `Table` with `cell_focus` (1156 D1-D3): a grid of people between two buttons,
//! one with a button in each row, one selectable with column menus, a windowed
//! one of 500 rows, and one with editable Age and City cells, which stores its
//! edits or (`/kept`) only logs them.

use dioxus::prelude::*;
use libero::components::{RowUpdate, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-grid", || rsx! { GridPage {} }),
    ("/table-grid/widgets", || rsx! { GridPage { widgets: true } }),
    ("/table-grid/selectable", || rsx! { GridPage { selectable: true } }),
    ("/table-grid/windowed", || rsx! { WindowedGridPage {} }),
    ("/table-grid/editable", || rsx! { EditableGridPage { store: true } }),
    ("/table-grid/editable/kept", || rsx! { EditableGridPage { store: false } }),
];

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
    city: &'static str,
}

fn people() -> Vec<Person> {
    [
        ("Ana", 31, "Lisbon"),
        ("Ben", 42, "Berlin"),
        ("Cleo", 27, "Cairo"),
        ("Dev", 55, "Delhi"),
        ("Eli", 38, "Essen"),
    ]
    .into_iter()
    .map(|(name, age, city)| Person { name, age, city })
    .collect()
}

#[component]
fn GridPage(#[props(default)] widgets: bool, #[props(default)] selectable: bool) -> Element {
    let mut columns = vec![
        column("Name").value(|p: &Person| p.name.to_string()).sortable().row_header(),
        column("Age").value(|p: &Person| p.age).sortable(),
        column("City").value(|p: &Person| p.city.to_string()),
    ];
    if widgets {
        columns.push(column("Action").value(|p: &Person| p.name.to_string()).render(|p: &Person| {
            rsx! {
                button { r#type: "button", "Greet {p.name}" }
            }
        }));
    }
    rsx! {
        button { id: "before", "Before" }
        Table {
            caption: "Team",
            cell_focus: true,
            selectable,
            column_menu: selectable,
            data: people(),
            columns,
            row_key: |p: &Person| p.name.to_string(),
        }
        button { id: "after", "After" }
    }
}

#[component]
fn EditableGridPage(store: bool) -> Element {
    let mut rows = use_signal(people);
    let mut log = use_signal(String::new);
    rsx! {
        button { id: "before", "Before" }
        Table {
            caption: "Team",
            data: rows(),
            columns: vec![
                column("Name").value(|p: &Person| p.name.to_string()).row_header(),
                column("Age")
                    .value(|p: &Person| p.age)
                    .editable(|p: &mut Person, age: u32| p.age = age)
                    .validate(|_: &Person, age: &u32| match *age <= 150 {
                        true => Ok(()),
                        false => Err("Too old".into()),
                    }),
                column("City")
                    .value(|p: &Person| p.city.to_string())
                    .editable(|p: &mut Person, city: String| p.city = city.leak()),
            ],
            row_key: |p: &Person| p.name.to_string(),
            onrowupdate: move |update: RowUpdate<Person>| {
                log.set(format!(
                    "{} {} {}: {} {}",
                    update.key, update.index, update.column, update.row.age, update.row.city,
                ));
                if store {
                    rows.write()[update.index] = update.row;
                }
            },
        }
        output { id: "log", "{log}" }
        button { id: "after", "After" }
    }
}

#[derive(Clone, PartialEq)]
struct Item {
    id: u32,
}

#[component]
fn WindowedGridPage() -> Element {
    rsx! {
        Table {
            aria_label: "Items",
            cell_focus: true,
            max_height: "300px",
            virtual_row_height: 40.0,
            data: (1..=500).map(|id| Item { id }).collect::<Vec<_>>(),
            columns: vec![
                column("Id").value(|item: &Item| item.id).row_header(),
                column("Double").value(|item: &Item| item.id * 2),
                column("Square").value(|item: &Item| item.id * item.id),
            ],
            row_key: |item: &Item| item.id.to_string(),
        }
    }
}
