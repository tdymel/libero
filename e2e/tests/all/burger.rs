//! `Burger`: a disclosure button whose name stays put while `aria-expanded` carries the state.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard};
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, wait};

const BURGER: &str = "#burger";
const EXPANDED: &str = "document.getElementById('burger').getAttribute('aria-expanded')";
const LABEL: &str = "document.getElementById('burger').getAttribute('aria-label')";

#[test]
fn it_meets_the_baseline() {
    Suite::new("burger", "/burger")
        .focusable(BURGER)
        .targets(BURGER)
        .run();
}

/// Enter and Space both toggle `aria-expanded`; the name and the focus stay on the button.
#[test]
fn enter_and_space_toggle_the_disclosure_and_keep_the_name_and_focus() {
    block_on(async {
        let fixture = Fixture::open("/burger", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &format!("{EXPANDED} === 'false'"), "a closed burger")
            .await
            .unwrap();
        let name: String = page
            .evaluate_bounded(LABEL)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(name, "Toggle navigation");

        keyboard::tab_to(page, BURGER, 6).await.unwrap();
        for (key, expected, step) in [
            (keyboard::ENTER, "true", "Enter to open"),
            (keyboard::ENTER, "false", "Enter to close"),
            (keyboard::SPACE, "true", "Space to open"),
            (keyboard::SPACE, "false", "Space to close"),
        ] {
            keyboard::press(page, key).await.unwrap();
            wait::for_js_true(page, &format!("{EXPANDED} === '{expected}'"), step)
                .await
                .unwrap();
            focus::assert_focused(page, BURGER, step).await.unwrap();
            let label: String = page
                .evaluate_bounded(LABEL)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(label, name, "the name changed after {step}");
        }

        fixture.console.assert_clean("toggling the burger").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The panel follows `aria-expanded`, and a disabled burger is skipped by Tab.
#[test]
fn the_panel_follows_the_burger_and_a_disabled_one_takes_no_tab_stop() {
    block_on(async {
        let fixture = Fixture::open("/burger", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, BURGER).await.unwrap();

        keyboard::tab_to(page, BURGER, 6).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, "#panel").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let on_locked = focus::is_focused(page, "#locked").await.unwrap();
        assert!(!on_locked, "a disabled burger took the tab stop");

        fixture.close().await.unwrap();
    });
}
