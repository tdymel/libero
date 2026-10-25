//! `NumberField`: steppers and arrows step one value. The steppers skip a press's render
//! (29), so a second press must step from the value the first set.

use anyhow::Result;
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_text, linger};
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, wait};

const INPUT: &str = "#quantity";
const PLUS: &str = "button[aria-label=Increase]";
const MINUS: &str = "button[aria-label=Decrease]";
const SPIN: &str = "input[role=spinbutton]";

async fn the_steppers_step<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(PLUS).await?;
    eventually_text(d, "#echo", "4", "one Increase").await?;
    d.click(PLUS).await?;
    eventually_text(d, "#echo", "5", "two Increases").await?;
    eventually(d, "aria-valuenow 5", async |d| {
        Ok(d.attr(SPIN, "aria-valuenow").await?.as_deref() == Some("5"))
    })
    .await?;
    d.click(MINUS).await?;
    eventually_text(d, "#echo", "4", "Decrease").await
}

async fn the_arrows_step<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(SPIN).await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, "#echo", "4", "ArrowUp").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, "#echo", "2", "two ArrowDowns").await
}

async fn typing_commits<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(SPIN).await?;
    d.press(keyboard::END).await?;
    d.press(keyboard::BACKSPACE).await?;
    d.type_text("42").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#echo", "42", "typing 42, Enter").await
}

/// Todo 3390: grouped while not focused, the bare number while typing.
async fn grouped_until_focused<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const AMOUNT: &str = "#amount";
    eventually(d, "grouped at rest", async |d| {
        Ok(d.value(AMOUNT).await? == "1,234,567.5")
    })
    .await?;
    d.click(AMOUNT).await?;
    eventually(d, "bare while focused", async |d| {
        Ok(d.value(AMOUNT).await? == "1234567.5")
    })
    .await?;
    d.press(keyboard::TAB).await?;
    eventually(d, "grouped again once left", async |d| {
        Ok(d.value(AMOUNT).await? == "1,234,567.5")
    })
    .await
}

e2e::scenario!(
    group_digits_groups_until_focused,
    "/number-field/grouped",
    grouped_until_focused
);
e2e::scenario!(
    the_steppers_step_a_number_field,
    "/number-field/echo",
    the_steppers_step
);
e2e::scenario!(
    the_arrows_step_a_focused_number_field,
    "/number-field/echo",
    the_arrows_step
);
e2e::scenario!(
    typing_a_number_commits_it,
    "/number-field/echo",
    typing_commits
);

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
        page.evaluate_bounded(format!("document.querySelector({INPUT:?}).focus()"))
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
            .evaluate_bounded(format!(
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

/// Todo 3033: an arrow the IME is composing picks a candidate, so it does not step the
/// value; the plain arrow after it does.
#[test]
fn composing_arrows_do_not_step_the_value() {
    block_on(async {
        let fixture = Fixture::open("/number-field/echo", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, SPIN).await.unwrap();
        keyboard::press_composing(page, SPIN, &["ArrowUp", "ArrowDown", "PageUp"])
            .await
            .unwrap();
        crate::settle::painted(page).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#echo').textContent === '3'",
            "the value kept through composing arrows",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#echo').textContent === '4'",
            "the plain ArrowUp",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("composing arrows").unwrap();
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

/// Clamping per keystroke turned the "2" of "25" into the floor 10. Out-of-range text
/// clamps on leaving the field.
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
        page.evaluate_bounded("document.activeElement.select()")
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

        // Todo 3847: a float field reverts `NaN`, which `str::parse` accepts but no caller wants.
        keyboard::tab_to(page, "#weight", 10).await.unwrap();
        retype(page, "NaN").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const el = document.querySelector('#weight'); \
             return el.value === '1.5' && el.getAttribute('aria-valuenow') === '1.5'; })()",
            "the weight to revert from NaN",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("emptying a number").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2327: a form reset drops text the field still shows for `None`, such as a lone
/// minus, which would come back when the caller clears the field again.
#[test]
fn a_form_reset_drops_an_unparsed_minus() {
    block_on(async {
        let fixture = Fixture::open("/number-field/reset", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let value = format!("document.querySelector({INPUT:?}).value");

        keyboard::tab_to(page, INPUT, 3).await.unwrap();
        keyboard::type_text(page, "-").await.unwrap();
        wait::for_js_true(page, &format!("{value} === '-'"), "the typed minus")
            .await
            .unwrap();
        // By script: a real click would blur the field, and its change commit drops the text.
        page.evaluate_bounded("document.querySelector('#reset').click()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{value} === ''"),
            "the reset to drop the minus",
        )
        .await
        .unwrap();
        pointer::click(page, "#set").await.unwrap();
        wait::for_js_true(page, &format!("{value} === '5'"), "the caller's 5")
            .await
            .unwrap();
        pointer::click(page, "#clear").await.unwrap();
        wait::for_js_true(
            page,
            &format!("{value} === ''"),
            "the cleared field, no minus",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("a reset number field")
            .unwrap();
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
    page.evaluate_bounded("document.activeElement.select()")
        .await?;
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
        .evaluate_bounded(format!(
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

/// Focuses the grouped amount, by a press at `1,23`'s width or by script from a caret at 4, and
/// waits for the bare number with a caret for which the JS test `caret` holds.
fn caret_after_focus(press: bool, caret: &str) {
    const AMOUNT: &str = "#amount";
    block_on(async {
        let fixture = Fixture::open("/number-field/grouped", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("document.querySelector({AMOUNT:?}).value === '1,234,567.5'"),
            "the grouped number",
        )
        .await
        .unwrap();
        let (x, y): (f64, f64) = page
            .evaluate_bounded(format!(
                "(() => {{ const el = document.querySelector({AMOUNT:?}); \
                 const style = getComputedStyle(el), r = el.getBoundingClientRect(); \
                 const ctx = document.createElement('canvas').getContext('2d'); \
                 ctx.font = style.font; \
                 const left = r.x + parseFloat(style.borderLeftWidth) + parseFloat(style.paddingLeft); \
                 return [left + ctx.measureText('1,23').width, r.y + r.height / 2]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        if press {
            pointer::click_at(page, pointer::Point { x, y })
                .await
                .unwrap();
        } else {
            page.evaluate_bounded(format!(
                "(el => {{ el.setSelectionRange(4, 4); el.focus(); }})(document.querySelector({AMOUNT:?}))"
            ))
            .await
            .unwrap();
        }
        // The caret settles a task after the bare text, so wait on both.
        let settled = wait::for_js_true(
            page,
            &format!(
                "(el => el.value === '1234567.5' && {caret})(document.querySelector({AMOUNT:?}))"
            ),
            "the bare number with the caret in place",
        )
        .await;
        if let Err(error) = settled {
            let state: (String, Option<u32>, Option<u32>) = page
                .evaluate_bounded(format!(
                    "(el => [el.value, el.selectionStart, el.selectionEnd])(document.querySelector({AMOUNT:?}))"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            panic!("{error:#}: value, selection start and end are {state:?}");
        }
        fixture
            .console
            .assert_clean("a grouped number's focus")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3430: a caret set before the focus stays between the 3 and the 4 of `1,234,567.5` once
/// the separators go, not one digit right per separator.
#[test]
fn a_focus_in_a_grouped_number_keeps_the_caret_between_its_digits() {
    caret_after_focus(false, "el.selectionStart === 3 && el.selectionEnd === 3");
}

/// A press places its own caret after the rewrite, near the 3 and the 4: left alone.
#[test]
fn a_press_in_a_grouped_number_keeps_the_caret_it_placed() {
    caret_after_focus(
        true,
        "el.selectionStart >= 2 && el.selectionStart <= 4 && el.selectionEnd === el.selectionStart",
    );
}

/// Todo 2608: under `dir=rtl` a negative number still reads `-5`, not `5-` (UAX #9).
async fn a_negative_number_reads_left_to_right<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let direction = d.style("#delta", "direction").await?;
    anyhow::ensure!(direction == "ltr", "the input runs {direction}");
    let value = d.value("#delta").await?;
    anyhow::ensure!(value == "-5", "the input holds {value:?}");
    Ok(())
}

e2e::scenario!(
    a_negative_number_reads_left_to_right_under_rtl,
    "/number-field/rtl",
    a_negative_number_reads_left_to_right
);

const PRICE: &str = "#price";

/// `Decimals::Exactly(2)` pads the resting `0.2` to `0.20`.
async fn decimals_pad_at_rest<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "0.20 at rest", async |d| {
        Ok(d.value(PRICE).await? == "0.20")
    })
    .await
}

e2e::scenario!(
    decimals_pad_the_resting_value,
    "/number-field/decimals",
    decimals_pad_at_rest
);

/// A stepper press or ArrowUp from 0.2 holds 0.3 and 0.4 exactly, with no float drift.
async fn a_step_keeps_two_places<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(PLUS).await?;
    eventually_text(d, "#readout", "Some(0.3)", "one stepper press").await?;
    eventually(d, "0.30 shown", async |d| {
        Ok(d.value(PRICE).await? == "0.30")
    })
    .await?;
    d.click(SPIN).await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, "#readout", "Some(0.4)", "ArrowUp").await?;
    eventually(d, "0.40 shown", async |d| {
        Ok(d.value(PRICE).await? == "0.40")
    })
    .await
}

e2e::scenario!(
    a_step_keeps_the_decimal_places,
    "/number-field/decimals",
    a_step_keeps_two_places
);

/// Digits typed past the places are cut as they arrive: `1.239` stays `1.23`.
async fn typing_past_the_places_is_cut<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(SPIN).await?;
    d.press(keyboard::END).await?;
    for _ in 0..6 {
        d.press(keyboard::BACKSPACE).await?;
    }
    d.type_text("1.239").await?;
    if let Err(error) = eventually(d, "1.23 in the field", async |d| {
        Ok(d.value(PRICE).await? == "1.23")
    })
    .await
    {
        let shown = d.value(PRICE).await?;
        let readout = d.text("#readout").await?;
        anyhow::bail!("{error:#}: the field reads {shown:?}, the readout {readout:?}");
    }
    Ok(())
}

e2e::scenario!(
    typing_past_the_places_is_cut_off,
    "/number-field/decimals",
    typing_past_the_places_is_cut
);

/// The leading `$` is text that describes the field: its id is in `aria-describedby`.
async fn the_leading_slot_describes_the_field<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let described = d.attr(PRICE, "aria-describedby").await?.unwrap_or_default();
    let mut texts = Vec::new();
    for id in described.split_whitespace() {
        texts.push(d.text(&format!("[id={id:?}]")).await?);
    }
    anyhow::ensure!(
        texts.iter().any(|text| text.trim() == "$"),
        "aria-describedby {described:?} reads {texts:?}, not the $"
    );
    Ok(())
}

e2e::scenario!(
    the_leading_slot_describes_the_field_by_id,
    "/number-field/decimals",
    the_leading_slot_describes_the_field
);

const SPINNER: &str = "[role=dialog]";
const COLUMN: &str = "[role=dialog] [data-slot=spin]";
const MINUTES: &str = "#minutes";

/// Todo 4133: a mouse types in a `TouchInput::Spin` field; only a finger spins.
async fn a_mouse_types_in_a_spin_field<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(MINUTES).await?;
    let mode = d.attr(MINUTES, "inputmode").await?;
    anyhow::ensure!(mode.as_deref() == Some("numeric"), "inputmode {mode:?}");
    anyhow::ensure!(!d.exists(SPINNER).await?, "a click opened the spinner");
    d.press(keyboard::END).await?;
    d.press(keyboard::BACKSPACE).await?;
    d.press(keyboard::BACKSPACE).await?;
    d.type_text("42").await?;
    d.press(keyboard::ENTER).await?;
    // `3`, empty, `4`, `42`: one change each.
    eventually_text(d, "#spun", "Some(42) 4", "typing 42, Enter").await
}

e2e::scenario!(
    a_mouse_still_types_in_a_spin_field,
    "/number-field/spin",
    a_mouse_types_in_a_spin_field,
    android: skip("a phone app's tap is a finger's: it spins (4133)"),
    ios: skip("a phone app's tap is a finger's: it spins (4133)")
);

/// Todo 4133: in the app a tap spins with the keyboard down, Back closes the sheet, and the
/// keyboard button hands the field back for a tap that raises the keyboard.
async fn a_tap_spins_in_the_app<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const KEYBOARD: &str = "[role=dialog] [data-slot=actions] button:first-child";
    d.click(MINUTES).await?;
    eventually(d, "the spinner after a tap", async |d| {
        d.exists(COLUMN).await
    })
    .await?;
    linger(d, 40).await;
    anyhow::ensure!(
        !d.soft_keyboard_shown().await?,
        "the tap raised the keyboard"
    );
    d.press_back().await?;
    eventually(d, "Back closed the spinner", async |d| {
        Ok(!d.exists(SPINNER).await?)
    })
    .await?;
    d.click(MINUTES).await?;
    eventually(d, "the spinner again", async |d| d.exists(KEYBOARD).await).await?;
    d.click(KEYBOARD).await?;
    eventually(d, "the field focused, the spinner closed", async |d| {
        Ok(d.is_focused(MINUTES).await? && !d.exists(SPINNER).await?)
    })
    .await?;
    // A `focus()` from Rust raises no WebView keyboard (3902): the next tap on the field does.
    linger(d, 40).await;
    anyhow::ensure!(
        !d.soft_keyboard_shown().await?,
        "the keyboard button raised the keyboard itself: the docs' Touch input section says it does not"
    );
    d.click(MINUTES).await?;
    eventually(d, "the soft keyboard on the next tap", async |d| {
        d.soft_keyboard_shown().await
    })
    .await?;
    anyhow::ensure!(
        !d.exists(SPINNER).await?,
        "a tap while typing opened the spinner"
    );
    // Back hides the keyboard first, so the next scenario starts without it.
    d.press_back().await?;
    eventually(d, "Back to hide the keyboard", async |d| {
        Ok(!d.soft_keyboard_shown().await?)
    })
    .await
}

e2e::scenario!(
    android_a_tap_spins_with_the_keyboard_down,
    "/number-field/spin",
    a_tap_spins_in_the_app,
    android_only("only a phone app reports its soft keyboard")
);

/// A phone, 360px wide, on the spin page.
async fn spin_phone() -> Fixture {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    let fixture = Fixture::open("/number-field/spin", Viewport::Mobile)
        .await
        .unwrap();
    fixture
        .page
        .execute_bounded(SetDeviceMetricsOverrideParams::new(360, 780, 1.0, true))
        .await
        .unwrap();
    fixture
}

async fn tap(page: &Page, selector: &str) -> Result<()> {
    let at = pointer::touch_down(page, selector).await?;
    pointer::touch_up(page, at).await
}

async fn spun(page: &Page, text: &str) -> Result<()> {
    let js = format!("document.querySelector('#spun').textContent === {text:?}");
    wait::for_js_true(page, &js, text).await
}

/// Opens the spinner with a tap: no keyboard, the column focused, the sheet at the foot.
async fn tap_open(page: &Page) -> Result<()> {
    tap(page, MINUTES).await?;
    let js = format!(
        "(() => {{ const i = document.querySelector({MINUTES:?}); \
         const c = document.querySelector({COLUMN:?}); const s = document.querySelector({SPINNER:?}); \
         if (!c || getComputedStyle(s).visibility !== 'visible') return false; \
         const r = s.getBoundingClientRect(); \
         return i.getAttribute('inputmode') === 'none' && c === document.activeElement \
           && Math.abs(r.bottom - innerHeight) < 1 && Math.abs(r.width - innerWidth) < 1; }})()"
    );
    wait::for_js_true(page, &js, "the sheet, its column focused, no keyboard").await
}

const CLOSED: &str = "!document.querySelector('[role=dialog]') \
    && document.querySelector('#minutes') === document.activeElement";

/// A tap opens the sheet without the keyboard; a drag of 3 steps sends 3 changes; Escape and
/// Done close it and hand focus back to the input.
#[test]
fn a_finger_spins_the_number() {
    block_on(async {
        let fixture = spin_phone().await;
        let page = &fixture.page;
        tap_open(page).await.unwrap();
        let named = format!(
            "(() => {{ const c = document.querySelector({COLUMN:?}); \
             return c.getAttribute('aria-valuenow') === '30' \
               && document.getElementById(c.getAttribute('aria-labelledby')).textContent === 'Break' \
               && document.getElementById(c.getAttribute('aria-describedby')).textContent.trim() === 'min'; }})()"
        );
        wait::for_js_true(page, &named, "a named column at 30, described by its unit")
            .await
            .unwrap();
        // Up 3 x 24px in small moves: each step is sent once.
        let at = pointer::touch_down(page, &format!("{COLUMN} [data-slot=value]"))
            .await
            .unwrap();
        for step in 1..=12 {
            let y = at.y - 6.0 * f64::from(step);
            pointer::touch_move(page, pointer::Point { x: at.x, y })
                .await
                .unwrap();
        }
        let end = pointer::Point {
            x: at.x,
            y: at.y - 72.0,
        };
        pointer::touch_up(page, end).await.unwrap();
        spun(page, "Some(45) 3").await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_js_true(page, CLOSED, "Escape closed it, focus on the input")
            .await
            .unwrap();

        tap_open(page).await.unwrap();
        tap(page, &format!("{COLUMN} [data-slot=neighbour]:last-child"))
            .await
            .unwrap();
        spun(page, "Some(50) 4").await.unwrap();
        tap(page, "[role=dialog] [data-slot=actions] button:last-child")
            .await
            .unwrap();
        wait::for_js_true(page, CLOSED, "Done closed it, focus on the input")
            .await
            .unwrap();
        spun(page, "Some(50) 4").await.unwrap();
        fixture.console.assert_clean("a finger's spinner").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The keyboard button closes the sheet and hands the input its keypad back, for typing.
#[test]
fn the_keyboard_button_switches_to_typing() {
    block_on(async {
        let fixture = spin_phone().await;
        let page = &fixture.page;
        tap_open(page).await.unwrap();
        tap(page, "[role=dialog] [data-slot=actions] button:first-child")
            .await
            .unwrap();
        let typing = format!(
            "{CLOSED} && document.querySelector({MINUTES:?}).getAttribute('inputmode') === 'numeric'"
        );
        wait::for_js_true(page, &typing, "the input focused with its keypad")
            .await
            .unwrap();
        // A tap now types: it opens nothing.
        tap(page, MINUTES).await.unwrap();
        page.evaluate(format!("document.querySelector({MINUTES:?}).select()"))
            .await
            .unwrap();
        keyboard::insert_text(page, "42").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        spun(page, "Some(42) 1").await.unwrap();
        wait::for_js_true(page, CLOSED, "no spinner while typing")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("typing in a spinner field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
