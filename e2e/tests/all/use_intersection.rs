//! `use_in_viewport` and `use_intersection` against a real `IntersectionObserver`:
//! scrolling a target into view and out again.

use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_text, linger};
use e2e::{Fixture, Viewport, wait};

async fn js<T: serde::de::DeserializeOwned>(page: &chromiumoxide::Page, expression: &str) -> T {
    page.evaluate(expression)
        .await
        .unwrap_or_else(|e| panic!("evaluate {expression}: {e}"))
        .into_value()
        .unwrap_or_else(|e| panic!("read {expression}: {e}"))
}

async fn state_reads(page: &chromiumoxide::Page, expected: &str) {
    wait::for_js_true(
        page,
        &format!("document.getElementById('state').textContent === '{expected}'"),
        &format!("#state to read {expected}"),
    )
    .await
    .unwrap();
}

/// Scrolls by a quarter screen per poll (the target must stay up for a few polls) until `#state` reads `expected`.
/// On the web it stops scrolling once the target is in (`dy` > 0) or out of the viewport.
async fn scroll_until<D: Driver>(d: &mut D, expected: &str, dy: f64) -> Result<()> {
    let (_, height) = d.viewport().await?;
    let web = d.platform() == Platform::Web;
    eventually(d, &format!("#state to read {expected}"), async |d| {
        if d.text("#state").await? == expected {
            return Ok(true);
        }
        if web {
            let target = d.rect("#target").await?;
            let shown = target.y < height && target.y + target.height > 0.0;
            if shown != (dy > 0.0) {
                d.scroll_by(dy * height / 4.0).await?;
            }
            return Ok(false);
        }
        d.scroll_by(dy * height / 4.0).await?;
        // The observer reports after a frame; a scroll that does not wait overshoots it.
        linger(d, 8).await;
        Ok(false)
    })
    .await
}

async fn in_view_only_while_scrolled_to<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#state", "out", "the first measure").await?;
    // A WebView finds the target by its own attribute beside the handle's (1255).
    if matches!(d.platform(), Platform::Desktop | Platform::Android) {
        for name in ["data-lsx-intersect", "data-lsx-observe"] {
            if d.attr("#target", name).await?.is_none() {
                bail!("{:?}: the target lost its {name}", d.platform());
            }
        }
    }
    scroll_until(d, "in", 1.0).await?;
    scroll_until(d, "out", -1.0).await
}

e2e::scenario!(
    a_target_is_in_the_viewport_only_while_scrolled_to,
    "/use-intersection/viewport",
    in_view_only_while_scrolled_to,
    native: skip("Blitz has no IntersectionObserver")
);

async fn a_clipped_target_stays_out<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#state", "shown in, hidden out", "the first measure").await?;
    d.frame().await?;
    eventually_text(d, "#state", "shown in, hidden out", "a clipped target").await
}

e2e::scenario!(
    a_target_clipped_by_a_scroller_is_not_in_the_viewport,
    "/use-intersection/clipped",
    a_clipped_target_stays_out,
    native: skip("Blitz has no IntersectionObserver")
);

e2e::scenario!(
    a_root_clips_a_target_that_the_viewport_shows,
    "/use-intersection/root-clip",
    a_clipped_target_stays_out,
    native: skip("Blitz has no IntersectionObserver")
);

async fn keeps_the_first_sighting<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#state", "unseen", "the first measure").await?;
    scroll_until(d, "seen", 1.0).await?;
    // The control is the viewport scenario: there the same scroll flips it back.
    let (_, height) = d.viewport().await?;
    d.scroll_by(-height * 2.0).await?;
    d.frame().await?;
    eventually_text(d, "#state", "seen", "scrolling away").await
}

e2e::scenario!(
    once_keeps_the_first_sighting,
    "/use-intersection/once",
    keeps_the_first_sighting,
    native: skip("Blitz has no IntersectionObserver")
);

#[test]
fn a_scroller_is_the_root_and_the_ratio_follows_the_threshold() {
    block_on(async {
        let fixture = Fixture::open("/use-intersection/root", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        state_reads(page, "0").await;
        // The target sits 300px down a 100px scroller: all of it shows at 280.
        let _: bool = js(
            page,
            "(document.getElementById('scroller').scrollTop = 280, true)",
        )
        .await;
        state_reads(page, "100").await;
        let _: bool = js(
            page,
            "(document.getElementById('scroller').scrollTop = 0, true)",
        )
        .await;
        state_reads(page, "0").await;

        fixture.close().await.unwrap();
    });
}
