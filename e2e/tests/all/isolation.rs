//! The harness's own promise that tests sharing one browser cannot disturb
//! each other.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

/// A chord one unit sends leaves every other page alone (597): Ctrl+PageDown once
/// activated another test's page, and Alt+ArrowLeft sent it Back.
#[test]
fn a_chord_in_one_page_does_not_navigate_another() {
    block_on(async {
        let other = Fixture::open("/loader", Viewport::Desktop).await.unwrap();
        let sender = Fixture::open("/menu", Viewport::Desktop).await.unwrap();
        let page = &sender.page;
        // The active tab takes the Back, as after `frames::start` in a full run.
        let front = e2e::frames::bring_to_front(&other.page).await.unwrap();
        let outcome = async {
            let refused = keyboard::press_with(page, keyboard::PAGE_DOWN, keyboard::CTRL).await;
            anyhow::ensure!(refused.is_err(), "Ctrl+PageDown was sent");
            keyboard::assert_chords_ignored(page, &[keyboard::PAGE_UP, keyboard::PAGE_DOWN], "1")
                .await?;
            keyboard::press_with(page, keyboard::ARROW_LEFT, keyboard::ALT).await?;
            page.evaluate("new Promise(r => setTimeout(() => r(1), 300))")
                .await?;
            let href: String = other.page.evaluate("location.href").await?.into_value()?;
            anyhow::ensure!(href.ends_with("/loader"), "the other page went to {href}");
            Ok(())
        }
        .await;
        let released = front.release().await;
        let _ = sender.close().await;
        let _ = other.close().await;
        outcome.unwrap();
        released.unwrap();
    });
}

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
