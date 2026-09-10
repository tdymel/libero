//! `Table`'s sort arrow turns by a state-driven `transform` when the sort
//! flips to descending.

use dioxus::prelude::*;
use libero::components::{Table, column};
use native_tests::mount;

#[derive(Clone, PartialEq)]
struct Person {
    name: &'static str,
    age: u32,
}

const HEADER: &str = "th[aria-sort]";
const BUTTON: &str = "th[aria-sort] button";
const ARROW: &str = "th[aria-sort] svg";

fn app() -> Element {
    let data = vec![
        Person {
            name: "Ada",
            age: 36,
        },
        Person {
            name: "Grace",
            age: 85,
        },
    ];
    rsx! {
        Table {
            data,
            columns: vec![
                column("Name").value(|p: &Person| p.name.to_string()),
                column("Age").value(|p: &Person| p.age).sortable(),
            ],
        }
    }
}

#[test]
fn the_arrow_turns_when_the_sort_flips() {
    let mut page = mount(app);
    page.click(BUTTON);
    page.advance(1.0);
    assert_eq!(
        page.attr(HEADER, "aria-sort").as_deref(),
        Some("ascending"),
        "{}",
        page.tree()
    );
    let ascending = page.computed(ARROW, "transform");
    page.click(BUTTON);
    page.advance(1.0);
    assert_eq!(
        page.attr(HEADER, "aria-sort").as_deref(),
        Some("descending")
    );
    let descending = page.computed(ARROW, "transform");
    assert_ne!(ascending, descending, "the arrow stayed at {ascending}");
}
