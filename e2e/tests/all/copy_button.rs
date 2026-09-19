//! `CopyButton`: a press copies, the status says so, and leaving resets it.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const BARE: &str = "#bare button";
const NAMED: &str = "#named button";

#[test]
fn it_meets_the_baseline() {
    Suite::new("copy_button", "/copy-button")
        .focusable(BARE)
        .focusable(NAMED)
        .targets(BARE)
        .targets(NAMED)
        .run();
}

fn status_is(id: &str, text: &str) -> String {
    format!("document.querySelector('#{id} [role=status]').textContent === '{text}'")
}

/// Enter copies and the always-mounted status says "Copied"; focus stays and
/// the name does not change. Tabbing away empties the status again.
#[test]
fn a_press_is_announced_and_leaving_resets_it() {
    block_on(async {
        let fixture = Fixture::open("/copy-button", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, NAMED, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("named", "Copied"),
            "the copy to be announced",
        )
        .await
        .unwrap();
        let name: String = page
            .evaluate("document.activeElement.getAttribute('aria-label')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(name, "Copy the install command");

        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(page, &status_is("named", ""), "leaving to reset the status")
            .await
            .unwrap();

        // Unnamed, it takes the localization's name.
        pointer::click(page, BARE).await.unwrap();
        wait::for_js_true(
            page,
            &status_is("bare", "Copied"),
            "the click to be announced",
        )
        .await
        .unwrap();
        let bare_name: String = page
            .evaluate(format!(
                "document.querySelector('{BARE}').getAttribute('aria-label')"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(bare_name, "Copy");

        fixture.console.assert_clean("copying").unwrap();
        fixture.close().await.unwrap();
    });
}
