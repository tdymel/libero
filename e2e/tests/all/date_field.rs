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

/// The dropdown is portaled after the page: Tab past either end goes back
/// through the text input, not to the end of the document (todo 449).
#[test]
fn tab_past_the_dropdown_moves_on_from_the_field() {
    block_on(async {
        let fixture = Fixture::open("/date-field/day", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(
            page,
            "document.activeElement.getAttribute('data-date') === '2026-03-18'",
            "Arrow Down to focus the held day",
        )
        .await;
        keyboard::press(page, keyboard::TAB).await.unwrap();
        expect(
            page,
            "document.activeElement.id === 'after' && !document.querySelector('[role=dialog]')",
            "Tab from the days to land on the button after the field and close the dropdown",
        )
        .await;

        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        // The days, Next month, the title: Previous month is disabled at `min`.
        for _ in 0..3 {
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        }
        expect(
            page,
            &format!("document.activeElement === document.querySelector({INPUT:?})"),
            "Shift+Tab from the title to land on the text input",
        )
        .await;

        fixture
            .console
            .assert_clean("tabbing out of the dropdown")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The same from the clock face of a date-time field, after a day pick moved
/// focus there.
#[test]
fn tab_past_the_clock_face_moves_on_from_the_field() {
    block_on(async {
        let fixture = Fixture::open("/date-field/moment", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        expect(
            page,
            "document.activeElement.getAttribute('data-slot') === 'face'",
            "a day pick to move focus to the clock face",
        )
        .await;
        keyboard::press(page, keyboard::TAB).await.unwrap();
        expect(
            page,
            "document.activeElement.id === 'after' && !document.querySelector('[role=dialog]')",
            "Tab from the face to land on the button after the field",
        )
        .await;
        expect(
            page,
            &readout_is("2026-03-19 09:30:00"),
            "the picked day kept",
        )
        .await;

        fixture
            .console
            .assert_clean("tabbing out of the face")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 536: the error says why text was refused, and describes the input.
/// `/date-field/day` has `min` March 5, 2026 and weekends excluded.
#[test]
fn a_refused_date_says_which_rule_it_broke() {
    block_on(async {
        let fixture = Fixture::open("/date-field/day", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        for (text, error) in [
            ("March 1, 2026", "Must be on or after March 5, 2026"),
            ("March 21, 2026", "That date is not available"),
            ("Marchember", "Not a valid date"),
        ] {
            page.evaluate(format!("document.querySelector({INPUT:?}).select()"))
                .await
                .unwrap();
            keyboard::type_text(page, text).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            expect(
                page,
                &format!(
                    "(() => {{ const input = document.querySelector({INPUT:?}); \
                     const ids = (input.getAttribute('aria-describedby') || '').split(' '); \
                     return input.getAttribute('aria-invalid') === 'true' && ids.some(id => \
                       document.getElementById(id)?.textContent.trim() === {error:?}) \
                       && document.activeElement === input \
                       && [...document.querySelectorAll('[role=status]')].some(s => s.textContent === {error:?}); }})()"
                ),
                // Todo 535: said by a live region too, while focus stays.
                &format!("{text:?} to describe the input with {error:?} and announce it"),
            )
            .await;
        }
        expect(page, &readout_is("2026-03-18"), "the held day kept").await;

        fixture.console.assert_clean("refused dates").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A digital column is a named group, so its buttons are heard with "Hours".
#[test]
fn a_digital_column_is_a_named_group() {
    block_on(async {
        let fixture = Fixture::open("/date-field/digital", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(
            page,
            "document.activeElement.getAttribute('data-slot') === 'option'",
            "Arrow Down to focus the held hour",
        )
        .await;
        let tree = e2e::ax::snapshot(page, "[role=dialog]").await.unwrap();
        for column in ["group \"Hours\"", "group \"Minutes\""] {
            assert!(tree.contains(column), "no {column} in:\n{tree}");
        }

        fixture.console.assert_clean("the digital columns").unwrap();
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
