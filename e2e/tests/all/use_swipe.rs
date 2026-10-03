//! `use_swipe`: a touch swipe reports its direction once, a short one and a mouse
//! drag do not. `use_edge_swipe`: only an inward swipe from the band past the
//! system back zone opens, mirrored under RTL, inside a scrolling `ScrollArea`.

use anyhow::Result;
use e2e::driver::{Driver, eventually};

async fn text_is<D: Driver>(d: &mut D, selector: &str, expected: &str) -> Result<()> {
    eventually(d, &format!("{selector} to read {expected:?}"), async |d| {
        Ok(d.text(selector).await? == expected)
    })
    .await
}

async fn pad_centre<D: Driver>(d: &mut D) -> Result<(f64, f64)> {
    let pad = d.rect("#pad").await?;
    Ok((pad.x + pad.width / 2.0, pad.y + pad.height / 2.0))
}

async fn reports_the_direction<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (x, y) = pad_centre(d).await?;
    // Short of the 48px distance: nothing.
    d.swipe_from(x, y, 30.0, 0.0).await?;
    d.settle().await?;
    text_is(d, "#last", "none").await?;
    d.swipe_from(x, y, 100.0, 10.0).await?;
    text_is(d, "#last", "Right").await?;
    d.swipe_from(x, y, 0.0, -80.0).await?;
    text_is(d, "#last", "Up").await
}

async fn ignores_a_mouse<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.drag("#pad", 120.0, 0.0).await?;
    d.settle().await?;
    text_is(d, "#last", "none").await
}

/// Where the default band (44..92px in) is, from the swipe's edge.
const IN_BAND: f64 = 70.0;
const IN_BACK_ZONE: f64 = 24.0;

async fn opens_from_the_band<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (_, vh) = d.viewport().await?;
    let y = vh / 2.0;
    // The system's back zone and the wrong way: none opens.
    d.swipe_from(IN_BACK_ZONE, y, 120.0, 0.0).await?;
    d.swipe_from(IN_BAND + 120.0, y, -120.0, 0.0).await?;
    d.settle().await?;
    text_is(d, "#opens", "0").await?;
    d.swipe_from(IN_BAND, y, 120.0, 0.0).await?;
    text_is(d, "#opens", "1").await?;
    // Up the page from the band scrolls, it does not open.
    d.swipe_from(IN_BAND, y, 0.0, -120.0).await?;
    d.settle().await?;
    text_is(d, "#opens", "1").await
}

async fn opens_from_the_right_under_rtl<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (vw, vh) = d.viewport().await?;
    let y = vh / 2.0;
    d.swipe_from(IN_BAND, y, 120.0, 0.0).await?;
    d.settle().await?;
    text_is(d, "#opens", "0").await?;
    d.swipe_from(vw - IN_BAND, y, -120.0, 0.0).await?;
    text_is(d, "#opens", "1").await
}

e2e::scenario!(
    a_touch_swipe_reports_its_direction_past_the_distance,
    "/use-swipe/basic",
    reports_the_direction,
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    a_mouse_drag_is_never_a_swipe,
    "/use-swipe/basic",
    ignores_a_mouse,
    android: skip("adb drags with a finger")
);
e2e::scenario!(
    an_edge_swipe_opens_only_from_the_band_past_the_back_zone,
    "/use-swipe/edge",
    opens_from_the_band,
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    an_edge_swipe_opens_from_the_right_under_rtl,
    "/use-swipe/edge-rtl",
    opens_from_the_right_under_rtl,
    desktop: skip("1126: no touch input under Xvfb")
);
