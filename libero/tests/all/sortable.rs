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

/// Safari drops the role of a `list-style: none` list (2508); a caller's role wins.
#[test]
fn the_list_states_its_role_and_a_callers_role_wins() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Sortable { onreorder: move |_: SortableMove| {},
                    SortableItem { index: 0, "A" }
                }
                Sortable { onreorder: move |_: SortableMove| {}, role: "listbox",
                    SortableItem { index: 0, "B" }
                }
            }
        }
    }
    let html = body(&render(app));

    assert_eq!(count(&html, "role=\"list\""), 1, "{html}");
    assert_eq!(count(&html, "role=\"listbox\""), 1, "{html}");
}

/// Before the items register, the count is unknown: no Move down is disabled (2511).
#[test]
fn no_move_down_is_disabled_on_the_first_render() {
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
    // The first pass only, as prerendered HTML ships it: no effect has run.
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    for item in ["Apple", "Pear"] {
        let at = html
            .find(&format!("aria-label=\"Move {item} down\""))
            .unwrap();
        let start = html[..at].rfind("<button").unwrap();
        let tag = &html[start..at + html[at..].find('>').unwrap()];
        assert!(!tag.contains("disabled"), "{item}: {tag}");
    }
}

/// The text of the elements `ids` points at, joined as a name is.
pub fn labelled_by(html: &str, ids: &str) -> String {
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
