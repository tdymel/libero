//! `Kanban`'s rendered contract: labelled card lists, the controls on each card
//! and one live region for the board.

use crate::{
    common::{body, render, tags_with},
    recorder::mounted_listeners,
};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Kanban, KanbanCard, KanbanColumn, KanbanColumnMove, KanbanMove},
};

fn app() -> Element {
    rsx! {
        LiberoProvider {
            Kanban { onmove: move |_: KanbanMove| {},
                KanbanColumn { index: 0, label: "To do",
                    KanbanCard { index: 0, label: "Write", "Write" }
                    KanbanCard { index: 1, label: "Test", "Test" }
                }
                KanbanColumn { index: 1, label: "Done" }
            }
        }
    }
}

fn no_buttons_app() -> Element {
    rsx! {
        LiberoProvider {
            Kanban { onmove: move |_: KanbanMove| {}, move_buttons: false,
                KanbanColumn { index: 0, label: "To do",
                    KanbanCard { index: 0, "Write" }
                }
            }
        }
    }
}

fn count(html: &str, needle: &str) -> usize {
    html.matches(needle).count()
}

#[test]
fn each_column_is_a_list_named_by_its_header() {
    let html = body(&render(app));

    assert_eq!(count(&html, "role=\"list\""), 2, "{html}");
    assert_eq!(count(&html, "data-slot=\"header\""), 2, "{html}");
    assert!(html.contains(">To do</div>"), "{html}");
    assert!(html.contains(">Done</div>"), "{html}");
    assert_eq!(count(&html, "aria-labelledby="), 2, "{html}");
}

/// Todo 3853: a blank label names nothing: a custom header names the list, else it is unnamed.
#[test]
fn a_blank_column_label_is_not_the_lists_name() {
    fn headed() -> Element {
        rsx! {
            LiberoProvider {
                Kanban { onmove: move |_: KanbanMove| {},
                    KanbanColumn { index: 0, label: "  ", header: rsx! { "To do (3)" } }
                }
            }
        }
    }
    fn bare() -> Element {
        rsx! {
            LiberoProvider {
                Kanban { onmove: move |_: KanbanMove| {},
                    KanbanColumn { index: 0, label: "  " }
                }
            }
        }
    }

    let list = |app: fn() -> Element| {
        let html = body(&render(app));
        tags_with(&html, r#"role="list""#).remove(0)
    };
    let headed = list(headed);
    assert!(!headed.contains_key("aria-label"), "{headed:?}");
    assert!(headed.contains_key("aria-labelledby"), "{headed:?}");
    let bare = list(bare);
    assert!(!bare.contains_key("aria-label"), "{bare:?}");
    assert!(!bare.contains_key("aria-labelledby"), "{bare:?}");
}

#[test]
fn a_custom_header_leaves_the_label_as_the_lists_name() {
    fn headed() -> Element {
        rsx! {
            LiberoProvider {
                Kanban { onmove: move |_: KanbanMove| {},
                    KanbanColumn { index: 0, label: "To do", header: rsx! { "To do (3)" } }
                }
            }
        }
    }
    let html = body(&render(headed));

    assert!(html.contains("aria-label=\"To do\""), "{html}");
    assert!(!html.contains("aria-labelledby="), "{html}");
}

#[test]
fn each_card_has_a_handle_two_move_buttons_and_a_move_to_menu() {
    let html = body(&render(app));

    assert_eq!(count(&html, "<li"), 2, "{html}");
    assert_eq!(count(&html, "aria-haspopup=\"menu\""), 2, "{html}");
}

#[test]
fn each_cards_controls_name_their_card() {
    let html = body(&render(app));

    for card in ["Write", "Test"] {
        for name in [
            format!("Reorder {card}"),
            format!("Move {card} up"),
            format!("Move {card} down"),
            format!("Move {card} to column"),
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
fn the_board_has_one_live_region() {
    let html = body(&render(app));

    assert_eq!(count(&html, "role=\"status\""), 1, "{html}");
}

#[test]
fn move_buttons_false_keeps_the_handle_and_the_menu() {
    let html = body(&render(no_buttons_app));

    assert!(!html.contains("Move up"), "{html}");
    // Unlabelled, the menu names a card by its position.
    assert!(
        html.contains("aria-label=\"Move Item 1 to column\""),
        "{html}"
    );
}

/// Todo 2036: an unlabelled card's handle reads its content, as Sortable's does.
#[test]
fn an_unlabelled_cards_handle_is_named_by_its_content() {
    let html = body(&render(no_buttons_app));

    let handle = html
        .split("<button")
        .find(|tag| tag.contains("data-slot=\"handle\""))
        .map(|tag| &tag[..tag.find('>').unwrap()])
        .unwrap_or_else(|| panic!("no handle: {html}"));
    assert!(!handle.contains("aria-label="), "{handle}");
    let ids = handle
        .split("aria-labelledby=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_else(|| panic!("no aria-labelledby: {handle}"));
    assert_eq!(crate::sortable::labelled_by(&html, ids), "Reorder Write");
}

fn reorder_app() -> Element {
    rsx! {
        LiberoProvider {
            Kanban { onmove: move |_: KanbanMove| {}, oncolumnmove: move |_: KanbanColumnMove| {},
                KanbanColumn { index: 0, label: "To do",
                    KanbanCard { index: 0, label: "Write", "Write" }
                }
                KanbanColumn { index: 1, label: "Done" }
            }
        }
    }
}

#[test]
fn oncolumnmove_gives_each_header_a_handle_and_two_move_buttons() {
    let html = body(&render(reorder_app));

    for column in ["To do", "Done"] {
        for name in [
            format!("Reorder {column} column"),
            format!("Move {column} column backward"),
            format!("Move {column} column forward"),
        ] {
            assert_eq!(
                count(&html, &format!("aria-label=\"{name}\"")),
                1,
                "{name}: {html}"
            );
        }
    }
    // The first column cannot move back; the last is known only once the columns registered.
    let disabled = |name: &str| {
        html.split("<button")
            .find(|tag| tag.contains(&format!("aria-label=\"{name}\"")))
            .is_some_and(|tag| tag[..tag.find('>').unwrap()].contains("disabled"))
    };
    assert!(disabled("Move To do column backward"), "{html}");
    assert!(!disabled("Move To do column forward"), "{html}");
}

#[test]
fn a_column_handle_is_described_by_the_column_keys() {
    let html = body(&render(reorder_app));

    let handle = html
        .split("<button")
        .find(|tag| tag.contains("aria-label=\"Reorder To do column\""))
        .unwrap_or_else(|| panic!("no column handle: {html}"));
    let id = handle
        .split("aria-describedby=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_else(|| panic!("no aria-describedby: {handle}"));
    let described = html
        .split(&format!("id=\"{id}\""))
        .nth(1)
        .unwrap_or_else(|| panic!("no description {id}: {html}"));
    assert!(
        described.contains("Press Space to lift the column"),
        "{described}"
    );
}

#[test]
fn without_oncolumnmove_the_headers_have_no_controls() {
    let html = body(&render(app));

    assert!(!html.contains("Reorder To do column"), "{html}");
    assert!(!html.contains("lift the column"), "{html}");
}

/// The column handles find their nodes by selector too.
#[test]
fn column_handles_register_no_onmounted() {
    fn fixed() -> Element {
        rsx! {
            LiberoProvider {
                Kanban { onmove: move |_: KanbanMove| {},
                    KanbanColumn { index: 0, label: "To do",
                        KanbanCard { index: 0, label: "Write", "Write" }
                    }
                    KanbanColumn { index: 1, label: "Done" }
                }
            }
        }
    }
    assert_eq!(mounted_listeners(reorder_app), mounted_listeners(fixed));
}

/// Todo 2749: a card finds its parts by selector, so cards add no `onmounted`.
#[test]
fn cards_register_no_onmounted() {
    fn empty() -> Element {
        rsx! {
            LiberoProvider {
                Kanban { onmove: move |_: KanbanMove| {},
                    KanbanColumn { index: 0, label: "To do" }
                    KanbanColumn { index: 1, label: "Done" }
                }
            }
        }
    }
    assert_eq!(mounted_listeners(app), mounted_listeners(empty));
}
