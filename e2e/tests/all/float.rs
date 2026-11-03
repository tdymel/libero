//! `Float` in Chromium: a fixed one clears the viewport's safe-area insets on the edges it
//! is anchored to, flipped under `dir="rtl"`; an absolute one keeps to its card.

use chromiumoxide::cdp::browser_protocol::emulation::{
    SafeAreaInsets, SetSafeAreaInsetsOverrideParams,
};
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, js, wait};

/// Each badge's distance from the viewport or card edges it is anchored to.
const GAPS: &str = "(() => { const vw = document.documentElement.clientWidth, vh = innerHeight; \
     const r = id => document.getElementById(id).getBoundingClientRect(); \
     const card = r('card'), be = r('bottom-end'), ts = r('top-start'), inside = r('inside'); \
     return [vh - be.bottom, vw - be.right, ts.top, ts.left, be.left, ts.right - vw, \
       card.bottom - inside.bottom, card.right - inside.right]; })()";

/// Todo 5464: a fixed Float clears the notch, the status bar and the home indicator.
#[test]
fn a_fixed_float_clears_the_safe_area() {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = Fixture::open("/float", Viewport::Mobile).await.unwrap();
            let page = &fixture.page;
            wait::for_visible(page, "#bottom-end").await.unwrap();
            let insets = SafeAreaInsets {
                top: Some(47),
                bottom: Some(34),
                left: Some(20),
                right: Some(30),
                ..Default::default()
            };
            page.execute_bounded(SetSafeAreaInsetsOverrideParams::new(insets))
                .await
                .unwrap();
            let _: bool = js(
                page,
                &format!("(document.documentElement.dir = '{dir}', true)"),
            )
            .await;
            wait::for_js_true(
                page,
                "document.getElementById('top-start').getBoundingClientRect().top > 40",
                "the inset to apply",
            )
            .await
            .unwrap();
            let gaps: [f64; 8] = js(page, GAPS).await;
            let [
                bottom,
                right,
                top,
                left,
                be_left,
                ts_right,
                card_bottom,
                card_right,
            ] = gaps;
            assert!((bottom - 34.0).abs() < 1.0, "{dir} bottom: {gaps:?}");
            assert!((top - 47.0).abs() < 1.0, "{dir} top: {gaps:?}");
            match dir {
                // End is the right edge, start the left.
                "ltr" => {
                    assert!((right - 30.0).abs() < 1.0, "{dir} end: {gaps:?}");
                    assert!((left - 20.0).abs() < 1.0, "{dir} start: {gaps:?}");
                }
                // Mirrored: end at the left inset, start at the right one.
                _ => {
                    assert!((be_left - 20.0).abs() < 1.0, "{dir} end: {gaps:?}");
                    assert!((ts_right + 30.0).abs() < 1.0, "{dir} start: {gaps:?}");
                }
            }
            // Absolute: the card's corner, no inset.
            assert!(card_bottom.abs() < 1.0, "{dir} inside: {gaps:?}");
            let card_edge = if dir == "ltr" { card_right } else { 0.0 };
            assert!(card_edge.abs() < 1.0, "{dir} inside: {gaps:?}");
            fixture
                .console
                .assert_clean(&format!("a fixed float, {dir}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
