//! `ScrollArea` re-measures when its pane resizes, not only at mount and on
//! scroll (todo 425). The pane is resized by script, as a `Splitter` or a
//! `FloatingWindow` would, with the window left alone.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, wait};

/// The highest row index a `Virtualize` has in the document.
const LAST_ROW: &str = "Math.max(...[...document.querySelectorAll('#list-pane [data-row]')].map(r => Number(r.dataset.row)))";
/// How often the caller's own `onresize` on the area has run.
const RESIZES: &str = "Number(document.querySelector('#list-resizes').textContent)";

/// 20px rows: a 120px pane renders a dozen, a 600px one needs rows to 29. The
/// area now listens for `onresize` itself, and a caller's listener on it, as
/// `Scroller` has, must still run.
#[test]
fn a_taller_pane_renders_rows_to_its_new_bottom() {
    block_on(async {
        let fixture = Fixture::open("/scroll-area", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // The measured window, not the first render's 1080px guess.
        wait::for_js_true(
            page,
            &format!("{LAST_ROW} < 29 && {RESIZES} > 0"),
            "the window to fit the short pane",
        )
        .await
        .unwrap();
        let before: f64 = page.evaluate(RESIZES).await.unwrap().into_value().unwrap();
        page.evaluate("document.querySelector('#list-pane').style.height = '600px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{LAST_ROW} >= 29"),
            "rows down to the taller pane's bottom",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{RESIZES} > {before}"),
            "the caller's own onresize to run",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a list pane resize").unwrap();
        fixture.close().await.unwrap();
    });
}
