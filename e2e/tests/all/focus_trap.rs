//! `FocusTrap`: its Tab stops, a trap nested in another, and a modal dialog's
//! trap entered from the dialog itself.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_focused};
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

/// The focused id after each of `presses` Tabs, or Shift+Tabs.
async fn walk<D: Driver>(d: &mut D, presses: usize, backwards: bool) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for _ in 0..presses {
        match backwards {
            true => d.press_shift(keyboard::TAB).await?,
            false => d.press(keyboard::TAB).await?,
        }
        ids.push(d.focused_id().await?);
    }
    Ok(ids)
}

/// A trap mounted with the page focuses its first stop (todo 664). A
/// `display: none` button used to stall Tab on the stop before it, for good,
/// and `<summary>` was never a stop. A native radio group is one stop, its
/// checked radio or else its first, as the browser's own Tab has it (todo 615).
async fn tab_passes_over_hidden_stops<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_focused(d, "#first", "mount").await?;
    let walked = walk(d, 5, false).await?;
    ensure!(
        walked == ["r2", "s1", "summary", "last", "first"],
        "Tab from First: {walked:?}"
    );
    let walked = walk(d, 5, true).await?;
    ensure!(
        walked == ["last", "summary", "s1", "r2", "first"],
        "Shift+Tab from First: {walked:?}"
    );

    // The live state, not the markup: a click checks another radio.
    d.click("#r3").await?;
    eventually_focused(d, "#r3", "a click on r3").await?;
    let walked = walk(d, 2, true).await?;
    ensure!(walked == ["first", "last"], "Shift+Tab from r3: {walked:?}");
    let walked = walk(d, 2, false).await?;
    ensure!(walked == ["first", "r3"], "Tab from last: {walked:?}");
    Ok(())
}

e2e::scenario!(
    tab_passes_over_a_stop_that_is_not_rendered,
    "/focus-trap",
    tab_passes_over_hidden_stops
);

/// The radio groups alone, in a trap a click opens.
async fn a_radio_group_is_one_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#open").await?;
    eventually_focused(d, "#first", "opening the trap").await?;
    let walked = walk(d, 4, false).await?;
    ensure!(
        walked == ["r2", "s1", "last", "first"],
        "Tab from First: {walked:?}"
    );
    Ok(())
}

e2e::scenario!(
    a_radio_group_is_one_tab_stop,
    "/focus-trap/radios",
    a_radio_group_is_one_stop
);

/// Both traps used to answer the one press: focus skipped a stop and walked
/// out into the outer trap.
async fn a_nested_trap_moves_focus_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_focused(d, "#outer-1", "mount").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, "#inner-1", "the inner trap mounting").await?;
    let walked = walk(d, 4, false).await?;
    ensure!(
        walked == ["inner-2", "inner-3", "inner-1", "inner-2"],
        "Tab: {walked:?}"
    );
    let walked = walk(d, 3, true).await?;
    ensure!(
        walked == ["inner-1", "inner-3", "inner-2"],
        "Shift+Tab: {walked:?}"
    );
    Ok(())
}

e2e::scenario!(
    a_nested_trap_moves_focus_once_per_tab,
    "/focus-trap/nested",
    a_nested_trap_moves_focus_once
);

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
