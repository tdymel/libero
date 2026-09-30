//! `ShortcutHelp`: a row stacks its description below the chord in a narrow
//! list, and keeps both on one line in a wide one (1224). A long list scrolls
//! inside a capped height (1444), a named tab stop only while it does (1615).

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

/// Whether the column's first description starts below its chord.
async fn stacked<D: Driver>(d: &mut D, column: &str) -> Result<bool> {
    let chord = d.rect(&format!("{column} dt")).await?;
    let description = d.rect(&format!("{column} dd")).await?;
    Ok(description.y >= chord.y + chord.height - 1.0)
}

async fn rows_stack_only_when_narrow<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(
        stacked(d, "#narrow").await?,
        "the narrow list keeps chord and description side by side"
    );
    ensure!(!stacked(d, "#wide").await?, "the wide list stacks its rows");
    Ok(())
}

e2e::scenario!(
    a_narrow_list_stacks_each_row,
    "/shortcut-help",
    rows_stack_only_when_narrow,
    native: skip("1224: Blitz drops `@container`, so the rows stay side by side")
);

/// The scroll box around the long list, a named region once it overflows (1615).
const LONG_REGION: &str = "#long [role=region]";

async fn a_long_list_scrolls_inside<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the long list's region", async |d| {
        d.exists(LONG_REGION).await
    })
    .await?;
    let (_, height) = d.viewport().await?;
    let list = d.rect(LONG_REGION).await?;
    let last = d.rect("#long dd:last-child").await?;
    ensure!(
        list.height <= height * 0.6 + 1.0,
        "the list is {}px tall in a {height}px window",
        list.height
    );
    ensure!(
        last.y > list.y + list.height,
        "the last row is not below the list's capped height"
    );
    let overflow = d.style(LONG_REGION, "overflow-y").await?;
    ensure!(overflow == "auto", "the list's overflow-y is {overflow}");
    let tabindex = d.attr(LONG_REGION, "tabindex").await?;
    ensure!(
        tabindex.as_deref() == Some("0"),
        "the list takes no focus to scroll by keyboard"
    );
    let name = d.attr(LONG_REGION, "aria-label").await?;
    ensure!(
        name.as_deref() == Some("Long"),
        "the region is named {name:?}"
    );
    for short in ["#narrow", "#wide"] {
        ensure!(
            !d.exists(&format!("{short} [tabindex='0']")).await?,
            "the {short} list scrolls nothing but is a tab stop"
        );
        ensure!(
            !d.exists(&format!("{short} [role=region]")).await?,
            "the {short} list scrolls nothing but is a region"
        );
    }
    Ok(())
}

e2e::scenario!(
    a_long_list_scrolls,
    "/shortcut-help",
    a_long_list_scrolls_inside
);
