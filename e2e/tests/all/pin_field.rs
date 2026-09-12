//! `PinField`: its label names the cells' group, so a click on it focuses the
//! first cell (todo 483); modifier chords are the browser's (todo 509).

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const CELL: &str = "[role=group] input";

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
