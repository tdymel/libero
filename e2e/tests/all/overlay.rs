//! `Overlay` and `LoadingOverlay` (todo 3985): a layer over one box, and that box's loading
//! state, busy and inert while it lasts.

use e2e::browser::block_on;
use e2e::passes::{focus::assert_focused, keyboard, pointer};
use e2e::{Fixture, Viewport, js, wait};

/// A `contained` overlay covers its positioned box, not the viewport.
#[test]
fn a_contained_overlay_covers_only_its_box() {
    block_on(async {
        let fixture = Fixture::open("/overlay", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#contained").await.unwrap();
        let same: bool = js(
            page,
            "(() => { const a = document.querySelector('#box').getBoundingClientRect(); \
             const b = document.querySelector('#contained').getBoundingClientRect(); \
             return a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height; })()",
        )
        .await;
        assert!(same, "the contained overlay matches its box");
        fixture.console.assert_clean("a contained overlay").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Loading: the box is `aria-busy`, Tab skips its content, a status region outside it
/// says so. Done: the layer goes, Tab reaches the content, the status empties.
#[test]
fn a_loading_box_is_busy_inert_and_announced() {
    /// The busy box's next sibling, so outside it.
    const STATUS: &str = "#loading + [role=status]";

    block_on(async {
        let fixture = Fixture::open("/overlay", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#loading [data-slot=overlay]")
            .await
            .unwrap();
        let busy: Option<String> = js(
            page,
            "document.querySelector('#loading').getAttribute('aria-busy')",
        )
        .await;
        assert_eq!(busy.as_deref(), Some("true"), "the box is busy");
        let status: String = js(
            page,
            format!("document.querySelector({STATUS:?}).textContent"),
        )
        .await;
        assert_eq!(status, "Loading", "the status says it");

        pointer::click(page, "#before").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        assert_focused(page, "#after", "Tab past the loading content")
            .await
            .unwrap();

        pointer::click(page, "#toggle").await.unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#loading [data-slot=overlay]') \
             && !document.querySelector('#loading').hasAttribute('aria-busy')",
            "the loading layer to go",
        )
        .await
        .unwrap();
        let status: String = js(
            page,
            format!("document.querySelector({STATUS:?}).textContent"),
        )
        .await;
        assert_eq!(status, "", "the status empties");
        pointer::click(page, "#before").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        assert_focused(page, "#inside", "Tab into the loaded content")
            .await
            .unwrap();

        fixture.console.assert_clean("a loading box").unwrap();
        fixture.close().await.unwrap();
    });
}
