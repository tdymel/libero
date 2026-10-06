//! `QrCode`: one image named by `aria_label`, drawn dark on light.

use e2e::browser::{block_on, force_colours};
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
        assert_eq!(
            colours(&fixture).await,
            ["rgb(255, 255, 255)", "rgb(0, 0, 0)"]
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 2525: forced colours would invert the code, which many scanners reject.
#[test]
fn the_code_stays_dark_on_light_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/qr-code", Viewport::Desktop).await.unwrap();
        wait::for_visible(&fixture.page, "#qr svg").await.unwrap();
        force_colours(&fixture.page).await.unwrap();
        // Chromium's emulated palette may be light already: the opt-out is the claim.
        let adjust: String = e2e::js(
            &fixture.page,
            "getComputedStyle(document.querySelector('#qr')).forcedColorAdjust",
        )
        .await;
        assert_eq!(adjust, "none");
        assert_eq!(
            colours(&fixture).await,
            ["rgb(255, 255, 255)", "rgb(0, 0, 0)"]
        );
        fixture.close().await.unwrap();
    });
}

/// The quiet zone's background and the modules' fill.
async fn colours(fixture: &Fixture) -> Vec<String> {
    fixture
        .page
        .evaluate(
            "[getComputedStyle(document.querySelector('#qr')).backgroundColor, \
             getComputedStyle(document.querySelector('#qr svg path')).fill]",
        )
        .await
        .unwrap()
        .into_value()
        .unwrap()
}
