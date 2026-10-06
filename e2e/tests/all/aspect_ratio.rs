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

/// Todo 2532: a `video` covers inset shadows and takes no `::after`, so its stripe
/// is a painted outline inside the clip.
#[test]
fn a_focused_video_shows_a_painted_ring() {
    block_on(async {
        let fixture = Fixture::open("/aspect-ratio-children", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#video", 10).await.unwrap();
        let outline: Vec<String> = js(
            page,
            "(() => { const s = getComputedStyle(document.activeElement); \
               return [s.outlineStyle, s.outlineColor, s.outlineOffset]; })()",
        )
        .await;
        assert_eq!(outline[0], "solid", "{outline:?}");
        assert_ne!(outline[1], "rgba(0, 0, 0, 0)", "{outline:?}");
        assert!(outline[2].starts_with('-'), "{outline:?}");
        let clipped: f64 = js(page, crate::image_list::ring_clipped("#video-ratio")).await;
        assert!(clipped <= 0.0, "the ring runs {clipped}px past the clip");
        fixture.console.assert_clean("a focused video").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2535: the overlay anchors on the root, so focus does not move a positioned child.
#[test]
fn focus_leaves_a_positioned_child_in_place() {
    block_on(async {
        let fixture = Fixture::open("/aspect-ratio-children", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let place = "(() => { const el = document.querySelector('#pinned'); \
            const r = el.getBoundingClientRect(); \
            return [getComputedStyle(el).position, `${r.left},${r.top},${r.width},${r.height}`]; })()";
        let before: Vec<String> = js(page, place).await;
        keyboard::tab_to(page, "#pinned", 10).await.unwrap();
        let after: Vec<String> = js(page, place).await;
        assert_eq!(before, after);
        assert_eq!(after[0], "absolute");
        let overlay: String = js(
            page,
            "getComputedStyle(document.activeElement, '::after').boxShadow",
        )
        .await;
        assert!(overlay.contains("inset"), "no overlay ring: {overlay}");
        fixture.console.assert_clean("a positioned child").unwrap();
        fixture.close().await.unwrap();
    });
}
