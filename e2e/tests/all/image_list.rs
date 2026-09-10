//! `ImageList`'s `quilted` cells come out at `c*w + (c-1)*g` by `r*w + (r-1)*g`
//! at ratio 1 (todos 89(c), 451): an ordinary cell stays square beside a wide
//! one, at every width. And per-breakpoint `cols` follow the viewport (todo 73).

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::{Fixture, Viewport};
use serde::Deserialize;

/// The gaps as laid out - the tall cell 0 sits beside ordinary cells 1 and 2 -
/// and every `<li>`'s box, in fixture order.
const MEASURE: &str = "(() => {
    const list = document.querySelector('#quilt-frame [role=list]');
    const cells = [...list.querySelectorAll(':scope > li')].map(li => {
        const cell = li.getBoundingClientRect();
        const media = li.firstElementChild.getBoundingClientRect();
        return [cell.left, cell.top, cell.width, cell.height, media.width, media.height];
    });
    return {
        column_gap: cells[1][0] - (cells[0][0] + cells[0][2]),
        row_gap: cells[2][1] - (cells[1][1] + cells[1][3]),
        cells,
    };
})()";

#[derive(Deserialize, Debug)]
struct Quilt {
    column_gap: f64,
    row_gap: f64,
    cells: Vec<[f64; 6]>,
}

/// Columns and rows of each fixture cell, in order: tall, two ordinary, wide, big.
const SHAPES: [(f64, f64); 5] = [(1.0, 2.0), (1.0, 1.0), (1.0, 1.0), (2.0, 1.0), (2.0, 2.0)];

fn close(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() <= 1.0
}

#[test]
fn quilted_cells_add_up_their_gaps_at_every_width() {
    block_on(async {
        for (viewport, widths) in [
            (Viewport::Desktop, &["1200px", "700px"][..]),
            (Viewport::Mobile, &["100%"][..]),
        ] {
            measure(viewport, widths).await;
        }
    });
}

async fn measure(viewport: Viewport, widths: &[&str]) {
    let fixture = Fixture::open("/image-list-quilted", viewport)
        .await
        .unwrap();
    let page = &fixture.page;

    for width in widths {
        page.evaluate(format!(
            "document.querySelector('#quilt-frame').style.width = '{width}'"
        ))
        .await
        .unwrap();
        let quilt: Quilt = page.evaluate(MEASURE).await.unwrap().into_value().unwrap();
        eprintln!("{} {width}: {quilt:?}", viewport.name());

        let g = quilt.row_gap;
        assert!(
            g > 0.0,
            "{width}: the fixture's gap is zero, nothing is under test"
        );
        assert!(
            close(quilt.column_gap, g),
            "{width}: row and column gaps differ"
        );
        let [_, _, w, h, ..] = quilt.cells[1];
        // The fixture's ratio is 1: the 2x1 cell used to ask `w + g/2` of every row.
        assert!(
            close(h, w),
            "{width}: an ordinary cell is {w}x{h}, expected {w}x{w}"
        );

        for (index, (&(c, r), cell)) in SHAPES.iter().zip(&quilt.cells).enumerate() {
            let (width_expected, height_expected) = (c * w + (c - 1.0) * g, r * h + (r - 1.0) * g);
            let [_, _, cell_w, cell_h, media_w, media_h] = *cell;
            assert!(
                close(cell_w, width_expected) && close(cell_h, height_expected),
                "{width}: cell {index} ({c}x{r}) is {cell_w}x{cell_h}, expected {width_expected}x{height_expected}"
            );
            assert!(
                close(media_w, cell_w) && close(media_h, cell_h),
                "{width}: cell {index}'s picture is {media_w}x{media_h} in a {cell_w}x{cell_h} cell"
            );
        }
    }

    fixture
        .console
        .assert_clean("a quilted image list")
        .unwrap();
    fixture.close().await.unwrap();
}

/// The list's width, the gap between the first two cells when they share a
/// row, and each cell's left, top and width.
const MEASURE_ROW: &str = "(() => {
    const list = document.querySelector('#responsive-frame [role=list]');
    const cells = [...list.querySelectorAll(':scope > li')].map(li => {
        const r = li.getBoundingClientRect();
        return [r.left, r.top, r.width];
    });
    const [a, b] = cells;
    const gap = a[1] === b[1] ? b[0] - (a[0] + a[2]) : 0;
    return { list: list.getBoundingClientRect().width, gap, cells };
})()";

#[derive(Deserialize, Debug)]
struct Row {
    list: f64,
    gap: f64,
    cells: Vec<[f64; 3]>,
}

/// Todo 73: `responsive(1).sm(2).md(4)` lays out one, two and four cells to a
/// row at a phone, tablet and desktop width, with no script measuring.
#[test]
fn responsive_cols_follow_the_viewport() {
    block_on(async {
        let fixture = Fixture::open("/image-list-responsive", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for (width, cols) in [(390, 1.0), (800, 2.0), (1280, 4.0)] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 900, 1.0, false))
                .await
                .unwrap();
            let row: Row = page
                .evaluate(MEASURE_ROW)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            eprintln!("{width}px: {row:?}");

            let top = row.cells[0][1];
            let in_first_row = row.cells.iter().filter(|cell| close(cell[1], top)).count();
            let expected = (row.list - (cols - 1.0) * row.gap) / cols;
            assert_eq!(in_first_row as f64, cols, "{width}px: {row:?}");
            assert!(
                row.cells.iter().all(|cell| close(cell[2], expected)),
                "{width}px: cells should be {expected} wide: {row:?}"
            );
        }

        fixture
            .console
            .assert_clean("a responsive image list")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
