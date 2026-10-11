//! `Table` with `cell_focus` in Blitz's AX tree (1156 D1, probe P4): roles only, as
//! Blitz maps no `aria-rowindex`, `aria-selected` or name; keys are shared scenarios.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Table, column};

fn app() -> Element {
    rsx! {
        Table {
            aria_label: "Team",
            cell_focus: true,
            data: vec![("Ana", 31), ("Ben", 42)],
            columns: vec![
                column("Name").value(|p: &(&str, u32)| p.0.to_string()).row_header(),
                column("Age").value(|p: &(&str, u32)| p.1),
            ],
        }
    }
}

/// Blitz reads a `td` as a plain cell in any table: the grid writes `role="gridcell"` there.
#[test]
fn a_grid_is_a_grid_in_the_ax_tree() {
    let page = mount(app);
    for (selector, role) in [
        ("table", "Grid"),
        ("tbody td", "GridCell"),
        ("tbody th", "RowHeader"),
    ] {
        assert_eq!(
            page.accessible(selector).0,
            role,
            "{selector}: {}",
            page.tree()
        );
    }
}
