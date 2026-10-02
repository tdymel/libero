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

    assert!(
        html.contains("aria-label=\"Move Item 1 backward\""),
        "{html}"
    );
    assert!(
        html.contains("aria-label=\"Move Item 2 forward\""),
        "{html}"
    );
}

/// The text of the elements `ids` points at, joined as a name is.
fn labelled_by(html: &str, ids: &str) -> String {
    ids.split_whitespace()
        .map(|id| {
            let after = html.split(&format!("id=\"{id}\"")).nth(1).unwrap();
            let text = after.split_once('>').unwrap().1;
            text.split('<').next().unwrap().trim().to_string()
        })
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Todo 1280: an unlabelled item's handle reads its content, not its position.
#[test]
fn an_unlabelled_items_handle_is_named_by_its_content() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sortable { onreorder: move |_: SortableMove| {},
                    SortableItem { index: 0, "Apple" }
                    SortableItem { index: 1, "Pear" }
                }
            }
        }
    }
    let html = body(&render(app));

    let handles: Vec<&str> = html
        .split("<button")
        .skip(1)
        .filter(|tag| tag.contains("data-slot=\"handle\""))
        .map(|tag| &tag[..tag.find('>').unwrap()])
        .collect();
    assert_eq!(handles.len(), 2, "{html}");
    for (handle, item) in handles.iter().zip(["Apple", "Pear"]) {
        assert!(!handle.contains("aria-label="), "{handle}");
        let ids = handle
            .split("aria-labelledby=\"")
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_else(|| panic!("no aria-labelledby: {handle}"));
        assert_eq!(labelled_by(&html, ids), format!("Reorder {item}"));
    }
}

#[test]
fn a_labelled_items_handle_keeps_its_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sortable { onreorder: move |_: SortableMove| {}, orientation: "horizontal",
                    SortableItem { index: 0, label: "Apple", "A" }
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains("aria-label=\"Reorder Apple\""), "{html}");
    assert!(!html.contains("aria-labelledby"), "{html}");
}
