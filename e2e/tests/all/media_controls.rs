//! Where media plays, `Video` and `Audio` draw their controls and never the unsupported
//! fallback (todo 5230); its native twin is `native/media_controls.rs`.

use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const PAGE: &str = "/media-controls";

#[test]
fn a_playing_video_keeps_its_picture_and_draws_no_fallback() {
    block_on(async {
        let fixture = Fixture::open(PAGE, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelector('#video [data-slot=seek] [role=slider]:not([aria-disabled=true])') !== null",
            "the video's controls",
        )
        .await
        .unwrap();
        let state: String = page
            .evaluate_bounded(
                "(() => { const v = document.querySelector('#video video'); \
                 return [getComputedStyle(v).display !== 'none', v.getAttribute('poster') !== null, \
                 document.querySelectorAll('#video [data-slot=message]').length].join(); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            state, "true,true,0",
            "shown, poster attribute, fallback boxes"
        );
        fixture.console.assert_clean(PAGE).unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_playing_audio_draws_no_fallback() {
    block_on(async {
        let fixture = Fixture::open(PAGE, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelector('#audio [data-slot=seek] [role=slider]:not([aria-disabled=true])') !== null",
            "the audio's controls",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "document.querySelectorAll('#audio [data-slot=message]').length === 0",
            "no fallback box",
        )
        .await
        .unwrap();
        fixture.console.assert_clean(PAGE).unwrap();
        fixture.close().await.unwrap();
    });
}
