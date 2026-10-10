//! The WebView apps rewrite dioxus's viewport meta to `viewport-fit=cover`, so the
//! safe-area insets reach `env()`; the iOS simulator's island gives a top inset.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, Platform, eventually_text};

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
