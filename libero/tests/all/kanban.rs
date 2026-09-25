//! `Kanban`'s rendered contract: labelled card lists, the controls on each card
//! and one live region for the board.

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Kanban, KanbanCard, KanbanColumn, KanbanMove},
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

#[test]
fn each_card_has_a_handle_two_move_buttons_and_a_move_to_menu() {
    let html = body(&render(app));

    assert_eq!(count(&html, "<li"), 2, "{html}");
    assert_eq!(count(&html, "aria-label=\"Reorder\""), 2, "{html}");
    assert_eq!(count(&html, "aria-label=\"Move up\""), 2, "{html}");
    assert_eq!(count(&html, "aria-label=\"Move down\""), 2, "{html}");
    assert_eq!(count(&html, "aria-label=\"Move to column\""), 2, "{html}");
    assert_eq!(count(&html, "aria-haspopup=\"menu\""), 2, "{html}");
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
    assert!(html.contains("aria-label=\"Reorder\""), "{html}");
    assert!(html.contains("aria-label=\"Move to column\""), "{html}");
}
