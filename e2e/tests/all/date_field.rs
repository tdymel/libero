//! `DateField` at month level (todo 26): typed `9/2026` reads as the month's
//! first day, and the dropdown opens on the month grid.
//!
//! `/date-field/month` holds March 2026, `today` pinned to 2026-03-18.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "input[data-controlled]";

#[test]
fn a_typed_month_is_held_as_its_first_day() {
    block_on(async {
        let fixture = Fixture::open("/date-field/month", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        expect(
            page,
            &value_is("March 2026"),
            "the month shown as MMMM YYYY",
        )
        .await;
        page.evaluate(format!("document.querySelector({INPUT:?}).select()"))
            .await
            .unwrap();
        keyboard::type_text(page, "9/2026").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        expect(page, &readout_is("2026-09-01"), "9/2026 held as 2026-09-01").await;
        expect(
            page,
            &value_is("September 2026"),
            "the text redrawn as September 2026",
        )
        .await;

        fixture.console.assert_clean("typing a month").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_dropdown_opens_on_the_month_grid() {
    block_on(async {
        let fixture = Fixture::open("/date-field/month", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        let grid = "(() => { const dialog = document.querySelector('[role=dialog]'); \
             return !!dialog && !!dialog.querySelector(\"[data-date='2026-03-01']\") \
               && !dialog.querySelector('[data-slot=day]'); })()";
        expect(page, grid, "the dropdown to show months, not days").await;

        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(
            page,
            "document.activeElement.getAttribute('data-date') === '2026-03-01'",
            "Arrow Down to focus the held month",
        )
        .await;
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();

        expect(
            page,
            &readout_is("2026-04-01"),
            "April picked as 2026-04-01",
        )
        .await;
        expect(page, &value_is("April 2026"), "the text shows April 2026").await;
        expect(
            page,
            "!document.querySelector('[role=dialog]') \
             && document.activeElement === document.querySelector('input[data-controlled]')",
            "the pick to close the dropdown and return focus to the text",
        )
        .await;

        fixture.console.assert_clean("the month dropdown").unwrap();
        fixture.close().await.unwrap();
    });
}

fn value_is(text: &str) -> String {
    format!("document.querySelector({INPUT:?}).value === {text:?}")
}

fn readout_is(text: &str) -> String {
    format!("document.querySelector('#readout').textContent.trim() === {text:?}")
}

async fn expect(page: &chromiumoxide::Page, check: &str, what: &str) {
    if let Err(error) = wait::for_js_true(page, check, what).await {
        let state: String = page
            .evaluate(format!(
                "`value=${{document.querySelector({INPUT:?}).value}} readout=${{document.querySelector('#readout').textContent}} focus=${{document.activeElement.outerHTML.slice(0, 120)}}`"
            ))
            .await
            .and_then(|value| Ok(value.into_value()?))
            .unwrap_or_default();
        panic!("{what}: {error}; {state}");
    }
}
