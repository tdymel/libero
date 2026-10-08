//! `Image`: the text alternative of a described, a decorative and a zoomable
//! picture, the fallback source, and the zoom button.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, ESCAPE};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("image", "/image")
        .no_contrast_coverage("pictures and a zoom button, no text")
        .focusable("#zoom")
        .targets("#zoom")
        .run();
}

/// The attribute's value, `-` when it is absent.
async fn attr(fixture: &Fixture, selector: &str, name: &str) -> String {
    fixture
        .page
        .evaluate(format!(
            "document.querySelector({selector:?}).getAttribute({name:?}) ?? '-'"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn a_broken_source_falls_back_and_keeps_its_alt() {
    block_on(async {
        let fixture = Fixture::open("/image", Viewport::Desktop).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            "(() => { const i = document.querySelector('#fallback'); \
             return i.complete && i.naturalWidth > 0; })()",
            "the fallback source to load",
        )
        .await
        .unwrap();
        assert_eq!(attr(&fixture, "#fallback", "alt").await, "A brown square");
        assert_eq!(attr(&fixture, "#decorative", "alt").await, "");
        fixture.close().await.unwrap();
    });
}

/// Todo 2528: `fallback_src: ""` is no fallback, so the broken `src` stays, as natively.
#[test]
fn an_empty_fallback_keeps_the_broken_source() {
    block_on(async {
        let fixture = Fixture::open("/image", Viewport::Desktop).await.unwrap();
        // Both share the missing source: once `#fallback` swapped, both errors were handled.
        wait::for_js_true(
            &fixture.page,
            "(() => { const i = document.querySelector('#fallback'); \
             return i.complete && i.naturalWidth > 0 \
             && document.querySelector('#empty-fallback').complete; })()",
            "both missing sources to fail",
        )
        .await
        .unwrap();
        assert_eq!(
            attr(&fixture, "#empty-fallback", "src").await,
            "/does-not-exist.png"
        );
        fixture.close().await.unwrap();
    });
}

/// Swallows every image `error`, as a server-rendered page misses it before hydration, lets the
/// pictures fail once, then mounts them again: the browser has the failure by then.
async fn open_missed_error(fixture: &Fixture) {
    let page = &fixture.page;
    page.evaluate(
        "window.addEventListener('error', e => { \
         if (e.target instanceof HTMLImageElement) e.stopImmediatePropagation(); }, true)",
    )
    .await
    .unwrap();
    pointer::click(page, "#mount").await.unwrap();
    wait::for_js_true(
        page,
        "[...document.querySelectorAll('img')].every(i => i.complete)",
        "the pictures to fail",
    )
    .await
    .unwrap();
    pointer::click(page, "#mount").await.unwrap();
    wait::for_js_true(
        page,
        "!document.querySelector('img')",
        "the pictures to unmount",
    )
    .await
    .unwrap();
    pointer::click(page, "#mount").await.unwrap();
}

/// A raster and an SVG picture that fail to decode (todo 2671: an SVG reads empty, not failed).
const MISSED_ERROR_ROUTES: [&str; 2] = ["/image/missed-error", "/image/missed-error-svg"];

/// Todos 2529, 2671: an `Image` whose source failed before its `onerror` existed shows the fallback.
#[test]
fn an_image_that_failed_before_its_listener_shows_the_fallback() {
    block_on(async {
        for route in MISSED_ERROR_ROUTES {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            open_missed_error(&fixture).await;
            wait::for_js_true(
                &fixture.page,
                "document.querySelector('#missed-image')?.src.includes('963')",
                "the fallback source",
            )
            .await
            .unwrap();
            // The cropper on the page warns about its source.
            fixture.console.drain();
            fixture.close().await.unwrap();
        }
    });
}

/// Todos 2529, 2671: an `Avatar` whose source failed before its `onerror` existed shows its
/// initials.
#[test]
fn an_avatar_that_failed_before_its_listener_shows_its_initials() {
    block_on(async {
        for route in MISSED_ERROR_ROUTES {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            open_missed_error(&fixture).await;
            wait::for_js_true(
                &fixture.page,
                "(() => { const a = document.querySelector('#missed-avatar'); \
                 return !!a && !a.querySelector('img') && a.textContent === 'AL'; })()",
                "the initials instead of the picture",
            )
            .await
            .unwrap();
            // The cropper on the page warns about its source.
            fixture.console.drain();
            fixture.close().await.unwrap();
        }
    });
}

/// Todos 2529, 2671: an `ImageCropper` whose source failed before its `onerror` existed reports
/// it once and draws no box.
#[test]
fn a_cropper_that_failed_before_its_listener_reports_and_draws_no_box() {
    block_on(async {
        for route in MISSED_ERROR_ROUTES {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            open_missed_error(&fixture).await;
            wait::for_js_true(
                &fixture.page,
                "document.getElementById('error')?.textContent === '1' \
                 && !document.querySelector('[data-slot=box], [data-slot=frame]')",
                &format!("onerror, and no box on {route}"),
            )
            .await
            .unwrap();
            // The cropper on the page warns about its source.
            fixture.console.drain();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 2527: the zoom button takes the picture's radius, so its focus ring is rounded too.
#[test]
fn the_zoom_button_has_the_picture_radius() {
    block_on(async {
        let fixture = Fixture::open("/image", Viewport::Desktop).await.unwrap();
        wait::for_visible(&fixture.page, "#zoom").await.unwrap();
        let [button, img]: [String; 2] = fixture
            .page
            .evaluate(
                "['#zoom', '#zoom > img'].map(q => \
                 getComputedStyle(document.querySelector(q)).borderRadius)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(img, "0px");
        assert_eq!(button, img);
        fixture.close().await.unwrap();
    });
}

/// `loading`, `srcset` and the rest describe the picture: on a zoomable
/// image they belong on the `<img>`, not on the `<button>` wrapping it.
#[test]
fn a_zoomable_image_keeps_its_img_attributes_on_the_img() {
    block_on(async {
        let fixture = Fixture::open("/image", Viewport::Desktop).await.unwrap();
        wait::for_visible(&fixture.page, "#zoom").await.unwrap();
        assert_eq!(attr(&fixture, "#zoom", "loading").await, "-");
        assert_eq!(attr(&fixture, "#zoom > img", "loading").await, "lazy");
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_zoom_button_opens_a_named_dialog_and_escape_returns_focus() {
    block_on(async {
        let fixture = Fixture::open("/image", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#zoom").await.unwrap();
        page.evaluate("document.querySelector('#zoom').focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const d = document.querySelector('[role=dialog]'); \
             return !!d && d.getAttribute('aria-label') === 'A blue square' \
             && d.contains(document.activeElement); })()",
            "a dialog named by the alt, holding focus",
        )
        .await
        .unwrap();
        keyboard::press(page, ESCAPE).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement?.id === 'zoom'",
            "focus back on the zoom button",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the image fixture").unwrap();
        fixture.close().await.unwrap();
    });
}
