//! `ButtonGroup`: flush seams, only the outer corners round, on logical sides
//! under RTL; the focused button's ring above its neighbours; the group's
//! defaults reach the buttons, each button's own prop winning.

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, Rect, eventually_focused};
use e2e::passes::keyboard;

const TOP_LEFT: &str = "border-top-left-radius";
const TOP_RIGHT: &str = "border-top-right-radius";
const BOTTOM_LEFT: &str = "border-bottom-left-radius";

async fn round<D: Driver>(d: &mut D, selector: &str, corner: &str) -> Result<bool> {
    Ok(d.style(selector, corner).await? != "0px")
}

/// `later` overlaps `earlier` by its 1px border, on the inline end of `earlier`.
fn joined(earlier: Rect, later: Rect, rtl: bool) -> bool {
    let seam = if rtl {
        earlier.x - (later.x + later.width)
    } else {
        later.x - (earlier.x + earlier.width)
    };
    (seam + 1.0).abs() < 0.5
}

async fn seams_and_corners<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    let rtl = route.ends_with("/rtl");
    // The start side's physical corner, then the end side's.
    let (start, end) = if rtl {
        (TOP_RIGHT, TOP_LEFT)
    } else {
        (TOP_LEFT, TOP_RIGHT)
    };

    let (o1, o2, o3) = (
        d.rect("#o1").await?,
        d.rect("#o2").await?,
        d.rect("#o3").await?,
    );
    ensure!(
        joined(o1, o2, rtl) && joined(o2, o3, rtl),
        "not flush: {o1:?} {o2:?} {o3:?}"
    );
    ensure!(
        round(d, "#o1", start).await? && !round(d, "#o1", end).await?,
        "#o1 corners"
    );
    ensure!(
        !round(d, "#o2", start).await? && !round(d, "#o2", end).await?,
        "#o2 corners"
    );
    ensure!(
        !round(d, "#o3", start).await? && round(d, "#o3", end).await?,
        "#o3 corners"
    );

    // A `display: contents` wrapper and `ThemeToggle`'s pair join like bare buttons.
    let (m1, m2) = (d.rect("#m1").await?, d.rect("#m2").await?);
    ensure!(joined(m1, m2, rtl), "wrapped item not flush: {m1:?} {m2:?}");
    ensure!(
        (m1.height - m2.height).abs() < 0.5,
        "the group's size: {m1:?} {m2:?}"
    );
    ensure!(!round(d, "#m1", end).await?, "#m1 end corner");
    ensure!(
        !round(d, "#m2", start).await? && !round(d, "#m2", end).await?,
        "#m2 corners"
    );
    ensure!(
        !round(d, "#m3 > button", start).await?,
        "the theme toggle's start"
    );
    ensure!(
        round(d, "#m3 > div > button", end).await?,
        "the chevron's end"
    );

    // Filled borders are the fill: the seam gets the theme's divider.
    let (start_border, end_border) = if rtl {
        ("border-right-color", "border-left-color")
    } else {
        ("border-left-color", "border-right-color")
    };
    let divider = d.style("#f2", start_border).await?;
    ensure!(
        divider != d.style("#f2", end_border).await?,
        "no divider: {divider}"
    );
    let outlined = d.style("#o2", start_border).await?;
    ensure!(
        outlined == d.style("#o2", end_border).await?,
        "outlined seam recoloured"
    );
    Ok(())
}

async fn vertical<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (v1, v2) = (d.rect("#v1").await?, d.rect("#v2").await?);
    ensure!(
        ((v1.y + v1.height) - v2.y - 1.0).abs() < 0.5,
        "not flush: {v1:?} {v2:?}"
    );
    ensure!(
        (v1.width - v2.width).abs() < 0.5,
        "not one width: {v1:?} {v2:?}"
    );
    ensure!(
        round(d, "#v1", TOP_LEFT).await? && !round(d, "#v1", BOTTOM_LEFT).await?,
        "#v1"
    );
    ensure!(
        !round(d, "#v2", TOP_LEFT).await? && !round(d, "#v3", TOP_LEFT).await?,
        "#v2 #v3"
    );
    ensure!(round(d, "#v3", BOTTOM_LEFT).await?, "#v3 bottom");
    Ok(())
}

async fn the_ring_rises<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#o1").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#o2", "Tab").await?;
    let lifted = d.style("#o2", "z-index").await?;
    ensure!(lifted == "1", "the focused button's z-index: {lifted}");
    Ok(())
}

async fn disabled_defaults<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        d.attr("#d1", "disabled").await?.is_some(),
        "#d1 took the group's disabled"
    );
    ensure!(
        d.attr("#d2", "disabled").await?.is_none(),
        "#d2's own `disabled: false` wins"
    );
    Ok(())
}

async fn pair_leads<D: Driver>(d: &mut D, route: &str) -> Result<()> {
    let (start, end) = if route.ends_with("/rtl") {
        ("border-right-color", "border-left-color")
    } else {
        ("border-left-color", "border-right-color")
    };
    let chevron = "#p1 > div > button";
    let (at, away) = (d.style(chevron, start).await?, d.style(chevron, end).await?);
    ensure!(at != away, "chevron seam colour {at} on both sides");
    ensure!(
        d.style("#p2", start).await? != d.style("#p2", end).await?,
        "no divider after the pair"
    );
    Ok(())
}

async fn hidden_child<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        !round(d, "#h2", TOP_LEFT).await?,
        "a hidden first child no longer squares its neighbour: update the docs"
    );
    ensure!(
        round(d, "#h3", TOP_LEFT).await? && round(d, "#h3", TOP_RIGHT).await?,
        "a lone visible child is round"
    );
    Ok(())
}

e2e::scenario!(
    a_leading_theme_toggle_pair_has_a_divider_at_its_chevron,
    "/button-group",
    pair_leads
);
e2e::scenario!(
    a_hidden_child_still_counts_as_first,
    "/button-group",
    hidden_child
);
e2e::scenario!(
    the_seams_are_flush_and_only_the_ends_round,
    "/button-group",
    seams_and_corners
);
e2e::scenario!(
    under_rtl_the_seams_and_corners_mirror,
    "/button-group/rtl",
    seams_and_corners
);
e2e::scenario!(
    a_vertical_group_joins_top_to_bottom,
    "/button-group",
    vertical
);
e2e::scenario!(
    the_focused_button_is_lifted_over_its_neighbours,
    "/button-group",
    the_ring_rises
);
e2e::scenario!(
    a_button_s_own_disabled_beats_the_group_s,
    "/button-group",
    disabled_defaults
);

#[test]
fn it_meets_the_baseline() {
    Suite::new("button_group", "/button-group")
        .focusable("#o2")
        .focusable("#v2")
        .targets("#o3")
        .dark_snapshot("the theme toggle names the next scheme, which follows the platform's")
        .run();
}
