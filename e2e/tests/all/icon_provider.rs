//! `IconProvider` (1094): the select under it draws the provided chevron, the one
//! beside it keeps lucide's, and a new `icons` value redraws in place.

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually};

const BAR: &str = "#provided svg path[data-glyph=bar]";
const ARROW: &str = "#provided svg path[data-glyph=arrow]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("icon_provider", "/icon-provider")
        .focusable("#swap")
        .run();
}

async fn one_slot_swapped<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(d.exists(BAR).await?, "the provided select draws the bar");
    ensure!(
        !d.exists("#default svg path[data-glyph]").await?,
        "the other select keeps lucide's chevron"
    );
    ensure!(d.exists("#default svg path").await?, "the default chevron");
    d.click("#swap").await?;
    eventually(d, "the new glyph", async |d| {
        Ok(d.exists(ARROW).await? && !d.exists(BAR).await?)
    })
    .await
}

e2e::scenario!(
    one_slot_swaps_and_redraws_on_a_new_set,
    "/icon-provider",
    one_slot_swapped
);

async fn whole_set_drawn<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let tabler = d.attr("#tabler svg path", "d").await?;
    let lucide = d.attr("#default svg path", "d").await?;
    ensure!(
        tabler.is_some(),
        "the select under the tabler set draws a chevron"
    );
    ensure!(
        tabler != lucide,
        "the tabler chevron differs from lucide's: {tabler:?} vs {lucide:?}"
    );
    Ok(())
}

e2e::scenario!(
    a_whole_set_redraws_its_subtree,
    "/icon-provider",
    whole_set_drawn
);
