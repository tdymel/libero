//! `ActionIcon`: an icon-only `<button>`, named by its required `aria_label`,
//! as a toggle, busy and disabled.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

/// The 16px and 20px sizes pass 2.5.8 through the spacing exception only.
#[test]
fn it_meets_the_baseline() {
    Suite::new("action_icon", "/action-icon")
        .focusable("#plain")
        .focusable("#toggle")
        .focusable("#loading")
        .targets("#plain")
        .targets("#toggle")
        .targets_spaced("#sm")
        .targets_spaced("#xs")
        .run();
}

const CLICKS: &str = "document.getElementById('clicks').textContent";

/// A busy icon keeps its tab stop, and neither Enter nor a click runs it.
#[test]
fn a_loading_icon_stays_focusable_and_swallows_presses() {
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, "#loading", 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        pointer::click(page, "#loading").await.unwrap();
        // The plain icon counts too, so a count of 1 proves the busy one ran nothing.
        pointer::click(page, "#plain").await.unwrap();
        wait::for_js_true(page, &format!("{CLICKS} === '1'"), "the plain icon's click")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("pressing a busy icon")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `aria-pressed` follows the toggle.
#[test]
fn a_toggle_reports_its_pressed_state() {
    const PRESSED: &str = "document.getElementById('toggle').getAttribute('aria-pressed')";
    block_on(async {
        let fixture = Fixture::open("/action-icon", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{PRESSED} === 'false'"),
            "an unpressed toggle",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, "#toggle", 10).await.unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        wait::for_js_true(page, &format!("{PRESSED} === 'true'"), "Space to press it")
            .await
            .unwrap();

        fixture.console.assert_clean("toggling an icon").unwrap();
        fixture.close().await.unwrap();
    });
}
