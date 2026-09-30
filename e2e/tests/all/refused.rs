//! Presses and Tab stops the web refuses (todo 734): a disabled button takes no
//! click, an `inert` subtree neither a click nor focus. Blitz gave both.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually_focused};
use e2e::passes::keyboard::TAB;

async fn a_disabled_button<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#off-label").await?;
    d.click("#off-in-card").await?;
    d.settle().await?;
    ensure!(
        d.text("#clicks").await? == "0",
        "a disabled button's handler ran"
    );
    Ok(())
}

async fn an_inert_button<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#asleep-button").await?;
    d.settle().await?;
    ensure!(
        d.text("#clicks").await? == "0",
        "an inert button's handler ran"
    );
    // Chromium focuses the holder above the inert box, not nothing.
    let owner = d.focused_id().await?;
    ensure!(
        owner == "sleep-card",
        "{:?}: focus on {owner:?}",
        d.platform()
    );
    Ok(())
}

async fn focus_after_a_press<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#first").await?;
    eventually_focused(d, "#first", "a click on #first").await?;
    d.click("#off-in-card").await?;
    d.settle().await?;
    let in_card = d.focused_id().await?;
    d.click("#first").await?;
    d.click("#off").await?;
    d.settle().await?;
    let off = d.focused_id().await?;
    // A touch on a disabled control sends no mouse events, so focus stays; a
    // plain `<button disabled>` does the same on the WebView (990).
    let holder = match d.platform() {
        Platform::Android => "first",
        _ => "card",
    };
    ensure!(
        in_card == holder && off.is_empty(),
        "{:?}: in card {in_card:?}, bare {off:?}",
        d.platform()
    );
    Ok(())
}

async fn tab_past_them<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#first").await?;
    d.press(TAB).await?;
    eventually_focused(d, "#last", "Tab from #first").await?;
    d.press_shift(TAB).await?;
    eventually_focused(d, "#first", "Shift+Tab from #last").await
}

e2e::scenario!(
    a_disabled_button_takes_no_click,
    "/refused",
    a_disabled_button
);
e2e::scenario!(
    an_inert_subtree_takes_no_click_and_no_focus,
    "/refused",
    an_inert_button
);
e2e::scenario!(
    a_press_on_a_disabled_button_focuses_what_holds_it,
    "/refused",
    focus_after_a_press
);
e2e::scenario!(
    tab_skips_disabled_buttons_and_an_inert_subtree,
    "/refused",
    tab_past_them,
    android: skip("958: element identity on the WebView")
);
