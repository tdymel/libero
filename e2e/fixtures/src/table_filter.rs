//! `Table` with a select column (S1): its header filter, menu and filter popover.

use dioxus::prelude::*;
use libero::components::{ColumnFilter, Table, column};

use crate::Routes;

pub const ROUTES: Routes = &[("/table-filter/select", || rsx! { SelectFilterTablePage {} })];

#[derive(Clone, PartialEq)]
struct Staff {
    name: &'static str,
    role: &'static str,
}

/// Five staff with a Role select column, its menu and header filter; `#filters`
/// echoes the change.
#[component]
fn SelectFilterTablePage() -> Element {
    let mut filters = use_signal(String::new);
    let data: Vec<Staff> = [
        ("Ada", "Admin"),
        ("Bo", "Editor"),
        ("Cy", "Viewer"),
        ("Di", "Admin"),
        ("Ed", "Editor"),
    ]
    .into_iter()
    .map(|(name, role)| Staff { name, role })
    .collect();
    rsx! {
        Table {
            caption: "Staff",
            data,
            columns: vec![
                column("Name").value(|row: &Staff| row.name.to_string()).row_header(),
                column("Role")
                    .value(|row: &Staff| row.role.to_string())
                    .options(["Admin", "Editor", "Viewer"]),
            ],
            row_key: |row: &Staff| row.name.to_string(),
            column_menu: true,
            header_filters: true,
            oncolumnfilterschange: move |next: Vec<ColumnFilter>| {
                let text: Vec<String> = next
                    .iter()
                    .map(|filter| {
                        format!("{:?} {} [{}]", filter.operator, filter.value, filter.values.join(","))
                    })
                    .collect();
                filters.set(text.join("; "));
            },
        }
        p { id: "filters", "{filters}" }
    }
}
