//! `ShortcutHelp`: a row stacks its description below the chord in a narrow
//! list, and keeps both on one line in a wide one (1224).

use anyhow::{Result, ensure};
use e2e::driver::Driver;

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
