//! `Table` master-detail on Blitz: the detail row spans the table through
//! `colspan`, and the toggle closes it.

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::{Table, column};

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
}

const TOGGLE: &str = "button[aria-label=\"Details for Ada\"]";

fn app() -> Element {
    rsx! {
        div { width: "400px",
            Table {
                aria_label: "People",
                data: vec![Person { name: "Ada", age: 36 }, Person { name: "Grace", age: 85 }],
                columns: vec![
                    column("Name").value(|p: &Person| p.name.to_string()).row_header(),
                    column("Age").value(|p: &Person| p.age),
                ],
                row_key: |p: &Person| p.name.to_string(),
                row_detail: |p: &Person| Some(rsx! { "Born {p.age} years ago" }),
                default_expanded: vec!["Ada".to_string()],
            }
        }
    }
}

#[test]
fn the_detail_row_spans_the_table_and_its_toggle_closes_it() {
    let mut page = mount(app);
    let (table_x, _, table_width, _) = page.rect("table");
    let (x, y, width, _) = page.rect("tr[data-detail] td");
    assert!(
        (x - table_x).abs() <= 1.0 && width >= table_width - 2.0,
        "the detail cell ({x}, {width}) does not span the table ({table_x}, {table_width}): {}",
        page.tree()
    );
    let (_, row_y, _, row_height) = page.rect("tbody th");
    assert!(
        (y - (row_y + row_height)).abs() <= 1.0,
        "the detail ({y}) is not under its row ({row_y} + {row_height})"
    );
    page.click(TOGGLE);
    assert!(!page.exists("tr[data-detail]"), "{}", page.tree());
    assert_eq!(page.attr(TOGGLE, "aria-expanded").as_deref(), Some("false"));
}
