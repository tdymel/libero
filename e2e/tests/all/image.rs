//! `Image`: the text alternative of a described, a decorative and a zoomable
//! picture, the fallback source, and the zoom button.

use e2e::browser::block_on;
use e2e::passes::keyboard::{self, ESCAPE};
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
