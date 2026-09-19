//! A `CodeBlock` wider than its box under `dir="rtl"` (todo 735). Its code is
//! `dir="ltr"`, so a lone block paints as it does left to right. A whole block
//! overflowing an RTL flex row runs off the left edge, where Blitz cannot scroll
//! (the web scrolls there with a negative `scrollLeft`).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::CodeBlock;

const SOURCE: &str = "let s = \"a string long enough to scroll the block sideways\";";

fn lone(dir: &'static str) -> Element {
    rsx! {
        div { dir, width: "240px", padding: "16px",
            CodeBlock { id: "wide", language: "rust", source: SOURCE }
        }
    }
}

fn in_row(dir: &'static str) -> Element {
    rsx! {
        div { id: "row", dir, display: "flex", width: "240px", overflow: "auto",
            CodeBlock { id: "wide", language: "rust", flex_shrink: "0", source: SOURCE }
        }
    }
}

/// The pixels along the first code line inside `within`.
fn ink(page: &Page, within: &str) -> Vec<[u8; 4]> {
    let (x, _, w, _) = page.rect(within);
    let (_, y, _, h) = page.rect("#wide code");
    let row = (y + h / 2.0) as u32;
    let points: Vec<(u32, u32)> = (x as u32 + 1..(x + w) as u32 - 1)
        .map(|at| (at, row))
        .collect();
    page.painted_pixels(&points)
}

#[test]
fn a_lone_rtl_code_block_paints_as_ltr() {
    let ltr = ink(&mount(|| lone("ltr")), "#wide");
    let rtl = ink(&mount(|| lone("rtl")), "#wide");
    assert!(ltr.iter().any(|p| *p != ltr[0]), "no code painted");
    assert_eq!(ltr, rtl, "an RTL block paints differently");
}

/// Scrolling toward the end of an overflowing LTR row shows other code.
#[test]
fn an_ltr_row_scrolls_to_its_overflow() {
    let mut page = mount(|| in_row("ltr"));
    let before = ink(&page, "#row");
    page.hover("#row");
    page.wheel_x("#row", 120.0);
    assert_ne!(before, ink(&page, "#row"), "the row did not scroll");
}

#[test]
#[ignore = "needs Blitz: RTL scroll origin, no scrolling into negative overflow (todo 735)"]
fn an_rtl_row_scrolls_to_its_overflow() {
    let mut page = mount(|| in_row("rtl"));
    let before = ink(&page, "#row");
    page.hover("#row");
    page.wheel_x("#row", -120.0);
    let (row, block) = (page.rect("#row"), page.rect("#wide"));
    assert_ne!(
        before,
        ink(&page, "#row"),
        "the start stays clipped: block {block:?} in row {row:?}"
    );
}
