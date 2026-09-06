//! The harness's own promise that tests sharing one browser cannot disturb
//! each other.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

/// An open combobox stays open while another test opens a page beside it.
#[test]
fn a_page_keeps_focus_while_another_opens() {
    block_on(async {
        let first = Fixture::open("/autocomplete", Viewport::Desktop)
            .await
            .unwrap();
        keyboard::tab_to(&first.page, "[role=combobox]", 10)
            .await
            .unwrap();
        keyboard::press(&first.page, keyboard::ARROW_DOWN)
            .await
            .unwrap();
        let expanded =
            "document.querySelector('[role=combobox]').getAttribute('aria-expanded') === 'true'";
        wait::for_js_true(&first.page, expanded, "the list to open")
            .await
            .unwrap();

        let second = Fixture::open("/select", Viewport::Desktop).await.unwrap();
        keyboard::tab_to(&second.page, "[role=combobox]", 10)
            .await
            .unwrap();

        let (has_focus, still_open): (bool, bool) = first
            .page
            .evaluate(format!("[document.hasFocus(), {expanded}]"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let _ = second.close().await;
        let _ = first.close().await;
        assert!(
            has_focus && still_open,
            "opening a second page took focus from the first (hasFocus {has_focus}, list open {still_open})"
        );
    });
}
