//! The WebView apps rewrite dioxus's viewport meta to `viewport-fit=cover`, so the
//! safe-area insets reach `env()`; the iOS simulator's island gives a top inset. A small
//! field keeps the page unzoomed.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};

async fn covers_the_notch<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#inset-top", "inset", "mount").await?;
    let meta = d
        .evaluate("document.querySelector('meta[name=viewport]')?.content ?? ''")
        .await?;
    let content = meta.as_str().unwrap_or_default();
    ensure!(
        content.contains("viewport-fit=cover"),
        "viewport meta: {content:?}"
    );
    if d.platform() == Platform::Ios {
        let inset = d
            .evaluate(
                "parseFloat(getComputedStyle(document.querySelector('#inset-top')).paddingTop)",
            )
            .await?;
        ensure!(
            inset.as_f64().is_some_and(|px| px > 0.0),
            "top safe-area inset: {inset}"
        );
    }
    Ok(())
}

e2e::scenario!(
    the_viewport_covers_the_notch,
    "/viewport-meta",
    covers_the_notch,
    web: skip("the web build carries its app's own index.html"),
    native: skip("Blitz has no viewport meta")
);

/// Todo 3194 (G12): a tap into a field under 16px leaves the page at scale 1; iOS zooms in
/// unless the viewport's `maximum-scale=1` stops it.
async fn small_field_keeps_the_scale<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the small field", async |d| {
        d.exists("#small-field").await
    })
    .await?;
    let size = d.style("#small-field", "font-size").await?;
    d.click("#small-field").await?;
    eventually_focused(d, "#small-field", "a tap").await?;
    d.frame().await?;
    let scale = d.evaluate("visualViewport.scale").await?;
    ensure!(
        scale.as_f64() == Some(1.0),
        "a field of {size} zoomed the page to {scale}"
    );
    Ok(())
}

e2e::scenario!(
    a_small_field_keeps_the_page_scale,
    "/viewport-meta",
    small_field_keeps_the_scale,
    web: skip("the web build carries its app's own index.html"),
    native: skip("Blitz has no visual viewport")
);
