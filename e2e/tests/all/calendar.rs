//! `DatePicker`'s day grid: the arrows, Home/End and PageUp/PageDown, and a
//! month change that keeps focus in the grid (todo 406).
//!
//! `/calendar` picks Wednesday 2026-03-18, which is also `today`, so the grid
//! and its tab stop never depend on the clock. Weeks start on Monday.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::{Fixture, Suite, Viewport, wait};

const STOP: &str = "[role=grid] [data-date='2026-03-18']:not([data-outside])";

#[test]
fn it_meets_the_baseline() {
    Suite::new("calendar", "/calendar")
        .focusable(STOP)
        .targets("[role=grid] [data-slot=day]")
        .run();
}

/// Every key the grid owns, each read back from where focus lands. A focus
/// that stays on the old cell, or lands on the neighbour month's copy of the
/// day (`data-outside`), fails the step that caused it.
#[test]
fn the_keys_move_focus_through_the_grid() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/calendar", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, STOP, 10).await.unwrap();
            expect_focus(page, "2026-03-18", "March 2026", "Tab into the grid", at).await;

            let steps: &[(Key, &str, &str, &str)] = &[
                (
                    keyboard::ARROW_RIGHT,
                    "2026-03-19",
                    "March 2026",
                    "ArrowRight",
                ),
                (
                    keyboard::ARROW_LEFT,
                    "2026-03-18",
                    "March 2026",
                    "ArrowLeft",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "2026-03-25",
                    "March 2026",
                    "ArrowDown",
                ),
                (keyboard::ARROW_UP, "2026-03-18", "March 2026", "ArrowUp"),
                (keyboard::HOME, "2026-03-16", "March 2026", "Home (Monday)"),
                (keyboard::END, "2026-03-22", "March 2026", "End (Sunday)"),
                (keyboard::PAGE_DOWN, "2026-04-22", "April 2026", "PageDown"),
                (keyboard::PAGE_UP, "2026-03-22", "March 2026", "PageUp"),
                (
                    keyboard::PAGE_UP,
                    "2026-02-22",
                    "February 2026",
                    "PageUp again",
                ),
                // Mar 1 is drawn in February's grid as an outside day; the
                // step must page to March and focus March's own cell.
                (
                    keyboard::ARROW_DOWN,
                    "2026-03-01",
                    "March 2026",
                    "ArrowDown off the month",
                ),
                (
                    keyboard::ARROW_LEFT,
                    "2026-02-28",
                    "February 2026",
                    "ArrowLeft off the month",
                ),
            ];
            for (key, date, month, what) in steps {
                keyboard::press(page, *key).await.unwrap();
                expect_focus(page, date, month, what, at).await;
            }

            keyboard::press_shift(page, keyboard::PAGE_DOWN)
                .await
                .unwrap();
            expect_focus(
                page,
                "2027-02-28",
                "February 2027",
                "Shift+PageDown (a year)",
                at,
            )
            .await;

            fixture
                .console
                .assert_clean(&format!("the calendar keys at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Waits until focus is on `date`'s own cell in the grid titled `month`.
async fn expect_focus(page: &chromiumoxide::Page, date: &str, month: &str, what: &str, at: &str) {
    let check = format!(
        "(() => {{ const el = document.activeElement; \
         const grid = el && el.closest('[role=grid]'); \
         return !!grid && grid.getAttribute('aria-label') === {month:?} \
           && el.getAttribute('data-date') === {date:?} && !el.hasAttribute('data-outside'); }})()"
    );
    if let Err(e) =
        wait::for_js_true(page, &check, &format!("{what} to focus {date} in {month}")).await
    {
        let actual: String = page
            .evaluate(
                "(() => { const el = document.activeElement; const grid = el && el.closest('[role=grid]'); \
                 return `${el && el.getAttribute('data-date')} in ${grid && grid.getAttribute('aria-label')}`; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        panic!("at {at}: {e}; focus is on {actual}");
    }
}
