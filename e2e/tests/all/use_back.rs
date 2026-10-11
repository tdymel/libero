//! `use_back`: Android's Back runs the page's handler while it is active, a
//! menu opened later first (todo 2003).

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

const STEP: &str = "#step";
const MENU_ITEM: &str = "[role=menuitem]";

async fn step<D: Driver>(d: &mut D) -> Result<String> {
    Ok(d.text(STEP).await?.trim().to_string())
}

async fn back_steps_down_below_a_menu<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#next").await?;
    d.click("#next").await?;
    eventually(d, "two steps on", async |d| Ok(step(d).await? == "3")).await?;

    d.click("#more").await?;
    eventually(d, "the menu to open", async |d| d.exists(MENU_ITEM).await).await?;
    d.press_back().await?;
    eventually(d, "Back to close the menu", async |d| {
        Ok(!d.exists(MENU_ITEM).await?)
    })
    .await?;
    d.settle().await?;
    ensure!(
        step(d).await? == "3",
        "the Back that closed the menu also stepped down"
    );

    d.press_back().await?;
    eventually(d, "Back to step down", async |d| Ok(step(d).await? == "2")).await?;
    // Each tap banked the steps an entry (3322): no tap between the Backs.
    d.press_back().await?;
    eventually(d, "Back to step down again", async |d| {
        Ok(step(d).await? == "1")
    })
    .await?;
    ensure!(
        d.exists("#next").await?,
        "the app left while a step was left"
    );
    Ok(())
}

e2e::scenario!(
    android_back_steps_down_below_a_menu,
    "/use-back",
    back_steps_down_below_a_menu,
    android_only("2003: no Back key off Android")
);

/// Back after Back, no tap between: each Next tap earned its entry (3322).
async fn back_steps_down_twice<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#next").await?;
    d.click("#next").await?;
    eventually(d, "two steps on", async |d| Ok(step(d).await? == "3")).await?;
    d.press_back().await?;
    eventually(d, "Back to step down", async |d| Ok(step(d).await? == "2")).await?;
    d.press_back().await?;
    eventually(d, "the second Back to step down", async |d| {
        Ok(step(d).await? == "1")
    })
    .await?;
    ensure!(d.exists("#next").await?, "the app left at step 1");
    Ok(())
}

e2e::scenario!(
    android_back_steps_down_twice_without_a_tap,
    "/use-back",
    back_steps_down_twice,
    android_only("3322: no Back key off Android")
);

const DEPTH: &str = "#depth";

async fn depth<D: Driver>(d: &mut D) -> Result<String> {
    Ok(d.text(DEPTH).await?.trim().to_string())
}

async fn expect_depth<D: Driver>(d: &mut D, what: &str, want: &str) -> Result<()> {
    eventually(d, what, async |d| Ok(depth(d).await? == want)).await
}

/// The docs shell's `use_back(can_go_back, go_back)` over route depths: each Back goes one
/// route up, whether the taps came in a run or between Backs (3384).
async fn back_walks_the_routes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for _ in 0..4 {
        d.click("#deeper").await?;
    }
    expect_depth(d, "depth 4", "4").await?;
    d.press_back().await?;
    expect_depth(d, "Back to depth 3", "3").await?;
    d.press_back().await?;
    expect_depth(d, "Back to depth 2", "2").await?;

    d.click("#deeper").await?;
    expect_depth(d, "depth 3 again", "3").await?;
    for (what, want) in [("depth 2", "2"), ("depth 1", "1"), ("the start", "0")] {
        d.press_back().await?;
        expect_depth(d, &format!("Back to {what}"), want).await?;
    }
    Ok(())
}

e2e::scenario!(
    android_back_walks_the_routes,
    "/use-back/route/0",
    back_walks_the_routes,
    android_only("3384: no Back key off Android")
);
