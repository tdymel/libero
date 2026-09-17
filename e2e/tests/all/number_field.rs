//! `NumberField`: the steppers and the arrow keys step one value.
//!
//! The steppers are a scope that skips the render a press causes (todo 29), so
//! a second press must still step from the value the first one set.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const INPUT: &str = "#quantity";
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

        // A click moves Tab's starting point to the stepper, so focus by script.
        page.evaluate(format!("document.querySelector({INPUT:?}).focus()"))
            .await
            .unwrap();
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

/// A stepper press leaves the focus on the field, as a native spinner does, so
/// the arrow keys keep working and the field is not left mid-edit.
#[test]
fn a_stepper_press_keeps_the_focus_on_the_field() {
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 3).await.unwrap();
        click(page, PLUS).await.unwrap();
        settle(page, "4").await.unwrap();
        let focused: bool = page
            .evaluate(format!(
                "document.activeElement === document.querySelector({INPUT:?})"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(focused, "the stepper press took the focus off the field");
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        settle(page, "5").await.unwrap();

        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("number_field", "/number-field")
        .focusable("#quantity")
        .focusable("#ranged")
        .focusable("#readonly")
        .run();
}

/// Clamping each keystroke turned the "2" of "25" into the floor 10, so no
/// number above a two-digit floor could be typed. Out of range text waits for
/// the field to be left, then clamps.
#[test]
fn typing_clamps_when_the_field_is_left_not_per_keystroke() {
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        retype(page, "25").await.unwrap();
        held(page, RANGED, "25").await.unwrap();

        retype(page, "5").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        held(page, RANGED, "10").await.unwrap();

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        retype(page, "150").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        held(page, RANGED, "99").await.unwrap();

        fixture
            .console
            .assert_clean("typing a ranged number")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// APG's spinbutton: Page Up and Page Down take a larger step, ten steps here,
/// clamped like any other.
#[test]
fn page_up_and_page_down_take_ten_steps() {
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        keyboard::press(page, keyboard::PAGE_UP).await.unwrap();
        held(page, RANGED, "60").await.unwrap();
        keyboard::press(page, keyboard::PAGE_DOWN).await.unwrap();
        keyboard::press(page, keyboard::PAGE_DOWN).await.unwrap();
        held(page, RANGED, "40").await.unwrap();
        for _ in 0..7 {
            keyboard::press(page, keyboard::PAGE_UP).await.unwrap();
        }
        held(page, RANGED, "99").await.unwrap();

        fixture.console.assert_clean("paging a number").unwrap();
        fixture.close().await.unwrap();
    });
}

const RANGED: &str = "#ranged";

/// A step from typed text still out of range starts at that text, as a native
/// spinner's does, not at the last number the caller took ("15" of "150").
#[test]
fn a_step_starts_from_the_typed_text() {
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        retype(page, "150").await.unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        held(page, RANGED, "99").await.unwrap();

        retype(page, "5").await.unwrap();
        click(page, &format!("[data-frame]:has({RANGED}) {PLUS}"))
            .await
            .unwrap();
        held(page, RANGED, "10").await.unwrap();

        fixture.console.assert_clean("stepping typed text").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 504: an emptied field hands its caller `None`, and text that never
/// parsed reverts on commit, so text, value and `aria-valuenow` stay together.
#[test]
fn emptying_the_field_clears_the_value_and_junk_reverts() {
    const BACKSPACE: keyboard::Key = keyboard::Key {
        key: "Backspace",
        code: "Backspace",
        vk: 8,
        text: None,
    };
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        page.evaluate("document.activeElement.select()")
            .await
            .unwrap();
        keyboard::press(page, BACKSPACE).await.unwrap();
        held(page, RANGED, "").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        held(page, RANGED, "").await.unwrap();

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        retype(page, "42").await.unwrap();
        held(page, RANGED, "42").await.unwrap();
        retype(page, "-").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        held(page, RANGED, "42").await.unwrap();

        fixture.console.assert_clean("emptying a number").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 509: Ctrl/Alt/Meta with an arrow or Page key is the caret's or the
/// browser's (Ctrl+PageDown switches tabs), not a step.
#[test]
fn modifier_chords_do_not_step() {
    block_on(async {
        let fixture = Fixture::open("/number-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, RANGED, 6).await.unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_UP,
                keyboard::ARROW_DOWN,
                keyboard::PAGE_UP,
                keyboard::PAGE_DOWN,
            ],
            &format!("document.querySelector({RANGED:?}).value"),
        )
        .await
        .unwrap();

        fixture.console.assert_clean("number field chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Select the field's text and type `text` over it.
async fn retype(page: &Page, text: &str) -> Result<()> {
    page.evaluate("document.activeElement.select()").await?;
    keyboard::type_text(page, text).await
}

/// The field and its caller agree on `value`; `""` is the empty field.
async fn held(page: &Page, selector: &str, value: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const el = document.querySelector({selector:?}); \
             return el.value === {value:?} && (el.getAttribute('aria-valuenow') ?? '') === {value:?} \
             && document.querySelector('#held').dataset.value === {value:?}; }})()"
        ),
        &format!("{selector} to hold {value}"),
    )
    .await
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
