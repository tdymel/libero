//! Todo 5638: `ThemeScope` paints its theme's surface, and a list opened inside carries it.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually};

const LIGHT: &str = "#light > [data-lsx-theme-scope=light]";
const DARK: &str = "#dark > [data-lsx-theme-scope=dark]";
/// `Theme::DARK.surface`, `#1A1B1E`.
const DARK_SURFACE: &str = "rgb(26, 27, 30)";

async fn scopes_paint_their_theme<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let dark = d.style(DARK, "background-color").await?;
    ensure!(
        dark == DARK_SURFACE,
        "the dark scope paints the dark surface: {dark}"
    );
    let light = d.style(LIGHT, "background-color").await?;
    ensure!(
        light == "rgb(255, 255, 255)",
        "the light scope paints white: {light}"
    );

    d.click(&format!("{DARK} [role=combobox]")).await?;
    eventually(d, "the portalled list sits in a dark scope", async |d| {
        d.exists("[data-lsx-theme-scope=dark] [role=listbox]").await
    })
    .await?;
    let list = "[data-lsx-theme-scope=dark] [role=listbox]";
    let ink = d.style(list, "color").await?;
    let page_ink = d.style("body", "color").await?;
    ensure!(
        ink != page_ink,
        "the list's text follows the dark ink, not the page's: {ink}"
    );
    Ok(())
}

e2e::scenario!(
    scopes_paint_their_theme_and_carry_it_into_overlays,
    "/theme-scope",
    scopes_paint_their_theme
);

const RED: &str = "rgb(255, 0, 0)";
const GREEN: &str = "rgb(0, 128, 0)";

/// Todo 5681: a probe's `Sx::dark` arm follows its scope, whichever scheme the page shows.
async fn probes_follow_their_scope<D: Driver>(d: &mut D, page: &str) -> Result<()> {
    for (probe, want) in [
        ("#probe-page", page),
        ("#probe-light", RED),
        ("#probe-dark", GREEN),
    ] {
        eventually(d, &format!("{probe} to be {want}"), async |d| {
            Ok(d.style(probe, "background-color").await? == want)
        })
        .await?;
    }
    Ok(())
}

async fn dark_arms_follow_the_scope_in_a_light_page<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    probes_follow_their_scope(d, RED).await
}

async fn dark_arms_follow_the_scope_in_a_dark_page<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    probes_follow_their_scope(d, GREEN).await
}

e2e::scenario!(
    dark_arms_follow_the_scope_in_a_light_page,
    "/theme-scope",
    dark_arms_follow_the_scope_in_a_light_page
);
e2e::scenario!(
    dark_arms_follow_the_scope_in_a_dark_page,
    "/theme-scope/dark-page",
    dark_arms_follow_the_scope_in_a_dark_page
);
