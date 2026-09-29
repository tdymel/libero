//! `:hover` on the DOM ancestors Blitz skips: it hovers layout parents only,
//! and a cell's layout parent is the table, not its `tr`.

use dioxus::prelude::*;
use e2e::native::mount;

const HOVERED: &str = "rgb(255, 0, 0)";

fn app() -> Element {
    rsx! {
        style { "tr:hover {{ background-color: {HOVERED}; }}" }
        table {
            tbody {
                tr { id: "a", td { "Apple" } }
                tr { id: "b", td { "Banana" } }
            }
        }
    }
}

#[test]
fn a_hovered_cell_hovers_its_row_and_leaving_unhovers_it() {
    let mut page = mount(app);
    page.hover("#a td");
    assert_eq!(page.computed("#a", "background-color"), HOVERED);
    page.hover("#b td");
    assert_eq!(page.computed("#b", "background-color"), HOVERED);
    assert_ne!(
        page.computed("#a", "background-color"),
        HOVERED,
        "the row the pointer left is still hovered"
    );
}
