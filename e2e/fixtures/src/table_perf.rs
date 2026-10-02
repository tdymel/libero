//! Large `Table`s for the timing tests (todo 1462): a mount of 100, 1000 or 5000 rows,
//! a 1000-row table with every interactive feature, a paged 5000 and a windowed 10k.
//! Outside `/perf/`, so no render counter slows the timed renders.

use std::rc::Rc;

use dioxus::prelude::*;
use libero::components::{Column, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/table-perf/mount", || rsx! { MountPage {} }),
    ("/table-perf/full", || rsx! { FullPage {} }),
    ("/table-perf/paged", || rsx! { PagedPage {} }),
    ("/table-perf/windowed", || rsx! { WindowedPage {} }),
];

#[derive(Clone, PartialEq)]
struct Person {
    id: u32,
    name: String,
    role: &'static str,
    city: &'static str,
    age: u32,
    salary: u32,
}

const ROLES: [&str; 5] = ["Engineer", "Designer", "Manager", "Analyst", "Support"];
const CITIES: [&str; 7] = ["Berlin", "Lisbon", "Oslo", "Madrid", "Vienna", "Prague", "Dublin"];
const NAMES: [&str; 8] = ["Alice", "Bruno", "Chen", "Dana", "Emil", "Fatima", "Goran", "Hana"];

/// `n` people, deterministic, ages and salaries spread so a sort moves most rows.
fn people(n: u32) -> Rc<Vec<Person>> {
    Rc::new(
        (0..n)
            .map(|id| Person {
                id,
                name: format!("{} {id}", NAMES[id as usize % NAMES.len()]),
                role: ROLES[id as usize * 7 % ROLES.len()],
                city: CITIES[id as usize * 3 % CITIES.len()],
                age: 20 + id * 37 % 45,
                salary: 30_000 + id * 7_919 % 90_000,
            })
            .collect(),
    )
}

fn columns() -> Vec<Column<Person>> {
    vec![
        column("Name")
            .value(|p: &Person| p.name.clone())
            .sortable()
            .row_header(),
        column("Role").value(|p: &Person| p.role.to_string()).sortable(),
        column("City").value(|p: &Person| p.city.to_string()).sortable(),
        column("Age").value(|p: &Person| p.age).sortable(),
        column("Salary").value(|p: &Person| p.salary).sortable(),
    ]
}

fn key(p: &Person) -> String {
    p.id.to_string()
}

/// `#plain-N` mounts a sortable table of N rows, `#full-N` one with every feature of
/// [`FullTable`]; `#unmount` drops it. `#shown` names what is mounted.
#[component]
fn MountPage() -> Element {
    let sets = use_hook(|| [100, 1000, 5000].map(|n| (n, people(n))));
    let mut shown = use_signal(|| None::<(bool, usize)>);
    let table = shown().map(|(full, at)| {
        let data = sets[at].1.as_ref().clone();
        match full {
            true => rsx! { FullTable { data } },
            false => rsx! {
                Table { caption: "People", data, columns: columns(), row_key: key }
            },
        }
    });
    let label = shown().map_or("none".to_string(), |(full, at)| {
        format!("{}-{}", if full { "full" } else { "plain" }, sets[at].0)
    });
    rsx! {
        div {
            for (at, (n, _)) in sets.iter().enumerate() {
                button { id: "plain-{n}", onclick: move |_| shown.set(Some((false, at))), "Plain {n}" }
                button { id: "full-{n}", onclick: move |_| shown.set(Some((true, at))), "Full {n}" }
            }
            button { id: "unmount", onclick: move |_| shown.set(None), "Unmount" }
            p { id: "shown", "data-shown": "{label}" }
        }
        {table}
    }
}

/// Selectable, detail rows, column menus, resizable columns and a quick filter.
#[component]
fn FullTable(data: Vec<Person>) -> Element {
    rsx! {
        Table {
            caption: "People",
            data,
            columns: columns(),
            row_key: key,
            selectable: true,
            column_menu: true,
            resizable_columns: true,
            show_quick_filter: true,
            striped: true,
            row_has_detail: |_: &Person| true,
            row_detail: |p: &Person| Some(rsx! { "{p.name} works in {p.city}." }),
        }
    }
}

/// [`FullTable`] over 1000 rows, the data built once.
#[component]
fn FullPage() -> Element {
    let data = use_hook(|| people(1000));
    rsx! {
        FullTable { data: data.as_ref().clone() }
    }
}

/// 5000 sortable rows, 50 a page.
#[component]
fn PagedPage() -> Element {
    let data = use_hook(|| people(5000));
    rsx! {
        Table {
            caption: "People",
            data: data.as_ref().clone(),
            columns: columns(),
            row_key: key,
            selectable: true,
            default_page_size: 50usize,
        }
    }
}

/// 10k selectable rows, 40px each under a 400px cap, with a quick filter. In `#pane`,
/// where the shared wheel reps find the scroller.
#[component]
fn WindowedPage() -> Element {
    let data = use_hook(|| people(10_000));
    rsx! {
        div { id: "pane",
            Table {
                caption: "People",
                data: data.as_ref().clone(),
                columns: columns(),
                row_key: key,
                selectable: true,
                show_quick_filter: true,
                max_height: "400px",
                virtual_row_height: 40.0,
            }
        }
    }
}
