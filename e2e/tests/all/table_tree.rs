//! `Table` tree rows (1156 TR1): a row's toggle opens its children under it, says
//! so and keeps the focus; a filter keeps and opens a match's ancestors.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

const SITE: &str = "button[aria-label=\"site, 2 rows\"]";
const LIVE: &str = "table + [role=status]";

async fn opens_under_its_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "tbody tr:nth-child(2) th", "launch", "closed roots").await?;
    d.click(SITE).await?;
    eventually_text(d, "#expanded", "site", "a toggle click").await?;
    eventually_text(d, LIVE, "site expanded", "the opened row").await?;
    eventually_text(d, "tbody tr:nth-child(2) th", "design", "the children").await?;
    eventually_text(d, "tbody tr:nth-child(3) th", "build", "the children").await?;
    eventually_text(d, "tbody tr:nth-child(4) th", "launch", "the next root").await?;
    if d.attr("tbody tr:nth-child(2)", "data-level")
        .await?
        .as_deref()
        != Some("1")
    {
        bail!("a child is not on level 1");
    }
    if d.attr(SITE, "aria-expanded").await?.as_deref() != Some("true") {
        bail!("the open toggle is not expanded");
    }
    // A child's toggle sits one indent further in than its parent's.
    let parent = d.rect(SITE).await?;
    let child = d.rect("button[aria-label=\"build, 2 rows\"]").await?;
    if child.x <= parent.x {
        bail!("the child's toggle {child:?} is not indented past {parent:?}");
    }
    d.click(SITE).await?;
    eventually_text(d, "#expanded", "", "a second click").await?;
    eventually_text(d, LIVE, "site collapsed", "the closed row").await?;
    eventually_text(
        d,
        "tbody tr:nth-child(2) th",
        "launch",
        "the children to go",
    )
    .await
}

e2e::scenario!(
    a_tree_toggle_opens_its_children_under_it,
    "/table-tree",
    opens_under_its_row
);

/// Enter opens and Space closes; the focus stays on the toggle through both.
async fn keyboard_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(SITE).await?;
    eventually_focused(d, SITE, "focus").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#expanded", "site", "Enter").await?;
    eventually_focused(d, SITE, "Enter").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#expanded", "", "Space").await?;
    eventually_focused(d, SITE, "Space").await
}

e2e::scenario!(
    the_keyboard_toggles_a_tree_row_and_keeps_the_focus,
    "/table-tree",
    keyboard_toggles
);

/// "front" matches deep down: site and build show above it, opened, while the rest go.
async fn filter_shows_ancestors<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("input[type=search]").await?;
    eventually_focused(d, "input[type=search]", "a click").await?;
    d.type_text("front").await?;
    let mut seen = String::new();
    let shown = eventually(d, "the ancestors and the match", async |d| {
        seen = d.text("tbody").await?;
        Ok(seen.split_whitespace().collect::<String>() == "siteAdabuildDifrontDi")
    })
    .await;
    if let Err(error) = shown {
        bail!("{error}: the body reads {seen:?}");
    }
    // Counted by its kept children: one.
    let site = "button[aria-label=\"site, 1 row\"]";
    if d.attr(site, "aria-expanded").await?.as_deref() != Some("true") {
        bail!("a kept ancestor is not open");
    }
    Ok(())
}

e2e::scenario!(
    a_filter_shows_a_match_under_its_ancestors,
    "/table-tree",
    filter_shows_ancestors
);
