//! `Kanban` under Blitz's pointer: a card held at the board's end edge scrolls it to a
//! hidden column, left to right, and right to left once Blitz scrolls there (1436).
//! The browser's twin is `all/kanban.rs`.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Kanban, KanbanCard, KanbanColumn, KanbanMove, Text};

const COLUMNS: [&str; 3] = ["To do", "Doing", "Done"];

fn board(dir: &'static str) -> Element {
    let mut cards = use_signal(|| vec![vec!["Alpha", "Beta", "Gamma"], vec!["Delta"], vec![]]);
    let order = cards()
        .iter()
        .map(|column| column.join(" "))
        .collect::<Vec<_>>()
        .join(" | ");
    rsx! {
        div { dir, style: "padding: 16px; width: 360px",
            Kanban {
                id: "board",
                onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
                for (column, label) in COLUMNS.into_iter().enumerate() {
                    KanbanColumn { key: "{label}", index: column, label, id: "column-{column}",
                        for (index, name) in cards()[column].clone().into_iter().enumerate() {
                            KanbanCard { key: "{name}", index, id: "{name}", label: name, Text { "{name}" } }
                        }
                    }
                }
            }
            p { id: "order", "{order}" }
        }
    }
}

fn ltr() -> Element {
    board("ltr")
}

fn rtl() -> Element {
    board("rtl")
}

fn centre(page: &Page, selector: &str) -> (f32, f32) {
    let (left, top, width, height) = page.rect(selector);
    ((left + width / 2.0) as f32, (top + height / 2.0) as f32)
}

/// Done's distance past the board's end edge: positive while hidden, zero once scrolled in.
fn done_past_end(page: &Page, rtl: bool) -> f64 {
    let (board, _, board_width, _) = page.rect("#board");
    let (done, _, done_width, _) = page.rect("#column-2");
    match rtl {
        true => board - done,
        false => done + done_width - (board + board_width),
    }
}

fn held_at_the_end_edge_scrolls_to_done(mut page: Page, rtl: bool) {
    assert!(
        done_past_end(&page, rtl) > 0.0,
        "Done starts in view: nothing to scroll to"
    );
    let (board, _, board_width, _) = page.rect("#board");
    let handle = centre(&page, "#Gamma [data-slot=handle]");
    let edge = match rtl {
        true => (board + 8.0) as f32,
        false => (board + board_width - 8.0) as f32,
    };
    page.press_at(handle.0, handle.1);
    for step in 1..=10u8 {
        let t = f32::from(step) / 10.0;
        page.move_to(handle.0 + (edge - handle.0) * t, handle.1);
    }
    let scrolled = page.wait_for(|page| done_past_end(page, rtl).abs() < 2.0);
    assert!(
        scrolled,
        "a card held at the end edge left Done {} px past the board's end",
        done_past_end(&page, rtl)
    );
    let done = centre(&page, "#column-2");
    page.move_to(done.0, handle.1);
    page.release_at(done.0, handle.1);
    let dropped = page.wait_for(|page| page.text("#order") == "Alpha Beta | Delta | Gamma");
    assert!(dropped, "#order reads {}", page.text("#order"));
}

#[test]
fn a_card_held_at_the_end_edge_scrolls_the_board() {
    held_at_the_end_edge_scrolls_to_done(mount(ltr), false);
}

#[test]
#[ignore = "707: Blitz cannot scroll an RTL box to its negative overflow"]
fn a_card_held_at_the_left_edge_scrolls_a_right_to_left_board() {
    held_at_the_end_edge_scrolls_to_done(mount(rtl), true);
}
