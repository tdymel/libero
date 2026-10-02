//! `GridZone` publishes its gap as a var its items and its own `gap` read
//! (todo 29 renders that var once per change, not every render), queries its
//! own width for a responsive span, and lays out rows and masonry spacing.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// Opens the fixture, waits for `expression`, and closes it clean.
fn holds(expression: &str, what: &str) {
    block_on(async {
        let fixture = Fixture::open("/grid-zone", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(&fixture.page, expression, what)
            .await
            .unwrap();
        fixture.console.assert_clean("a grid zone").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The two half items sit on one row, split by the gap: `lg` is
/// `min(16px, 4%)`, 16px in the fixture's 400px box.
#[test]
fn two_half_items_share_a_row_split_by_the_gap() {
    holds(
        "(() => { \
           const a = document.querySelector('#a').getBoundingClientRect(); \
           const b = document.querySelector('#b').getBoundingClientRect(); \
           return a.top === b.top && Math.abs(b.left - a.right - 16) < 1; \
         })()",
        "the items side by side, one gap apart",
    );
}

/// The zone is 400px in a 1280px viewport: a span that asked the viewport
/// would be half from `md`, the zone's own width keeps it full.
#[test]
fn a_responsive_span_follows_its_zones_width_not_the_viewports() {
    holds(
        "(() => { \
           const zone = document.querySelector('#area-zone').getBoundingClientRect(); \
           const a = document.querySelector('#responsive-a').getBoundingClientRect(); \
           const b = document.querySelector('#responsive-b').getBoundingClientRect(); \
           return zone.width > 0 && Math.abs(a.width - zone.width) < 1 && b.top >= a.bottom; \
         })()",
        "each responsive item full width, one under the other",
    );
}

/// A zone without an area is no query container: as a shrink-to-fit flex item
/// a container would collapse to zero width and stack its items at x=0.
#[test]
fn a_zone_without_an_area_keeps_its_width_as_a_flex_item() {
    holds(
        "(() => { \
           const zone = document.querySelector('#loose-zone').getBoundingClientRect(); \
           const a = document.querySelector('#loose-a').getBoundingClientRect(); \
           const b = document.querySelector('#loose-b').getBoundingClientRect(); \
           return zone.width > 0 && a.width > 0 && a.top === b.top && b.left > a.right; \
         })()",
        "the loose zone wide enough for its two items side by side",
    );
}

/// `rows: 2` spans the two rows its neighbours stack in.
#[test]
fn an_item_spans_rows_of_its_own() {
    holds(
        "(() => { \
           const tall = document.querySelector('#tall').getBoundingClientRect(); \
           const one = document.querySelector('#right-1').getBoundingClientRect(); \
           const two = document.querySelector('#right-2').getBoundingClientRect(); \
           return tall.top === one.top && two.top > one.bottom \
             && Math.abs(tall.bottom - two.bottom) < 1; \
         })()",
        "the tall item as high as the two rows beside it",
    );
}

/// In masonry the row gap is zero: the vertical spacing is the items'
/// `margin-bottom`, which the zone's `gap` has to reach.
#[test]
fn a_masonry_zones_gap_spaces_its_items_vertically() {
    holds(
        "(() => { \
           const a = document.querySelector('#stone-a'); \
           const b = document.querySelector('#stone-b').getBoundingClientRect(); \
           const margin = parseFloat(getComputedStyle(a).marginBottom); \
           const space = b.top - a.getBoundingClientRect().bottom; \
           return margin > 0 && Math.abs(space - margin) < 1; \
         })()",
        "the masonry items one gap apart",
    );
}
