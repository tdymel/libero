//! `NumberField`: the steppers and the arrow keys step one value.
//!
//! The steppers are a scope that skips the render a press causes (todo 29), so
//! a second press must still step from the value the first one set.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "input[role=spinbutton]";
const PLUS: &str = "button[aria-label=Increase]";
const MINUS: &str = "button[aria-label=Decrease]";

#[test]
fn the_steppers_and_the_arrows_step_from_the_current_value() {
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        click(page, PLUS).await.unwrap();
        settle(page, "4").await.unwrap();
        click(page, PLUS).await.unwrap();
        settle(page, "5").await.unwrap();

        keyboard::tab_to(page, INPUT, 3).await.unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        settle(page, "6").await.unwrap();

        click(page, MINUS).await.unwrap();
        settle(page, "5").await.unwrap();

        fixture
            .console
            .assert_clean("stepping a number field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A real pointer click at the centre of `selector`.
async fn click(page: &Page, selector: &str) -> Result<()> {
    let (x, y): (f64, f64) = page
        .evaluate(format!(
            "(() => {{ const r = document.querySelector({selector:?}).getBoundingClientRect(); \
             return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
        ))
        .await?
        .into_value()?;
    let at = pointer::Point { x, y };
    pointer::drag(page, at, at, 1).await
}

/// The field shows `value` and announces it.
async fn settle(page: &Page, value: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const el = document.querySelector({INPUT:?}); \
             return el.value === {value:?} && el.getAttribute('aria-valuenow') === {value:?}; }})()"
        ),
        &format!("the field to show {value}"),
    )
    .await
}
