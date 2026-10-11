//! A theme's `side`, `align`, `activation`, `loop_focus` and `new_tab_icon` reach the components that leave them unset.

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

/// The card's end meets the trigger's end, as the themed `align: End` says (the default is start).
async fn the_hover_card_aligns_to_the_themed_end<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.hover("#trigger").await?;
    eventually(d, "the card's end under the trigger's end", async |d| {
        if !d.exists("[role=dialog]").await?
            || d.style("[role=dialog]", "visibility").await? != "visible"
        {
            return Ok(false);
        }
        let (card, trigger) = (d.rect("[role=dialog]").await?, d.rect("#trigger").await?);
        Ok(card.width > trigger.width + 8.0
            && (card.x + card.width - (trigger.x + trigger.width)).abs() <= 1.0)
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

/// The theme's `new_tab_icon: true` draws the icon; the call site's `false` still wins.
async fn a_themed_link_draws_the_new_tab_icon<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.exists("#themed-link [data-slot=new-tab]").await?,
        "the themed link has no icon"
    );
    ensure!(
        !d.exists("#themed-link-off [data-slot=new-tab]").await?,
        "`new_tab_icon: false` drew the icon"
    );
    Ok(())
}

/// With the library's theme, no `_blank` link draws the icon (the default is off), and each still says so.
async fn an_unthemed_link_draws_no_icon_but_says_so<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    for id in ["anchor", "button", "action-icon"] {
        ensure!(
            !d.exists(&format!("#{id} [data-slot=new-tab]")).await?,
            "#{id} draws the new-tab icon"
        );
    }
    for id in ["anchor", "button"] {
        let text = d.text(&format!("#{id}")).await?;
        ensure!(
            text.contains("(opens in a new tab)"),
            "#{id} reads {text:?}"
        );
    }
    let label = d.attr("#action-icon", "aria-label").await?;
    ensure!(
        label.as_deref() == Some("Action (opens in a new tab)"),
        "the action icon is named {label:?}"
    );
    Ok(())
}

e2e::scenario!(
    an_unset_new_tab_icon_follows_the_theme,
    "/theme-defaults",
    a_themed_link_draws_the_new_tab_icon
);
e2e::scenario!(
    an_unset_new_tab_icon_is_off_by_default,
    "/theme-defaults/plain",
    an_unthemed_link_draws_no_icon_but_says_so
);
e2e::scenario!(
    an_unset_hover_card_side_follows_the_theme,
    "/theme-defaults",
    the_hover_card_opens_on_the_themed_side,
    ios: skip("iOS: a touch screen has no hover")
);
e2e::scenario!(
    an_unset_hover_card_align_follows_the_theme,
    "/theme-defaults",
    the_hover_card_aligns_to_the_themed_end,
    ios: skip("iOS: a touch screen has no hover")
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
