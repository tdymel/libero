//! `PinField`: its label names the cells' group, so a click on it focuses the
//! first cell (todo 483); modifier chords are the browser's (todo 509); what a
//! cell shows is the pin, however the character arrived (todo 449).

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::input::InsertTextParams;
use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, ax, wait};

const CELL: &str = "[role=group] input";
/// The cells' text joined by `|`, and the focused cell's index.
const CELLS: &str = "(() => { const c = [...document.querySelectorAll('[role=group] input')]; \
    return c.map(i => i.value).join('|') + ' @' + c.indexOf(document.activeElement); })()";

#[test]
fn a_click_on_the_label_focuses_the_first_cell() {
    crate::select::label_click_focuses("/pin-field", CELL);
}

/// Ctrl/Alt/Meta with an arrow, Home or End leave the focus on its cell; the
/// plain arrow still moves it.
#[test]
fn modifier_chords_leave_the_focus_on_its_cell() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let cell =
            "[...document.querySelectorAll('[role=group] input')].indexOf(document.activeElement)";
        page.evaluate("document.querySelectorAll('[role=group] input')[1].focus()")
            .await
            .unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_LEFT,
                keyboard::ARROW_RIGHT,
                keyboard::HOME,
                keyboard::END,
            ],
            cell,
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{cell} === 2"),
            "ArrowRight to the next cell",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("pin field chords").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focuses cell `index` with the caret at `caret`.
async fn focus_at(page: &Page, index: usize, caret: usize) {
    page.evaluate(format!(
        "(() => {{ const i = document.querySelectorAll('[role=group] input')[{index}]; \
         i.focus(); i.setSelectionRange({caret}, {caret}); }})()"
    ))
    .await
    .unwrap();
}

/// A digit key, pressed as a keyboard does: `keydown` first, then the text.
async fn press_digit(page: &Page, digit: &'static str) {
    let vk = 48 + digit.parse::<i64>().unwrap();
    let code = [
        "Digit0", "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8",
        "Digit9",
    ][(vk - 48) as usize];
    let key = keyboard::Key {
        key: digit,
        code,
        vk,
        text: Some(digit),
    };
    keyboard::press(page, key).await.unwrap();
}

async fn expect_cells(page: &Page, expected: &str, what: &str) {
    wait::for_js_true(
        page,
        &format!("{CELLS} === {}", serde_json::to_string(expected).unwrap()),
        what,
    )
    .await
    .unwrap_or_else(|e| {
        panic!("{what}: {e}");
    });
}

/// A click on a cell's left half puts the caret before its digit: typing there
/// replaces the digit, and the cell holds one character.
#[test]
fn typing_before_a_digit_replaces_it() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1234").await.unwrap();
        expect_cells(page, "1|2|3|4 @3", "1234 typed").await;

        focus_at(page, 1, 0).await;
        press_digit(page, "7").await;
        expect_cells(page, "1|7|3|4 @2", "7 typed before the 2").await;

        fixture.console.assert_clean("pin field retype").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A paste into a filled cell replaces from that cell on, whichever side of
/// its digit the caret was.
#[test]
fn a_paste_into_a_filled_cell_replaces_from_there() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1234").await.unwrap();
        expect_cells(page, "1|2|3|4 @3", "1234 typed").await;

        focus_at(page, 1, 1).await;
        page.execute(InsertTextParams::new("98")).await.unwrap();
        expect_cells(page, "1|9|8|4 @3", "98 pasted after the 2").await;

        focus_at(page, 1, 0).await;
        page.execute(InsertTextParams::new("56")).await.unwrap();
        expect_cells(page, "1|5|6|4 @3", "56 pasted before the 9").await;

        fixture.console.assert_clean("pin field paste").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Holes are not representable: a character typed past the pin lands in the
/// first empty cell, which is what the cells show, and focus moves after it.
#[test]
fn typing_past_the_pin_lands_in_the_first_empty_cell() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1").await.unwrap();
        expect_cells(page, "1||| @1", "1 typed").await;

        focus_at(page, 3, 0).await;
        press_digit(page, "5").await;
        expect_cells(page, "1|5|| @2", "5 typed in the last cell").await;

        fixture
            .console
            .assert_clean("pin field past the end")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Text with no key named, as an Android soft keyboard sends it: a cell whose
/// value the edit leaves unchanged still shows the pin (todo 590).
#[test]
fn text_without_a_key_leaves_no_drifted_cell() {
    block_on(async {
        let fixture = Fixture::open("/pin-field", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus_at(page, 0, 0).await;
        keyboard::type_text(page, "1").await.unwrap();
        expect_cells(page, "1||| @1", "1 typed").await;

        focus_at(page, 3, 0).await;
        page.execute(InsertTextParams::new("5")).await.unwrap();
        expect_cells(page, "1|5|| @2", "5 inserted in the last cell").await;

        focus_at(page, 0, 1).await;
        page.execute(InsertTextParams::new("1")).await.unwrap();
        expect_cells(page, "1|5|| @1", "1 inserted after the 1").await;

        fixture.console.assert_clean("pin field no key").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `aria-required` is not allowed on a group (axe `aria-allowed-attr`), so
/// the cells carry it, and each cell reports the error too.
#[test]
fn every_cell_reports_required_and_invalid() {
    block_on(async {
        let fixture = Fixture::open("/pin-field/error", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        let tree = ax::snapshot(page, "[role=group]").await.unwrap();
        assert_eq!(
            tree.matches("textbox [invalid] [required]").count(),
            4,
            "every cell required and invalid:\n{tree}"
        );
        let on_group: bool = page
            .evaluate("document.querySelector('[role=group]').hasAttribute('aria-required')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!on_group, "aria-required on the group");

        fixture.console.assert_clean("pin field error").unwrap();
        fixture.close().await.unwrap();
    });
}
