//! `FocusTrap`: its Tab stops, a trap nested in another, and a modal dialog's
//! trap entered from the dialog itself.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

const ACTIVE_ID: &str = "document.activeElement.id";

async fn walk(page: &chromiumoxide::Page, presses: usize, backwards: bool) -> Vec<String> {
    let mut ids = Vec::new();
    for _ in 0..presses {
        match backwards {
            true => keyboard::press_shift(page, keyboard::TAB).await.unwrap(),
            false => keyboard::press(page, keyboard::TAB).await.unwrap(),
        }
        ids.push(
            page.evaluate(ACTIVE_ID)
                .await
                .unwrap()
                .into_value()
                .unwrap(),
        );
    }
    ids
}

/// A `display: none` button used to stall Tab on the stop before it, for
/// good, and `<summary>` was never a stop. A native radio group is one stop,
/// its checked radio or else its first, as the browser's own Tab has it
/// (todo 615).
#[test]
fn tab_passes_over_a_stop_that_is_not_rendered() {
    block_on(async {
        let fixture = Fixture::open("/focus-trap", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus::wait_for_focus(page, "#first", "mount")
            .await
            .unwrap();

        assert_eq!(
            walk(page, 5, false).await,
            ["r2", "s1", "summary", "last", "first"],
            "Tab from First"
        );
        assert_eq!(
            walk(page, 5, true).await,
            ["last", "summary", "s1", "r2", "first"],
            "Shift+Tab from First"
        );

        // The live state, not the markup: a click checks another radio.
        pointer::click(page, "#r3").await.unwrap();
        wait::for_js_true(page, "document.getElementById('r3').checked", "r3 checked")
            .await
            .unwrap();
        assert_eq!(walk(page, 2, true).await, ["first", "last"], "Shift+Tab");
        assert_eq!(walk(page, 2, false).await, ["first", "r3"], "Tab");
        fixture.console.assert_clean("the focus trap").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Both traps used to answer the one press: focus skipped a stop and walked
/// out into the outer trap.
#[test]
fn a_nested_trap_moves_focus_once_per_tab() {
    block_on(async {
        let fixture = Fixture::open("/focus-trap/nested", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus::wait_for_focus(page, "#outer-1", "mount")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focus::wait_for_focus(page, "#inner-1", "the inner trap mounting")
            .await
            .unwrap();

        assert_eq!(
            walk(page, 4, false).await,
            ["inner-2", "inner-3", "inner-1", "inner-2"]
        );
        assert_eq!(walk(page, 3, true).await, ["inner-1", "inner-3", "inner-2"]);
        fixture.console.assert_clean("the nested traps").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focus on the dialog itself (a click on its text) is outside the stops:
/// Shift+Tab went to the first one, not the last.
#[test]
fn shift_tab_from_the_dialog_itself_reaches_its_last_control() {
    block_on(async {
        let fixture = Fixture::open("/focus-trap/dialog", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#open-dialog", 3).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, "[role=dialog]").await.unwrap();

        pointer::click(page, "#dialog-text").await.unwrap();
        focus::assert_focused(page, "[role=dialog]", "a click on the text")
            .await
            .unwrap();
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        focus::assert_focused(page, "#rename", "Shift+Tab from the dialog")
            .await
            .unwrap();

        pointer::click(page, "#dialog-text").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::assert_focused(page, "[role=dialog] button", "Tab from the dialog")
            .await
            .unwrap();
        fixture.console.assert_clean("the dialog trap").unwrap();
        fixture.close().await.unwrap();
    });
}
