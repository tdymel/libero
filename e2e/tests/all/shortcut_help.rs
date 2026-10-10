//! `ShortcutHelp`: a row stacks its description below the chord in a narrow
//! list, and keeps both on one line in a wide one (1224). A long list scrolls
//! inside a capped height (1444), a named tab stop only while it does (1615).

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

/// Whether each of the column's descriptions starts below its chord, one entry per row.
async fn stacked<D: Driver>(d: &mut D, column: &str) -> Result<Vec<bool>> {
    let chords = d.rects(&format!("{column} dt")).await?;
    let descriptions = d.rects(&format!("{column} dd")).await?;
    ensure!(
        !chords.is_empty() && chords.len() == descriptions.len(),
        "{column} has {} chords and {} descriptions",
        chords.len(),
        descriptions.len()
    );
    Ok(chords
        .iter()
        .zip(&descriptions)
        .map(|(chord, description)| description.y >= chord.y + chord.height - 1.0)
        .collect())
}

async fn rows_stack_only_when_narrow<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let narrow = stacked(d, "#narrow").await?;
    ensure!(
        narrow.iter().all(|stacked| *stacked),
        "the narrow list keeps a chord and description side by side: {narrow:?}"
    );
    let wide = stacked(d, "#wide").await?;
    ensure!(
        wide.iter().all(|stacked| !stacked),
        "the wide list stacks a row: {wide:?}"
    );
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
