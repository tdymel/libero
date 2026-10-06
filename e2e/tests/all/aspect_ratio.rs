//! `AspectRatio`: a focused child fills the clipped box, so its ring is inset.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, js};

/// Todo 2480: the picture in a focused link covers the link's inset shadows, so the
/// stripe rides an overlay above it, inside the clip.
#[test]
fn a_focused_link_paints_its_ring_over_its_picture() {
    block_on(async {
        let fixture = Fixture::open("/aspect-ratio-link", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#link", 10).await.unwrap();
        let overlay: String = js(
            page,
            "getComputedStyle(document.activeElement, '::after').boxShadow",
        )
        .await;
        assert!(
            overlay.contains("inset"),
            "no ring over the picture: {overlay}"
        );
        let clipped: f64 = js(page, crate::image_list::ring_clipped("#ratio")).await;
        assert!(clipped <= 0.0, "the ring runs {clipped}px past the clip");
        fixture.console.assert_clean("a linked picture").unwrap();
        fixture.close().await.unwrap();
    });
}
