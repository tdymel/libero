//! `Kanban` under Blitz's pointer: a card held at the board's end edge scrolls it to a
//! hidden column, left to right, and right to left once Blitz scrolls there (1436).
//! The browser's twin is `all/kanban.rs`.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Avatar, Badge, Flex, Kanban, KanbanCard, KanbanColumn, KanbanMove, Text};
use libero::sx::sx;

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

const TITLES: [&str; 3] = [
    "Alpha card",
    "Focus ring clipped in a scrolled board",
    "Document the column states",
];

fn narrow() -> Element {
    let mut cards = use_signal(|| vec![vec![0, 1, 2], vec![], vec![]]);
    rsx! {
        div { style: "width: 760px",
            Kanban { onmove: move |step: KanbanMove| step.apply(&mut cards.write()),
                for (column, label) in ["To do", "In progress", "Done"].into_iter().enumerate() {
                    KanbanColumn { key: "{label}", index: column, label, id: "lane-{column}",
                        header: rsx! {
                            Flex { direction: "row", justify: "space-between",
                                Text { component: "span", size: "sm", "{label}" }
                                Badge { size: "sm", circle: true, "{cards()[column].len()}" }
                            }
                        },
                        for (index, title) in cards()[column].clone().into_iter().enumerate() {
                            KanbanCard { key: "{title}", index, id: "card-{index}-{column}", label: TITLES[title],
                                // The docs demo's card: a title above two rows, in a column flex.
                                Flex { gap: "xs",
                                    Text { id: "title-{title}", size: "sm", "{TITLES[title]}" }
                                    Flex { direction: "row", gap: "4px", Badge { size: "xs", "a11y" } }
                                    Flex { direction: "row", justify: "space-between", gap: "xs", wrap: false,
                                        Text { component: "span", size: "xs", sx: sx().white_space("nowrap"), "LIB-147" }
                                        Flex { direction: "row", gap: "4px", wrap: false,
                                            Badge { size: "sm", circle: true, "2" }
                                            Avatar { name: "Grace Hopper", initials: "GH", size: "xs" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A title's glyphs in To do, from its box's left edge, and how far the last one starts
/// inside its right.
fn title_glyphs(page: &Page, title: usize) -> (Vec<(u32, i64)>, f64) {
    let selector = format!("#title-{title}");
    let (left, _, width, _) = page.rect(&selector);
    let (lane, _, lane_width, _) = page.rect("#lane-0");
    let glyphs: Vec<(u32, f64)> = page
        .painted_glyphs(&selector)
        .into_iter()
        .filter(|(_, x)| (lane..lane + lane_width).contains(x))
        .collect();
    let last = glyphs.iter().map(|(_, x)| *x).fold(f64::MIN, f64::max);
    let from_left = glyphs
        .into_iter()
        .map(|(glyph, x)| (glyph, (x - left).round() as i64))
        .collect();
    (from_left, left + width - last)
}

/// Todo 5198: the cards left in a column keep their lines inside the card; Blitz kept
/// their old text layout, wider than the card.
#[test]
fn the_cards_left_in_a_column_keep_their_lines() {
    let mut page = mount(narrow);
    let wraps = page.wrapped_text("#title-1");
    assert!(!wraps.is_empty(), "the long title does not wrap");
    let before = [1, 2].map(|title| title_glyphs(&page, title));
    let handle = centre(&page, "#card-0-0 [data-slot=handle]");
    let done = centre(&page, "#lane-2 [data-slot=list]");
    page.press_at(handle.0, handle.1);
    for step in 1..=10u8 {
        let t = f32::from(step) / 10.0;
        page.move_to(
            handle.0 + (done.0 - handle.0) * t,
            handle.1 + (done.1 - handle.1) * t,
        );
    }
    page.release_at(done.0, done.1);
    let moved = page.wait_for(|page| page.exists("#card-0-2"));
    assert!(moved, "the card never moved:\n{}", page.tree());
    for (title, (glyphs, room)) in [1, 2].into_iter().zip(before) {
        let (after, room_after) = title_glyphs(&page, title);
        assert!(
            room_after > 0.0,
            "title {title}: a glyph starts past the box, {room_after}"
        );
        assert_eq!(
            after, glyphs,
            "title {title} laid out again ({room} -> {room_after})"
        );
    }
}

#[test]
#[ignore = "707: Blitz cannot scroll an RTL box to its negative overflow"]
fn a_card_held_at_the_left_edge_scrolls_a_right_to_left_board() {
    held_at_the_end_edge_scrolls_to_done(mount(rtl), true);
}
