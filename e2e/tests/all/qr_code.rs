//! `QrCode`: one image named by `aria_label`, drawn dark on light.

use e2e::browser::block_on;
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("qr_code", "/qr-code")
        .no_contrast_coverage("an SVG named by aria-label, no text")
        .run();
}

/// The modules take the theme's foreground through `currentColor`, the quiet
/// zone the root's background, and a scanner needs the contrast between them.
#[test]
fn the_modules_are_painted_dark_on_light() {
    block_on(async {
        let fixture = Fixture::open("/qr-code", Viewport::Desktop).await.unwrap();
        wait::for_visible(&fixture.page, "#qr svg").await.unwrap();
        let colours: Vec<String> = fixture
            .page
            .evaluate(
                "[getComputedStyle(document.querySelector('#qr')).backgroundColor, \
                 getComputedStyle(document.querySelector('#qr svg path')).fill]",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(colours, ["rgb(255, 255, 255)", "rgb(0, 0, 0)"]);
        fixture.close().await.unwrap();
    });
}
