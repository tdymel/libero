//! `use_swipe`: a touch swipe reports its direction once, a short one and a mouse
//! drag do not. `use_edge_swipe`: only an inward swipe from the band past the
//! system back zone opens, mirrored under RTL, inside a scrolling `ScrollArea`.

use anyhow::Result;
use e2e::driver::{Driver, Platform, eventually};

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

/// Under Android's gesture navigation the back zone is the system's: a swipe there goes
/// Back and closes the app (2133), so only three-button runs and the web try it.
fn system_takes_the_back_zone<D: Driver>(d: &D) -> bool {
    d.platform() == Platform::Android
        && std::env::var("E2E_ANDROID_NAV").is_ok_and(|nav| nav == "gestural")
}

async fn opens_from_the_band<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (_, vh) = d.viewport().await?;
    let y = vh / 2.0;
    // The system's back zone and the wrong way: none opens.
    if !system_takes_the_back_zone(d) {
        d.swipe_from(IN_BACK_ZONE, y, 120.0, 0.0).await?;
    }
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

/// The drawer opens mid-press and the page turns inert, so the press ends off it:
/// every later swipe still opens (todo 2170).
async fn reopens_after_each_close<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (_, vh) = d.viewport().await?;
    let scroller = d.rect("#scroller").await?;
    for round in 1..=5 {
        // Every other round starts on an inner scroller, which takes the pointer (2170).
        let y = if round % 2 == 0 {
            scroller.y + scroller.height / 2.0
        } else {
            vh * 0.75
        };
        d.swipe_from(IN_BAND, y, 120.0, 0.0).await?;
        text_is(d, "#opens", &round.to_string()).await?;
        // One tap right after the swipe closes it (todo 2189).
        d.click("#close").await?;
        eventually(d, "the drawer to close", async |d| {
            Ok(!d.exists("#drawer").await?)
        })
        .await?;
    }
    Ok(())
}

/// Todo 2190: in a fling Chromium sends a touch's first move uncancelable, an inner
/// scroller pans and the pointer is cancelled; the touch still opens the drawer.
mod an_edge_swipe_opens_while_the_page_flings {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        use e2e::driver::{Android, Driver};
        use e2e::wait;

        e2e::android::block_on(async {
            let mut driver = Android::open("/use-swipe/drawer").await.unwrap();
            let page = driver.page().clone();
            // The code box the whole page long: the swipe starts on it wherever the fling stops.
            page.evaluate(
                "document.querySelector('#scroller').style.height = '2800px';
                 window.__cancels = 0; addEventListener('pointercancel', () => __cancels++, true)",
            )
            .await
            .unwrap();
            let (vw, vh) = driver.viewport().await.unwrap();
            driver
                .fling_from(vw / 2.0, vh * 0.85, 0.0, -vh * 0.35)
                .await
                .unwrap();
            // The fling's own press was cancelled too: only the swipe's counts.
            page.evaluate("__cancels = 0").await.unwrap();
            driver
                .swipe_from(super::IN_BAND, vh / 2.0, 120.0, 0.0)
                .await
                .unwrap();
            wait::for_js_true(
                &page,
                "document.querySelector('#opens').textContent === '1'",
                "the drawer to open",
            )
            .await
            .unwrap();
            let cancels: u32 = page
                .evaluate("__cancels")
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                cancels > 0,
                "the fling ended before the swipe: no pointer was cancelled"
            );
            driver.finish("use_swipe").await.unwrap();
        });
    }
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
e2e::scenario!(
    an_edge_swipe_reopens_a_drawer_five_times,
    "/use-swipe/drawer",
    reopens_after_each_close,
    desktop: skip("1126: no touch input under Xvfb")
);
