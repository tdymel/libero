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

/// The wheel in 40 px steps over the virtualized list, a frame apart. The fixtures' debug wasm
/// takes about a second per step, so the numbers only compare against a run of the same build.
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
