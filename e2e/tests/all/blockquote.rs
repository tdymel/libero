//! `Blockquote`: the accent bar sits at the inline start and the corners away from it are rounded.

use e2e::browser::block_on;
use e2e::{Fixture, Viewport, js};

/// Todo 2915: the bar and the rounded corners follow the writing direction.
#[test]
fn the_bar_and_the_corners_follow_the_writing_direction() {
    block_on(async {
        let fixture = Fixture::open("/blockquote", Viewport::Desktop)
            .await
            .unwrap();
        let sides: Vec<Vec<String>> = js(
            &fixture.page,
            "['#ltr', '#rtl'].map(id => { const s = getComputedStyle(document.querySelector(id + ' blockquote')); \
             return [s.borderLeftWidth, s.borderRightWidth, s.borderTopLeftRadius, s.borderTopRightRadius, \
               s.borderBottomLeftRadius, s.borderBottomRightRadius]; })",
        )
        .await;
        let [ltr, rtl] = &sides[..] else {
            panic!("{sides:?}")
        };
        let flat = |side: &str| side == "0px";
        assert!(
            !flat(&ltr[0]) && flat(&ltr[1]),
            "bar, left to right: {ltr:?}"
        );
        assert!(
            flat(&ltr[2]) && !flat(&ltr[3]),
            "top corners, left to right: {ltr:?}"
        );
        assert!(
            flat(&ltr[4]) && !flat(&ltr[5]),
            "bottom corners, left to right: {ltr:?}"
        );
        assert!(
            flat(&rtl[0]) && !flat(&rtl[1]),
            "bar, right to left: {rtl:?}"
        );
        assert!(
            !flat(&rtl[2]) && flat(&rtl[3]),
            "top corners, right to left: {rtl:?}"
        );
        assert!(
            !flat(&rtl[4]) && flat(&rtl[5]),
            "bottom corners, right to left: {rtl:?}"
        );
        fixture
            .console
            .assert_clean("a right-to-left quote")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
