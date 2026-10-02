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
