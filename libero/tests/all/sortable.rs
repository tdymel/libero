//! `Sortable`'s rendered contract: each item's handle and move buttons name it.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Sortable, SortableItem},
    hooks::SortableMove,
};

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn each_items_controls_name_their_item() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sortable { onreorder: move |_: SortableMove| {},
                    SortableItem { index: 0, label: "Apple", "Apple" }
                    SortableItem { index: 1, label: "Pear", "Pear" }
                }
            }
        }
    }
    let html = body(&render(app));

    for item in ["Apple", "Pear"] {
        for name in [
            format!("Reorder {item}"),
            format!("Move {item} up"),
            format!("Move {item} down"),
        ] {
            assert_eq!(
                count(&html, &format!("aria-label=\"{name}\"")),
                1,
                "{name}: {html}"
            );
        }
    }
}

#[test]
fn a_rows_unlabelled_items_are_named_by_position() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sortable { onreorder: move |_: SortableMove| {}, orientation: "horizontal",
                    SortableItem { index: 0, "A" }
                    SortableItem { index: 1, "B" }
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains("aria-label=\"Reorder Item 2\""), "{html}");
    assert!(
        html.contains("aria-label=\"Move Item 1 backward\""),
        "{html}"
    );
    assert!(
        html.contains("aria-label=\"Move Item 2 forward\""),
        "{html}"
    );
}
