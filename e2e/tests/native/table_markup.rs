//! Plain table markup the Table epic builds on (1156 phase 0d probe): sticky
//! cells, cell widths, spans and a moved cell, as Blitz lays out and paints them.

use dioxus::prelude::*;
use e2e::native::{Page, mount};

const RED: [u8; 4] = [255, 0, 0, 255];
const WHITE: [u8; 4] = [255, 255, 255, 255];

fn pixel(page: &Page, x: f64, y: f64) -> [u8; 4] {
    page.painted_pixels(&[(x as u32, y as u32)])[0]
}

/// A red pixel at the box's top-left corner, clear of its text.
fn painted_red(page: &Page, selector: &str) -> bool {
    let (x, y, _, _) = page.rect(selector);
    pixel(page, x + 2.0, y + 2.0) == RED
}

fn header_and_column() -> Element {
    rsx! {
        div { id: "scroller", width: "200px", height: "200px", overflow: "auto",
            table { border_spacing: "0",
                thead {
                    tr {
                        th {
                            id: "corner",
                            position: "sticky",
                            top: "0",
                            left: "0",
                            z_index: "2",
                            background: "red",
                            width: "50px",
                            "X"
                        }
                        for j in 1..8 {
                            th {
                                id: "h{j}",
                                position: "sticky",
                                top: "0",
                                background: "blue",
                                min_width: "100px",
                                "H{j}"
                            }
                        }
                    }
                }
                tbody {
                    for i in 0..30 {
                        tr { key: "{i}",
                            td {
                                id: "c{i}-0",
                                position: "sticky",
                                left: "0",
                                background: "green",
                                "R{i}"
                            }
                            for j in 1..8 {
                                td { id: "c{i}-{j}", "C{j}" }
                            }
                        }
                    }
                }
            }
            div { id: "after", height: "1000px" }
        }
    }
}

/// Sticky `th`s hold at the scroller's top, a sticky first column at its start,
/// the corner cell at both, painted on top of what scrolls under it.
#[test]
fn sticky_header_cells_and_first_column_hold() {
    let mut page = mount(header_and_column);
    let (ox, oy, _, _) = page.rect("#scroller");
    page.hover("#c5-3");
    page.wheel("#c5-3", 300.0);
    page.hover("#c5-3");
    page.wheel_x("#c5-3", 300.0);
    let held = |page: &Page| {
        let corner = page.rect("#corner");
        (corner.0 - ox, corner.1 - oy) == (0.0, 0.0)
    };
    page.wait_for(held);
    assert!(held(&page), "the corner: {:?}", page.rect("#corner"));
    assert_eq!(page.rect("#h3").1 - oy, 0.0, "the header: {}", page.tree());
    assert_eq!(
        page.rect("#c12-0").0 - ox,
        0.0,
        "the column: {}",
        page.tree()
    );
    assert!(painted_red(&page, "#corner"), "the corner is painted under");
    assert!(page.hits("#corner"), "a press on the corner misses it");
}

/// Past the table's end the header leaves with it, as on the web.
#[test]
fn a_sticky_header_cell_leaves_with_its_table() {
    let mut page = mount(header_and_column);
    let oy = page.rect("#scroller").1;
    let (_, top, _, height) = page.rect("table");
    page.hover("#after");
    page.wheel("#after", 1200.0);
    let bottom = |page: &Page| {
        let (_, y, _, h) = page.rect("#h3");
        y + h - oy
    };
    let end = top + height - oy - 1200.0;
    page.wait_for(|page| bottom(page) == end);
    assert_eq!(bottom(&page), end, "{}", page.tree());
}

fn widths() -> Element {
    rsx! {
        table { id: "fixed", border_spacing: "0", table_layout: "fixed", width: "600px",
            thead {
                tr {
                    th { id: "a", width: "300px", "A" }
                    th { id: "b", width: "100px", "B" }
                    th { id: "c", "C" }
                }
            }
            tbody {
                tr {
                    td { id: "ta", "A" }
                    td { "B" }
                    td { "C" }
                }
            }
        }
        table { border_spacing: "0",
            thead {
                tr {
                    th { id: "a2", width: "250px", "A" }
                    th { id: "b2", min_width: "120px", "B" }
                }
            }
            tbody {
                tr {
                    td { id: "ta2", "A" }
                    td { "B" }
                }
            }
        }
    }
}

/// A header cell's `width`/`min-width` sizes its column, in a fixed and an
/// auto layout; the rest goes to the column without one. (`<col>` widths do not.)
#[test]
fn a_header_cell_width_sizes_its_column() {
    let page = mount(widths);
    let width = |selector: &str| page.rect(selector).2;
    assert_eq!(
        [width("#a"), width("#b"), width("#c"), width("#ta")],
        [300.0, 100.0, 200.0, 300.0]
    );
    assert_eq!(
        [width("#a2"), width("#b2"), width("#ta2")],
        [250.0, 120.0, 250.0]
    );
}

fn spans() -> Element {
    rsx! {
        table { border_spacing: "0",
            tbody {
                tr {
                    td { id: "tall", rowspan: "2", background: "red", "Tall" }
                    td { id: "a", height: "30px", "A" }
                    td { "B" }
                }
                tr {
                    td { id: "c", height: "30px", "C" }
                    td { "D" }
                }
                tr {
                    td { id: "wide", colspan: "2", background: "red", "Wide" }
                    td { id: "e", "E" }
                }
                tr {
                    td { id: "f", "F" }
                    td { id: "g", width: "80px", "G" }
                    td { "H" }
                }
            }
        }
    }
}

/// `rowspan` covers both rows, `colspan` both columns, laid out and painted.
#[test]
fn spanned_cells_cover_their_rows_and_columns() {
    let page = mount(spans);
    let (tall, a, c) = (page.rect("#tall"), page.rect("#a"), page.rect("#c"));
    assert_eq!(tall.3, a.3 + c.3, "rowspan: {tall:?}, {a:?}, {c:?}");
    let (wide, f, g, e) = (
        page.rect("#wide"),
        page.rect("#f"),
        page.rect("#g"),
        page.rect("#e"),
    );
    assert_eq!(wide.2, f.2 + g.2, "colspan: {wide:?}, {f:?}, {g:?}");
    assert_eq!(e.0, g.0 + g.2, "the cell after the span: {e:?}");
    assert_eq!(pixel(&page, tall.0 + 2.0, c.1 + c.3 / 2.0), RED);
    assert_eq!(pixel(&page, g.0 + g.2 / 2.0, wide.1 + 2.0), RED);
}

fn moved_cell() -> Element {
    rsx! {
        table { border_spacing: "0",
            tbody {
                tr {
                    td { id: "zero", height: "30px", width: "100px", "Zero" }
                }
                tr {
                    td {
                        id: "one",
                        height: "30px",
                        width: "100px",
                        background: "red",
                        transform: "translateY(60px)",
                    }
                }
                tr {
                    td { height: "30px" }
                }
                tr {
                    td { height: "30px" }
                }
            }
        }
    }
}

fn row_var_on_cells() -> Element {
    rsx! {
        style { "tr[data-shift] > td {{ transform: var(--shift); }}" }
        table { border_spacing: "0",
            tbody {
                tr { id: "r0",
                    td { id: "zero", height: "30px", width: "100px", "Zero" }
                }
                tr { id: "r1", "data-shift": true, style: "--shift: translate(0px, 60px);",
                    td { id: "one", height: "30px", width: "100px", background: "red" }
                }
                tr { td { height: "30px" } }
                tr { td { height: "30px" } }
            }
        }
    }
}

/// `Table` moves a row by its cells (1520): a `transform` var set on the `tr`
/// reaches them, and the row's rect is its moved cells'.
#[test]
fn a_rows_cells_take_its_transform_var() {
    let page = mount(row_var_on_cells);
    let (x, y, _, height) = page.rect("#zero");
    assert_eq!(pixel(&page, x + 10.0, y + height + 75.0), RED);
    assert_eq!(page.rect("#r0"), (x, y, 100.0, 30.0));
    assert_eq!(page.rect("#r1"), (x, y + height + 60.0, 100.0, 30.0));
}

/// A row reorder moves cells: a `transform` on a `td` paints it moved (one on
/// a `tr` is not painted, the row has no box of its own).
#[test]
fn a_transformed_cell_paints_moved() {
    let page = mount(moved_cell);
    let (x, y, _, height) = page.rect("#zero");
    let in_flow = pixel(&page, x + 10.0, y + height + 15.0);
    let moved = pixel(&page, x + 10.0, y + height + 75.0);
    assert_eq!((in_flow, moved), (WHITE, RED));
}
