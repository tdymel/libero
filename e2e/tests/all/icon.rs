//! `Icon`: its colour variables are cached, so a new `color` still has to
//! repaint the badge.

use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::{Fixture, Viewport, ax, wait};

const FILL: &str = "getComputedStyle(document.querySelector('#icon')).backgroundColor";

#[test]
fn a_filled_icon_repaints_when_its_colour_changes() {
    block_on(async {
        let fixture = Fixture::open("/icon", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        let before: String = page.evaluate(FILL).await.unwrap().into_value().unwrap();
        assert_ne!(before, "rgba(0, 0, 0, 0)", "a filled icon draws a fill");

        pointer::click(page, "#swap").await.unwrap();
        wait::for_js_true(
            page,
            &format!("{FILL} !== {before:?}"),
            "the fill to follow the new colour",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("the icon fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 786: a contained glyph keeps clear of its box's edges; a bare one fills it.
#[test]
fn a_contained_glyph_is_inset_and_a_bare_one_fills_the_box() {
    block_on(async {
        let fixture = Fixture::open("/icon", Viewport::Desktop).await.unwrap();
        let share = |id: &str| {
            format!(
                "(() => {{ const b = document.getElementById('{id}'); \
                 return b.querySelector('svg').getBoundingClientRect().width / b.getBoundingClientRect().width; }})()"
            )
        };

        let filled: f64 = fixture
            .page
            .evaluate(share("icon"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!((filled - 0.6).abs() < 0.02, "filled glyph share {filled}");
        let bare: f64 = fixture
            .page
            .evaluate(share("bare"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!((bare - 1.0).abs() < 0.02, "standard glyph share {bare}");

        fixture.close().await.unwrap();
    });
}

/// Todo 612: an unnamed icon is not in the accessibility tree; a named one is
/// an image under its name.
#[test]
fn an_unnamed_icon_is_hidden_and_a_named_one_is_an_image() {
    block_on(async {
        let fixture = Fixture::open("/icon", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        // `aria-hidden` drops the node from the tree, or leaves it ignored.
        let unnamed = ax::snapshot(page, "#icon").await.unwrap_or_default();
        assert_eq!(unnamed.trim(), "", "{unnamed}");
        let named = ax::snapshot(page, "#named").await.unwrap();
        // Chrome still lists the unnamed svg under it; `img` children are presentational.
        assert!(named.starts_with("image \"Verified\"\n"), "{named}");

        fixture.close().await.unwrap();
    });
}
