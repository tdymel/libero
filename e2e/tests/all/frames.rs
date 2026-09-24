//! Frame time (1086): the recorder proves itself, then a wheel over a `ScrollArea` is
//! measured. The measurement is report only and opt-in, outside the gate:
//! `cargo run -p e2e -- frames:: --ignored --nocapture`.

use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType,
};
use e2e::browser::block_on;
use e2e::frames;
use e2e::passes::pointer;
use e2e::{Fixture, Viewport};

/// A page that burns 50 ms in every frame must show frames of 50 ms and more.
#[test]
fn the_recorder_sees_a_page_that_burns_fifty_milliseconds_per_frame() {
    block_on(async {
        let fixture = Fixture::open("/scroll-area", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(
            "(() => { const burn = () => { const end = performance.now() + 50; \
             while (performance.now() < end) {} requestAnimationFrame(burn); }; \
             requestAnimationFrame(burn); })()",
        )
        .await
        .unwrap();
        frames::start(page).await.unwrap();
        tokio::time::sleep(Duration::from_millis(2500)).await;
        let stats = frames::stop(page).await.unwrap();
        assert!(stats.count > 0, "no frames recorded: {stats:?}");
        assert!(
            stats.max >= 50.0,
            "max {} ms under the 50 ms burn",
            stats.max
        );
        fixture.close().await.unwrap();
    });
}

/// The wheel in 40 px steps over the virtualized list, a frame apart. Run it with
/// `E2E_RELEASE=1`: the debug wasm is far slower and only compares against a debug run.
#[test]
#[ignore = "frame-time report, run on request"]
fn frame_time_of_a_scroll_area_under_the_wheel() {
    block_on(async {
        let fixture = Fixture::open("/scroll-area", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let at = pointer::centre_of(page, "#list-pane").await.unwrap();

        frames::start(page).await.unwrap();
        tokio::time::sleep(Duration::from_millis(3000)).await;
        frames::report("idle_control", frames::stop(page).await.unwrap()).unwrap();
        frames::start(page).await.unwrap();
        for _ in 0..40 {
            page.execute(
                DispatchMouseEventParams::builder()
                    .r#type(DispatchMouseEventType::MouseWheel)
                    .x(at.x)
                    .y(at.y)
                    .delta_x(0.0)
                    .delta_y(40.0)
                    .build()
                    .unwrap(),
            )
            .await
            .unwrap();
            tokio::time::sleep(Duration::from_millis(16)).await;
        }
        let stats = frames::stop(page).await.unwrap();

        frames::report("scroll_area_wheel", stats).unwrap();
        assert!(stats.count > 0, "no frames recorded: {stats:?}");
        fixture.close().await.unwrap();
    });
}

/// The same list under a touch swipe in the Android WebView, opt-in with `E2E_FRAMES=1`:
/// `E2E_RELEASE=1 E2E_FRAMES=1 cargo run -p e2e -- android frames`.
mod scroll_area_swipe {
    #[cfg(feature = "android")]
    #[test]
    fn android() {
        use e2e::driver::{Android, Driver};
        use e2e::frames;

        if std::env::var_os("E2E_FRAMES").is_none() {
            println!("frame-time report skipped: set E2E_FRAMES=1");
            return;
        }
        e2e::android::block_on(async {
            let mut driver = Android::open("/scroll-area").await.unwrap();
            let page = driver.page().clone();
            let first_row = async || -> String {
                page.evaluate("document.querySelector('#list-pane [data-row]').dataset.row")
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap()
            };
            let before = first_row().await;
            frames::start(&page).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(3000)).await;
            frames::report("android_idle_control", frames::stop(&page).await.unwrap()).unwrap();
            frames::start(&page).await.unwrap();
            // One way: drags back and forth ended where they began (todo 1163).
            for _ in 0..12 {
                driver.drag("#list-pane", 0.0, -100.0).await.unwrap();
            }
            let stats = frames::stop(&page).await.unwrap();
            frames::report("android_scroll_area_swipe", stats).unwrap();
            assert!(stats.count > 0, "no frames recorded: {stats:?}");
            let after = first_row().await;
            println!("first row {before} -> {after}");
            assert!(
                after.parse::<usize>().unwrap() > before.parse::<usize>().unwrap(),
                "the swipe did not scroll: first row {before} -> {after}"
            );
            driver.finish("frames").await.unwrap();
        });
    }
}
