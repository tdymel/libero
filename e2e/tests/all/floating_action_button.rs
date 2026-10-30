//! `FloatingActionButton` in Chromium: it stays at its corner while the page scrolls,
//! flips under `dir="rtl"`, clears a fixed `NavigationBar`, keeps its focus ring
//! whole and gives a finger a 44px target at every size.

use e2e::browser::block_on;
use e2e::{Fixture, Suite, Viewport, js, wait};
use serde_json::Value;

const FAB: &str = "#fab";
const SMALL: &str = "#small";

#[test]
fn it_meets_the_baseline() {
    Suite::new("floating_action_button", "/floating-action-button")
        .focusable(FAB)
        .focusable(SMALL)
        .targets(FAB)
        .targets(SMALL)
        .run();
}

/// Its box, the viewport and the spacing it should keep from the edges.
async fn placed(page: &chromiumoxide::Page) -> Value {
    js::<Value>(
        page,
        "(() => { const r = document.querySelector('#fab').getBoundingClientRect(); \
            const gap = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--lsx-spacing-md')); \
            return { left: r.left, right: r.right, top: r.top, bottom: r.bottom, width: r.width, \
                vw: document.documentElement.clientWidth, vh: innerHeight, gap }; })()",
    )
    .await
}

fn near(a: &Value, b: f64) -> bool {
    (a.as_f64().unwrap() - b).abs() < 1.0
}

#[test]
fn it_holds_its_corner_while_the_page_scrolls() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        let _: bool = js(page, "(() => { scrollTo(0, 1500); return true; })()").await;
        wait::for_js_true(page, "scrollY >= 1500", "the scroll")
            .await
            .unwrap();

        let at = placed(page).await;
        let gap = at["gap"].as_f64().unwrap();
        assert!(near(&at["right"], at["vw"].as_f64().unwrap() - gap), "{at}");
        assert!(
            near(&at["bottom"], at["vh"].as_f64().unwrap() - gap),
            "{at}"
        );
        assert!(near(&at["width"], 60.0), "{at}");
        fixture.console.assert_clean("scrolling").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn end_is_the_left_corner_under_rtl() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/rtl", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        let at = placed(page).await;
        assert!(near(&at["left"], at["gap"].as_f64().unwrap()), "{at}");
        fixture.console.assert_clean("rtl").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_sits_above_a_fixed_navigation_bar() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button/bar", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.documentElement).getPropertyValue('--lsx-navigation-bar-height').trim().endsWith('px')",
            "the measured bar height",
        )
        .await
        .unwrap();
        let report = js::<Value>(
            page,
            "(() => { const fab = document.querySelector('#fab').getBoundingClientRect(); \
                const bar = document.querySelector('#bar').getBoundingClientRect(); \
                const gap = parseFloat(getComputedStyle(document.documentElement).getPropertyValue('--lsx-spacing-md')); \
                const icon = document.querySelector('#fab svg').getBoundingClientRect(); \
                return { fab: fab.bottom, bar: bar.top, gap, icon: icon.width, name: document.querySelector('#fab').textContent.trim() }; })()",
        )
        .await;
        let bar = report["bar"].as_f64().unwrap();
        assert!(
            near(&report["fab"], bar - report["gap"].as_f64().unwrap()),
            "{report}"
        );
        assert_eq!(report["name"], "Compose", "{report}");
        // `Button` leaves an unsized svg at 0; the FAB sizes it.
        assert!(report["icon"].as_f64().unwrap() >= 16.0, "{report}");
        fixture.console.assert_clean("over the bar").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The shadow gives way on keyboard focus: the ring's own layers come first in `box-shadow`.
#[test]
fn keyboard_focus_draws_the_ring_over_the_shadow() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, FAB).await.unwrap();
        let resting: String = js(
            page,
            "getComputedStyle(document.querySelector('#fab')).boxShadow",
        )
        .await;
        e2e::passes::keyboard::tab_to(page, FAB, 6).await.unwrap();
        let focused = js::<Value>(
            page,
            "(() => { const s = getComputedStyle(document.querySelector('#fab')); \
                return { shadow: s.boxShadow, outline: s.outlineStyle }; })()",
        )
        .await;
        assert_ne!(resting, "none", "a resting shadow");
        let shadow = focused["shadow"].as_str().unwrap();
        assert!(
            shadow.len() > resting.len(),
            "ring layers join the shadow: {focused}"
        );
        assert_eq!(focused["outline"], "solid", "{focused}");
        fixture.console.assert_clean("focusing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `xl` is 60px anyway; an `xs` one grows a 44px press area round its 24px box.
#[test]
fn a_finger_gets_44px_at_every_size() {
    block_on(async {
        let fixture = Fixture::open("/floating-action-button", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        crate::hit_area::emulate_coarse_pointer(page).await;
        wait::for_visible(page, SMALL).await.unwrap();
        let report = js::<Value>(
            page,
            "(() => { const b = document.querySelector('#small'); const r = b.getBoundingClientRect(); \
                const hit = document.elementFromPoint(r.left + r.width / 2 + 20, r.top + r.height / 2); \
                const fab = document.querySelector('#fab').getBoundingClientRect(); \
                return { hit: b.contains(hit), small: r.width, fab: Math.min(fab.width, fab.height) }; })()",
        )
        .await;
        assert_eq!(report["hit"], true, "{report}");
        assert!(report["fab"].as_f64().unwrap() >= 43.5, "{report}");
        fixture.console.assert_clean("measuring").unwrap();
        fixture.close().await.unwrap();
    });
}
