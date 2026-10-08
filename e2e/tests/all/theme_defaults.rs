//! A theme's `side`, `activation` and `loop_focus` reach the components that leave them unset.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;

const SELECTED: &str = "[role=tab][aria-selected=true]";
const SECOND_TAB: &str = "[role=tablist] > [role=tab]:nth-child(2)";

/// The card opens above its trigger, as the themed `side: Top` says (the default is below).
async fn the_hover_card_opens_on_the_themed_side<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover("#trigger").await?;
    eventually(d, "the card above the trigger", async |d| {
        if !d.exists("[role=dialog]").await?
            || d.style("[role=dialog]", "visibility").await? != "visible"
        {
            return Ok(false);
        }
        let (card, trigger) = (d.rect("[role=dialog]").await?, d.rect("#trigger").await?);
        Ok(card.y + card.height <= trigger.y + 1.0)
    })
    .await
}

/// Arrows only move the focus, as the themed `activation: Manual` says (the default selects).
async fn the_tabs_activate_manually<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SELECTED).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, SECOND_TAB, "ArrowRight").await?;
    ensure!(
        d.text(SELECTED).await? == "Account",
        "the arrow selected the tab it moved to"
    );
    Ok(())
}

/// The bar stops at its first item, as the themed `loop_focus: false` says (the default wraps).
async fn the_toolbar_stops_at_its_ends<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#one").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, "#two", "ArrowRight after a stop at the start").await
}

e2e::scenario!(
    an_unset_hover_card_side_follows_the_theme,
    "/theme-defaults",
    the_hover_card_opens_on_the_themed_side
);
e2e::scenario!(
    an_unset_tabs_activation_follows_the_theme,
    "/theme-defaults",
    the_tabs_activate_manually
);
e2e::scenario!(
    an_unset_toolbar_loop_focus_follows_the_theme,
    "/theme-defaults",
    the_toolbar_stops_at_its_ends
);
