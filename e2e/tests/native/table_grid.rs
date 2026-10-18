//! `Table` with `cell_focus` in Blitz's AX tree (1156 D1, probe P4): roles only, as
//! Blitz maps no `aria-rowindex`, `aria-selected` or name; keys are shared scenarios.
//! And an editable cell's input on Blitz (D3, probe P2).

use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{RowUpdate, Table, column};

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
        // A header cell is a grid cell by its tabindex, and still a column header (D2).
        ("thead th[tabindex]", "ColumnHeader"),
    ] {
        assert_eq!(
            page.accessible(selector).0,
            role,
            "{selector}: {}",
            page.tree()
        );
    }
}

fn editable() -> Element {
    let mut rows = use_signal(|| vec![("Ana", 31_u32), ("Ben", 42)]);
    rsx! {
        Table {
            aria_label: "Team",
            data: rows(),
            columns: vec![
                column("Name").value(|p: &(&str, u32)| p.0.to_string()).row_header(),
                column("Age")
                    .value(|p: &(&str, u32)| p.1)
                    .editable(|p: &mut (&str, u32), age: u32| p.1 = age),
            ],
            row_key: |p: &(&str, u32)| p.0.to_string(),
            onrowupdate: move |update: RowUpdate<(&'static str, u32)>| {
                rows.write()[update.index] = update.row;
            },
        }
    }
}

/// A double click opens the editor, which takes the typing; Enter commits it (D3).
#[test]
fn an_input_in_a_cell_takes_typing() {
    const AGE: &str = "tbody tr:nth-child(1) td";
    const EDITOR: &str = "[data-cell-editor] input";
    let mut page = mount(editable);
    page.double_click(AGE);
    assert!(page.is_focused(EDITOR), "{}", page.focus_owner());
    // Blitz puts the caret first.
    page.press(Key::Named(NamedKey::End));
    for _ in 0..2 {
        page.press(Key::Named(NamedKey::Backspace));
    }
    for digit in ["4", "0"] {
        page.press(Key::Character(digit.into()));
    }
    page.press(Key::Named(NamedKey::Enter));
    assert!(!page.exists(EDITOR), "the editor stays open");
    assert_eq!(page.text(AGE).trim(), "40");
}
