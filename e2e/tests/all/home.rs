//! The docs home page's booking card: filled in and submitted from the
//! keyboard alone, and the booking lands in its second tab.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const TAB_BOOK: &str = "[role=tab][aria-selected=true]";
const NAME: &str = "input[name=name]";
// The day's input is a combobox too.
const GUESTS: &str = "div[role=combobox]";

#[test]
fn the_booking_card_submits_from_the_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/home-booking", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // An empty submit is refused, and the summary takes focus.
        keyboard::tab_to(page, "button[type=submit]", 12)
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        expect(
            page,
            "document.activeElement?.closest('[data-slot=summary]') !== null",
            "the error summary to take focus",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, NAME, 12).await.unwrap();
        keyboard::type_text(page, "Linus").await.unwrap();
        // The day's text field is the next stop.
        keyboard::press(page, keyboard::TAB).await.unwrap();
        keyboard::type_text(page, "October 16, 2026").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        keyboard::tab_to(page, GUESTS, 4).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        expect(
            page,
            &format!("document.querySelector({GUESTS:?}).textContent.includes('2 guests')"),
            "2 guests picked",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, "input[role=switch]", 2)
            .await
            .unwrap();
        keyboard::press(page, keyboard::SPACE).await.unwrap();
        keyboard::tab_to(page, "button[type=submit]", 2)
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        expect(
            page,
            "document.body.textContent.includes('Booked a table for Linus.')",
            "the booking to be confirmed",
        )
        .await
        .unwrap();
        expect(
            page,
            &format!("document.querySelector({NAME:?}).value === ''"),
            "the form to reset",
        )
        .await
        .unwrap();

        // Back to the tab strip, over to the bookings.
        for _ in 0..8 {
            if focused(page, TAB_BOOK).await.unwrap() {
                break;
            }
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        }
        assert!(
            focused(page, TAB_BOOK).await.unwrap(),
            "the tab strip is reachable"
        );
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(
            page,
            "[...document.querySelectorAll('tbody tr')].some(r => r.textContent.includes('Linus') \
             && r.textContent.includes('2026-10-16') && r.textContent.includes('Terrace'))",
            "the new booking in the table",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("booking a table").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    e2e::Suite::new("home_booking", "/home-booking")
        .focusable(NAME)
        // The fields' own target sizes are their fixtures' business.
        .targets("button, [role=tab]")
        .run();
}

async fn expect(page: &Page, js: &str, what: &str) -> Result<()> {
    wait::for_js_true(page, js, what).await
}

async fn focused(page: &Page, selector: &str) -> Result<bool> {
    Ok(page
        .evaluate(format!(
            "document.activeElement === document.querySelector({selector:?})"
        ))
        .await?
        .into_value()?)
}
