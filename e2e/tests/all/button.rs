//! `Button`: a busy button swallows keyboard activation, every mode shows a
//! ring on Tab, and link mode navigates through the router (todo 406).

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

const PLAIN: &str = "#plain";
const BUSY: &str = "#busy";
const LINK: &str = "#to-landing";

#[test]
fn it_meets_the_baseline() {
    Suite::new("button", "/button")
        .focusable(PLAIN)
        .focusable(BUSY)
        .focusable(LINK)
        .targets(PLAIN)
        .targets(BUSY)
        .targets(LINK)
        .run();
}

fn presses(which: &str) -> String {
    format!("document.querySelector('#presses').dataset.{which}")
}

/// The plain button is the control: its presses land after the busy ones in
/// the same event queue, so a busy count still at 0 once they have landed
/// means the busy presses were swallowed, not merely not yet handled.
#[test]
fn a_loading_button_swallows_enter_and_space() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, PLAIN, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '2'", presses("plain")),
            "Enter and Space to activate the plain button",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, BUSY, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();

        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '3'", presses("plain")),
            "the plain button's press after the busy ones",
        )
        .await
        .unwrap();

        let busy: String = page
            .evaluate(presses("busy"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(busy, "0", "a loading button ran its onclick");

        fixture
            .console
            .assert_clean("pressing a busy button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A marker on `window` survives only a client-side navigation, so it tells
/// the router's `Link` apart from a full page load of the same URL.
#[test]
fn link_mode_navigates_through_the_router() {
    block_on(async {
        let fixture = Fixture::open("/button", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        page.evaluate("window.__buttonMarker = 1").await.unwrap();
        keyboard::tab_to(page, LINK, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        wait::for_selector(page, "#landing").await.unwrap();
        let (path, kept): (String, bool) = page
            .evaluate("[location.pathname, window.__buttonMarker === 1]")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(path, "/button/landing");
        assert!(kept, "the link reloaded the page instead of routing");

        fixture
            .console
            .assert_clean("following a link button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
