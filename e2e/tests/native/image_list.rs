//! `ImageList`: `quilted` cells measure `c*w + (c-1)*g` by `r*w + (r-1)*g` at ratio 1
//! (todos 89(c), 451); responsive `cols` follow the viewport (todo 73).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::{
    components::{GridSpan, ImageItem, ImageList},
    theme::responsive,
};

/// Columns and rows of each cell, in order: tall, two ordinary, wide, big.
const SHAPES: [(f64, f64); 5] = [(1.0, 2.0), (1.0, 1.0), (1.0, 1.0), (2.0, 1.0), (2.0, 2.0)];

fn quilt(width: &'static str) -> Element {
    let cell = || rsx! { div { background: "#777" } };
    rsx! {
        div { id: "frame", width,
            ImageList {
                cols: 2u8,
                variant: "quilted",
                gap: "md",
                items: vec![
                    ImageItem::new(cell()).rows(2),
                    ImageItem::new(cell()),
                    ImageItem::new(cell()),
                    ImageItem::new(cell()).span(GridSpan::Full),
                    ImageItem::new(cell()).span(GridSpan::Full).rows(2),
                ],
            }
        }
    }
}

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1.0
}

/// `(x, y, width, height)` of every cell, in order.
fn cells(page: &Page) -> Vec<(f64, f64, f64, f64)> {
    (1..=page.query_all("#frame [role=list] > li").len())
        .map(|n| page.rect(&format!("#frame [role=list] > li:nth-child({n})")))
        .collect()
}

fn assert_quilt(page: &Page, width: &str) {
    let cells = cells(page);
    assert_eq!(cells.len(), SHAPES.len(), "{}", page.tree());
    let g = cells[2].1 - (cells[1].1 + cells[1].3);
    assert!(g > 0.0, "{width}: no gap, nothing is under test: {cells:?}");
    let column_gap = cells[1].0 - (cells[0].0 + cells[0].2);
    assert!(
        close(column_gap, g),
        "{width}: gaps {column_gap} and {g} differ"
    );
    let (_, _, w, h) = cells[1];
    assert!(close(h, w), "{width}: an ordinary cell is {w}x{h}");
    for (index, (&(c, r), &(_, _, cell_w, cell_h))) in SHAPES.iter().zip(&cells).enumerate() {
        let (want_w, want_h) = (c * w + (c - 1.0) * g, r * h + (r - 1.0) * g);
        assert!(
            close(cell_w, want_w) && close(cell_h, want_h),
            "{width}: cell {index} ({c}x{r}) is {cell_w}x{cell_h}, expected {want_w}x{want_h}"
        );
    }
}

/// Blitz drops a `cqi` length. `quilted` sizes its cells with a percentage
/// padding instead (todo 788), so this only pins the gap.
#[test]
#[ignore = "needs Blitz: container query units (cqi)"]
fn a_cqi_length_resolves_against_its_container() {
    let page = mount(|| {
        rsx! {
            div { style: "width: 300px; container-type: inline-size",
                div { id: "sized", style: "height: 50cqi" }
            }
        }
    });
    let (_, _, _, height) = page.rect("#sized");
    assert!(close(height, 150.0), "50cqi of 300px is {height}px");
}

#[test]
fn quilted_cells_add_up_their_gaps_at_600px() {
    let page = mount(|| quilt("600px"));
    assert_quilt(&page, "600px");
}

#[test]
fn quilted_cells_add_up_their_gaps_at_350px() {
    let page = mount(|| quilt("350px"));
    assert_quilt(&page, "350px");
}

/// One, two and four to a row from `sm` (48rem) and `md` (62rem); the harness
/// viewport is past `md`.
#[test]
fn responsive_cols_follow_the_viewport() {
    let page = mount(|| {
        let cell = || ImageItem::new(rsx! { div { background: "#777" } });
        rsx! {
            div { id: "frame",
                ImageList {
                    cols: responsive(1).sm(2).md(4),
                    gap: "md",
                    items: vec![cell(), cell(), cell(), cell()],
                }
            }
        }
    });
    let cells = cells(&page);
    let top = cells[0].1;
    assert!(
        cells.iter().all(|cell| close(cell.1, top)),
        "not one row: {cells:?}"
    );
    let (list_w, gap) = (
        page.rect("#frame [role=list]").2,
        cells[1].0 - (cells[0].0 + cells[0].2),
    );
    let expected = (list_w - 3.0 * gap) / 4.0;
    assert!(
        cells.iter().all(|cell| close(cell.2, expected)),
        "cells not {expected} wide: {cells:?}"
    );
    // The standard variant's ratio box, unlike `quilted`'s, needs no `cqi`.
    assert!(
        cells.iter().all(|cell| cell.3 > 0.0),
        "a cell has no height: {cells:?}"
    );
}
