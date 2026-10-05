//! `FocusTrap`: its Tab stops, a trap nested in another, and a modal dialog's
//! trap entered from the dialog itself.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_focused};
use e2e::passes::{focus, keyboard, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const OPEN: &str = "#open-dialog";

/// The modal page: the stops pages are bare native controls.
#[test]
fn it_meets_the_baseline() {
    Suite::new("focus_trap", "/focus-trap/dialog")
        .focusable(OPEN)
        .contrast_covers("[role=dialog]")
        .state(
            "open",
            &[Step::TabTo(OPEN), Step::Press(keyboard::ENTER)],
            "[role=dialog]",
        )
        .run();
}

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

/// A trap mounted with the page focuses its first stop (664); a `display: none` button once
/// stalled Tab. A radio group is one stop, its checked or first radio (615).
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
    tab_passes_over_hidden_stops,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
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
    a_radio_group_is_one_stop,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

/// A hidden `[data-autofocus]` target took the search with it: focus stayed on Open (2299).
async fn a_hidden_autofocus_falls_through<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#open").await?;
    eventually_focused(d, "#first", "opening the trap").await?;
    Ok(())
}

e2e::scenario!(
    an_autofocus_target_that_takes_no_focus_falls_through,
    "/focus-trap/autofocus",
    a_hidden_autofocus_falls_through,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

const SWITCH: &str = "[aria-label='activate focus trap']";

/// The docs demo's exit: Escape and Release each switch the trap off and hand
/// focus back to the switch that mounted it (1528).
async fn the_trap_lets_go<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SWITCH).await?;
    d.press(keyboard::SPACE).await?;
    eventually_focused(d, "#first", "the switch mounting the trap").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually_focused(d, SWITCH, "Escape in the trap").await?;
    ensure!(!d.exists("#first").await?, "Escape left the trap mounted");

    d.press(keyboard::SPACE).await?;
    eventually_focused(d, "#first", "the switch mounting the trap again").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#release", "Tab to Release").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, SWITCH, "Release").await?;
    ensure!(!d.exists("#first").await?, "Release left the trap mounted");
    Ok(())
}

e2e::scenario!(
    escape_or_release_hands_focus_back_to_the_switch,
    "/focus-trap/exit",
    the_trap_lets_go,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
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
    a_nested_trap_moves_focus_once,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);

/// Focus that leaves without a Tab (2297): a click on text outside comes back, a blur that
/// keeps the element (a window switch) moves nothing, and a Tab after the focused stop is
/// removed stays in.
#[test]
fn focus_that_falls_out_of_the_trap_comes_back() {
    block_on(async {
        let fixture = Fixture::open("/focus-trap/leaving", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        focus::wait_for_focus(page, "#first", "mount")
            .await
            .unwrap();

        pointer::click(page, "#middle").await.unwrap();
        focus::wait_for_focus(page, "#middle", "a click on Middle")
            .await
            .unwrap();
        pointer::click(page, "#outside").await.unwrap();
        focus::wait_for_focus(page, "#middle", "a click on the text outside")
            .await
            .unwrap();
        // From the text, Tab would reach First: from Middle it reaches Remove.
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, "#remove", "Tab after the click outside")
            .await
            .unwrap();
        keyboard::press_shift(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, "#middle", "Shift+Tab")
            .await
            .unwrap();

        page.evaluate(
            "document.activeElement.dispatchEvent(new FocusEvent('focusout', { bubbles: true }))",
        )
        .await
        .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        focus::wait_for_focus(page, "#remove", "Tab after a blur that kept Middle")
            .await
            .unwrap();

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(page, "!document.querySelector('#remove')", "Remove gone")
            .await
            .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "['first', 'middle'].includes(document.activeElement.id)",
            "Tab after the removal to stay in the trap",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("focus leaving the trap")
            .unwrap();
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
