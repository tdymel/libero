//! The sequential focus starting point (todo 622): a click on nothing focusable
//! starts the next Tab or Shift+Tab from the clicked node.

use anyhow::Result;
use e2e::driver::{Driver, eventually_focused};
use e2e::passes::keyboard::TAB;

async fn tab_from_the_click<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#late").await?;
    d.press(TAB).await?;
    eventually_focused(d, "#last", "Tab after a click on #late").await
}

async fn shift_tab_from_the_click<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#late").await?;
    d.press_shift(TAB).await?;
    eventually_focused(d, "#middle", "Shift+Tab after a click on #late").await
}

async fn prose_moves_the_start<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#first").await?;
    eventually_focused(d, "#first", "a click on #first").await?;
    d.click("#early").await?;
    d.press(TAB).await?;
    eventually_focused(d, "#middle", "Tab after a click on #early").await
}

async fn a_second_tab_carries_on<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#early").await?;
    d.press(TAB).await?;
    eventually_focused(d, "#middle", "the first Tab").await?;
    d.press(TAB).await?;
    eventually_focused(d, "#last", "the second Tab").await
}

e2e::scenario!(
    tab_after_a_click_starts_from_the_clicked_node,
    "/focus-start",
    tab_from_the_click
);
e2e::scenario!(
    shift_tab_after_a_click_starts_from_the_clicked_node,
    "/focus-start",
    shift_tab_from_the_click
);
e2e::scenario!(
    a_click_on_prose_moves_the_starting_point_away_from_a_focused_control,
    "/focus-start",
    prose_moves_the_start
);
e2e::scenario!(
    a_second_tab_carries_on_from_the_first,
    "/focus-start",
    a_second_tab_carries_on
);
