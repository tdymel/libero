//! `ImageList`'s `quilted` cells are `c*w + (c-1)*g` by `r*w + (r-1)*g` at ratio 1 (89(c),
//! 451), at every width; per-breakpoint `cols` follow the viewport (73).

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};
use serde::Deserialize;

/// Todo 609: the captioned cell is a `figure`, its bar the `figcaption`.
#[test]
fn it_meets_the_baseline() {
    Suite::new("image_list", "/image-list-captions").run();
}

/// How far the focused element's ring reaches past any ancestor up to `root`
/// that clips its overflow, in px, per clipped axis: the stripe's outline and the halo's
/// plain spread shadows (todo 2860). Zero or less: the whole ring shows; zero when nothing clips.
pub fn ring_clipped(root: &str) -> String {
    format!(
        "(() => {{ const el = document.activeElement; const s = getComputedStyle(el); \
         const halo = s.boxShadow.split(/,(?![^(]*\\))/).filter(p => !p.includes('inset')) \
           .map(p => (p.match(/-?[\\d.]+px/g) || []).map(parseFloat)) \
           .filter(n => n.length === 4 && n[0] === 0 && n[1] === 0 && n[2] === 0).map(n => n[3]); \
         const reach = Math.max(parseFloat(s.outlineOffset) + parseFloat(s.outlineWidth), ...halo); \
         const r = el.getBoundingClientRect(); const stop = document.querySelector('{root}'); \
         let worst = -Infinity; \
         for (let clip = el.parentElement; clip && clip !== stop.parentElement; clip = clip.parentElement) {{ \
           const cs = getComputedStyle(clip); const c = clip.getBoundingClientRect(); \
           if (cs.overflowX !== 'visible') worst = Math.max(worst, c.left - (r.left - reach), (r.right + reach) - c.right); \
           if (cs.overflowY !== 'visible') worst = Math.max(worst, c.top - (r.top - reach), (r.bottom + reach) - c.bottom); }} \
         return isFinite(worst) ? worst : 0; }})()"
    )
}

/// Todo 2523: a quilted cell's `Below` bar, unclipped since 2481, stays inside its
/// `<li>`, and no two cells overlap: the `1fr` rows grow to the bar.
#[test]
fn a_quilted_below_bar_stays_inside_its_cell() {
    block_on(async {
        let fixture = Fixture::open("/image-list-quilted-below", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[role=list] figcaption")
            .await
            .unwrap();
        let problems: Vec<String> = page
            .evaluate(
                "(() => { const cells = [...document.querySelectorAll('[role=list] > li')]; \
                 const rects = cells.map(c => c.getBoundingClientRect()); const out = []; \
                 cells.forEach((cell, i) => { const r = rects[i]; \
                   for (const el of cell.querySelectorAll('*')) { const e = el.getBoundingClientRect(); \
                     if (e.bottom > r.bottom + 0.5 || e.right > r.right + 0.5) \
                       out.push(`cell ${i}: ${el.tagName} ends at ${e.bottom} past ${r.bottom}`); } \
                   rects.forEach((o, j) => { if (j > i && r.left < o.right - 0.5 && o.left < r.right - 0.5 \
                     && r.top < o.bottom - 0.5 && o.top < r.bottom - 0.5) out.push(`cells ${i} and ${j} overlap`); }); }); \
                 return out; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        fixture.close().await.unwrap();
    });
}

/// Todo 2858: a caption word wider than its cell wraps inside the bar, on an overlay
/// bar, which clips, and on a `Below` bar, which would spill into the neighbour.
#[test]
fn an_unbreakable_caption_word_wraps_inside_its_cell() {
    block_on(async {
        let fixture = Fixture::open("/image-list-long-caption", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "[role=list] figcaption")
            .await
            .unwrap();
        let problems: Vec<String> = page
            .evaluate(
                "(() => { const out = []; \
                 for (const [i, bar] of document.querySelectorAll('[role=list] figcaption').entries()) { \
                   const cell = bar.closest('li').getBoundingClientRect(); \
                   const range = document.createRange(); range.selectNodeContents(bar); \
                   const text = range.getBoundingClientRect(); \
                   if (text.right > cell.right + 0.5) out.push(`bar ${i}: text ends at ${text.right} past ${cell.right}`); } \
                 return out.length ? out : (document.querySelectorAll('[role=list] figcaption').length === 2 ? [] : ['no bars']); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        fixture.close().await.unwrap();
    });
}

/// Todo 618: a `to` cell's link fills its `overflow: hidden` `<li>`, so an
/// outset ring was cut away on all four sides (WCAG 2.4.7).
#[test]
fn a_focused_link_cell_keeps_its_ring() {
    block_on(async {
        let fixture = Fixture::open("/image-list-links", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "[role=list] a", 10).await.unwrap();
        let clipped: f64 = page
            .evaluate(ring_clipped("[role=list]"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            clipped <= 0.0,
            "the ring runs {clipped}px past the cell's clip"
        );
        // Todo 2480: the picture covers the link's own inset shadows: the stripe is the overlay's.
        let overlay: String = e2e::js(
            page,
            "getComputedStyle(document.activeElement, '::before').boxShadow",
        )
        .await;
        assert!(
            overlay.contains("inset"),
            "no ring over the picture: {overlay}"
        );
        let z: String = e2e::js(
            page,
            "getComputedStyle(document.activeElement, '::before').zIndex",
        )
        .await;
        assert_eq!(z, "2", "the overlay must paint over the bar");
        fixture.close().await.unwrap();
    });
}

const ZOOM: &str = "[role=list] button[aria-haspopup=dialog]";

/// Todos 2425, 2426, 2481: the zoom button fills the clipped media box and the `<li>`,
/// the `Below` bar's buttons sit at the cell's start and end.
#[test]
fn a_zoom_button_and_a_below_bar_button_keep_their_rings() {
    block_on(async {
        let fixture = Fixture::open("/image-list-focus", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        for target in [ZOOM, "#below-start", "#below-action"] {
            keyboard::tab_to(page, target, 10).await.unwrap();
            let clipped: f64 = page
                .evaluate(ring_clipped("[role=list]"))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                clipped <= 0.0,
                "{target}: the ring runs {clipped}px past the cell's clip"
            );
            if target == ZOOM {
                // The picture covers the button's own inset shadows: the stripe is the overlay's.
                let overlay: String = page
                    .evaluate("getComputedStyle(document.activeElement, '::after').boxShadow")
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                assert!(
                    overlay.contains("inset"),
                    "no ring over the picture: {overlay}"
                );
            }
        }
        fixture
            .console
            .assert_clean("a focused image list")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2577: a focused `video` cell covers inset shadows, so its stripe is a painted outline.
#[test]
fn a_focused_video_cell_shows_a_painted_ring() {
    block_on(async {
        let fixture = Fixture::open("/image-list-focus", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#cell-video", 10).await.unwrap();
        let outline: Vec<String> = e2e::js(
            page,
            "(() => { const s = getComputedStyle(document.activeElement); \
               return [s.outlineStyle, s.outlineColor, s.outlineOffset]; })()",
        )
        .await;
        assert_eq!(outline[0], "solid", "{outline:?}");
        assert_ne!(outline[1], "rgba(0, 0, 0, 0)", "{outline:?}");
        assert!(outline[2].starts_with('-'), "{outline:?}");
        let clipped: f64 = e2e::js(page, ring_clipped("[role=list]")).await;
        assert!(clipped <= 0.0, "the ring runs {clipped}px past the clip");
        // Todo 2670: the overlay beside it paints the halo band over the picture too.
        let halo: String = e2e::js(
            page,
            "(() => { const s = getComputedStyle(document.activeElement.nextElementSibling); \
               return s.position === 'absolute' ? s.boxShadow.split(/,(?![^(]*\\))/)[0] : ''; })()",
        )
        .await;
        assert!(halo.contains("inset"), "no halo overlay: {halo}");
        assert!(
            !halo.contains(" 0px 0px 0px 0px"),
            "the halo band is flat: {halo}"
        );
        fixture
            .console
            .assert_clean("a focused video cell")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// What a pointer at the centre of `selector` hits: the link's `href`, or the
/// hit element's id when it is not in a link.
fn hit_at(selector: &str) -> String {
    format!(
        "(() => {{ const r = document.querySelector('{selector}').getBoundingClientRect(); \
         const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2); \
         const link = hit.closest('a'); \
         return link ? link.getAttribute('href') : hit.id; }})()"
    )
}

/// Todo 1588: the caption's text passes a click to the cell's link, its
/// button keeps its own.
#[test]
fn a_linked_cell_bar_passes_clicks_to_the_link_but_not_from_its_controls() {
    block_on(async {
        let fixture = Fixture::open("/image-list-links", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        for (selector, expected) in [
            ("#bar-caption", "/image-list-responsive"),
            ("#bar-action", "bar-action"),
        ] {
            let hit: String = page
                .evaluate(hit_at(selector))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(hit, expected, "a pointer on {selector}");
        }
        fixture.close().await.unwrap();
    });
}

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

/// Every `<li>`'s top and left once the masonry engine has measured them all,
/// in DOM order.
const MEASURE_MASONRY: &str = "(() => {
    const cells = [...document.querySelectorAll('#masonry-frame [role=list] > li')];
    if (!cells.every(li => (li.dataset.state || '').split(' ').includes('measured'))) return null;
    return cells.map(li => { const r = li.getBoundingClientRect(); return [r.top, r.left]; });
})()";

/// Todo 610: `masonry` only spans rows, and sparse auto-placement never moves
/// back, so the reading order (by top, then left) is the DOM order.
#[test]
fn masonry_keeps_the_reading_order_of_the_dom() {
    block_on(async {
        let fixture = Fixture::open("/image-list-masonry", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("{MEASURE_MASONRY} !== null"),
            "every cell measured",
        )
        .await
        .unwrap();
        let cells: Vec<[f64; 2]> = page
            .evaluate(MEASURE_MASONRY)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        eprintln!("masonry: {cells:?}");

        // Packed, not a plain grid: the second three do not share a row.
        assert!(
            !(close(cells[3][0], cells[4][0]) && close(cells[4][0], cells[5][0])),
            "nothing packed: {cells:?}"
        );
        let mut visual: Vec<usize> = (0..cells.len()).collect();
        visual.sort_by(|&a, &b| {
            let (a, b) = (cells[a], cells[b]);
            match close(a[0], b[0]) {
                true => a[1].total_cmp(&b[1]),
                false => a[0].total_cmp(&b[0]),
            }
        });
        assert_eq!(visual, (0..cells.len()).collect::<Vec<_>>(), "{cells:?}");

        fixture
            .console
            .assert_clean("a masonry image list")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
