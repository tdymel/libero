//! `Indicator` on the badge fixture: the capped count, the ring, the ping and its reduced-motion arm.

use e2e::browser::block_on;
use e2e::passes::motion;
use e2e::{Fixture, Viewport, wait};

/// The ping's `::before` animation name; `none` when it does not run.
const PING: &str =
    "getComputedStyle(document.querySelector('#processing'), '::before').animationName";

/// The count caps at `99+`, a small one shows as is, and every dot is hidden from the tree.
#[test]
fn a_count_is_capped_and_hidden_from_the_tree() {
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
        let [capped, plain, shown]: [bool; 3] = fixture
            .page
            .evaluate(
                "(() => { const dots = [...document.querySelectorAll('span[aria-hidden=true]')]; \
                 const texts = dots.map(d => d.textContent); \
                 return [texts.includes('99+') && !texts.includes('128'), texts.includes('7'), \
                 ['#dot', '#ringed', '#processing'].every(id => \
                 document.querySelector(id).getAttribute('aria-hidden') === 'true')]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(capped, "128 over the cap of 99 does not read `99+`");
        assert!(plain, "a count under the cap does not show as is");
        assert!(shown, "a dot is exposed to the tree");
        fixture.close().await.unwrap();
    });
}

/// The ring is a shadow, not a border: it adds no size, so the dot stays where it was.
#[test]
fn the_ring_is_drawn_outside_the_dot() {
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
        wait::for_visible(&fixture.page, "#ringed").await.unwrap();
        let [ringed, bare, border]: [String; 3] = fixture
            .page
            .evaluate(
                "(() => { const box = id => { const r = document.querySelector(id).getBoundingClientRect(); \
                 return `${r.width}x${r.height}`; }; \
                 const s = getComputedStyle(document.querySelector('#ringed')); \
                 return [s.boxShadow, `${box('#ringed')}/${box('#dot')}`, s.borderTopWidth]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(ringed, "none", "the ring is gone");
        let (ringed_box, bare_box) = bare.split_once('/').unwrap();
        assert_eq!(ringed_box, bare_box, "the ring changed the dot's size");
        assert_eq!(border, "0px", "the ring is a border");
        fixture.close().await.unwrap();
    });
}

/// The ping runs while `processing` is set, and stands still under reduced motion (WCAG 2.2.2).
#[test]
fn the_ping_runs_and_stops_under_reduced_motion() {
    block_on(async {
        let fixture = Fixture::open("/badge", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, false).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{PING} === 'lsx-indicator-processing'"),
            "the ping to run",
        )
        .await
        .unwrap();
        let bare_ping: String = e2e::js(
            page,
            "getComputedStyle(document.querySelector('#dot'), '::before').animationName",
        )
        .await;
        assert_eq!(bare_ping, "none", "a dot without `processing` pings");
        motion::set_reduced_motion(page, true).await.unwrap();
        motion::assert_reduced_motion_matches(page).await.unwrap();
        wait::for_js_true(page, &format!("{PING} === 'none'"), "the ping to stop")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}
