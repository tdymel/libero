//! `GridZone` publishes its gap as a var its items and its own `gap` read
//! (todo 29 renders that var once per change, not every render).

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// The two half items sit on one row, split by the gap: `lg` is
/// `min(16px, 4%)`, 16px in the fixture's 400px box.
#[test]
fn two_half_items_share_a_row_split_by_the_gap() {
    block_on(async {
        let fixture = Fixture::open("/grid-zone", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "(() => { \
               const a = document.querySelector('#a').getBoundingClientRect(); \
               const b = document.querySelector('#b').getBoundingClientRect(); \
               return a.top === b.top && Math.abs(b.left - a.right - 16) < 1; \
             })()",
            "the items side by side, one gap apart",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a grid zone").unwrap();
        fixture.close().await.unwrap();
    });
}
