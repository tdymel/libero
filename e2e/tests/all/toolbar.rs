//! `Toolbar`: the `RovingTabindex` archetype over mixed items, plus vertical and RTL bars.

use anyhow::Result;
use e2e::archetypes::{Orientation, RovingTabindex};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport};

const ITEMS: &str = "[role=toolbar] [data-toolbar-item]";
const FONT: &str = "[role=toolbar] [data-toolbar-item][aria-label=Font]";

/// Every item in turn, the disabled `Undo` included, then a wrap and both ends.
async fn the_arrows_rove<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#bold").await?;
    for (key, name, to) in [
        (keyboard::ARROW_RIGHT, "ArrowRight", "#italic"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#left"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#right"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#undo"),
        (keyboard::ARROW_RIGHT, "ArrowRight", FONT),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#bold"),
        (keyboard::ARROW_LEFT, "ArrowLeft", FONT),
        (keyboard::ARROW_LEFT, "ArrowLeft", "#undo"),
        (keyboard::HOME, "Home", "#bold"),
        (keyboard::END, "End", FONT),
    ] {
        d.press(key).await?;
        eventually_focused(d, to, name).await?;
    }
    Ok(())
}

/// `Select` keeps its own Home, which opens its list (APG select-only).
async fn an_item_keeps_the_keys_it_takes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(FONT).await?;
    d.press(keyboard::HOME).await?;
    eventually_focused(d, FONT, "Home on the select").await?;
    eventually(d, "Home to open the list", async |d| {
        Ok(d.attr(FONT, "aria-expanded").await?.as_deref() == Some("true"))
    })
    .await
}

/// Tabbing back in lands on the item focused last, not the first.
async fn the_tab_stop_follows_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#bold").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, "#italic", "ArrowRight").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "Tab").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "#italic", "Shift+Tab").await?;

    // A click moves the stop too.
    d.click("#right").await?;
    eventually_focused(d, "#right", "a click").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "Tab after a click").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "#right", "Shift+Tab after a click").await
}

/// Up and Down move, stopping at the ends (`loop_focus: false`); Left and Right belong to the items.
async fn a_vertical_bar_moves_on_up_and_down<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#pen").await?;
    for (key, name, to) in [
        (keyboard::ARROW_UP, "ArrowUp at the top", "#pen"),
        (keyboard::ARROW_DOWN, "ArrowDown", "#eraser"),
        (keyboard::ARROW_DOWN, "ArrowDown", "#fill"),
        (keyboard::ARROW_DOWN, "ArrowDown at the bottom", "#fill"),
        (keyboard::ARROW_UP, "ArrowUp", "#eraser"),
        (keyboard::ARROW_RIGHT, "ArrowRight", "#eraser"),
    ] {
        d.press(key).await?;
        eventually_focused(d, to, name).await?;
    }
    Ok(())
}

/// Under RTL the next item is on the left.
async fn rtl_swaps_the_arrows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#one").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    eventually_focused(d, "#two", "ArrowLeft").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, "#one", "ArrowRight").await
}

e2e::scenario!(
    the_arrows_rove_over_every_kind_of_item,
    "/toolbar",
    the_arrows_rove
);
e2e::scenario!(
    a_select_keeps_its_home_key,
    "/toolbar",
    an_item_keeps_the_keys_it_takes
);
e2e::scenario!(
    tabbing_back_in_returns_to_the_last_item,
    "/toolbar",
    the_tab_stop_follows_focus
);
e2e::scenario!(
    a_vertical_toolbar_moves_on_up_and_down,
    "/toolbar-vertical",
    a_vertical_bar_moves_on_up_and_down
);
e2e::scenario!(
    a_right_to_left_toolbar_swaps_the_arrows,
    "/toolbar-rtl",
    rtl_swaps_the_arrows
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("toolbar", "/toolbar")
        .focusable("#bold")
        // `Select`'s trigger sits inside its taller frame, which takes the press.
        .targets("[role=toolbar] button[data-toolbar-item]")
        .run();
}

#[test]
fn it_honours_the_roving_tabindex_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            for (route, orientation, wraps) in [
                ("/toolbar", Orientation::Horizontal, true),
                ("/toolbar-vertical", Orientation::Vertical, false),
            ] {
                let fixture = Fixture::open(route, viewport).await.unwrap();

                RovingTabindex {
                    items: ITEMS,
                    orientation,
                    wraps,
                }
                .assert_contract(&fixture.page)
                .await
                .unwrap_or_else(|e| panic!("{route} at {}: {e}", viewport.name()));

                fixture
                    .console
                    .assert_clean(&format!("{route} contract at {}", viewport.name()))
                    .unwrap();
                fixture.close().await.unwrap();
            }
        }
    });
}
