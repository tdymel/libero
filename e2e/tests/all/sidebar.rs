//! `Sidebar`: the overflow tab stop and its forwarded name, `top` / `bottom`, the
//! default size beside content at 320px, and content wider than the panel.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, ax, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("sidebar", "/sidebar").run();
}

/// Overflowing content with nothing focusable makes the scroll area a tab stop,
/// named by the panel's own `aria-label` or `aria-labelledby`.
#[test]
fn an_overflowing_sidebar_is_a_named_tab_stop() {
    block_on(async {
        let fixture = Fixture::open("/sidebar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#start > [data-slot='scroll']", 2)
            .await
            .unwrap();
        keyboard::tab_to(page, "#end > [data-slot='scroll']", 2)
            .await
            .unwrap();

        let start = ax::snapshot(page, "#start").await.unwrap();
        assert!(
            start.starts_with("complementary \"Filters\"\n  region \"Filters\""),
            "{start}"
        );
        let end = ax::snapshot(page, "#end").await.unwrap();
        assert!(
            end.starts_with("complementary \"Inspector\"\n  region \"Inspector\""),
            "{end}"
        );

        fixture.console.assert_clean("the sidebar fixture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `top` / `bottom` size the height and border the edge facing the content.
#[test]
fn a_top_or_bottom_sidebar_sizes_its_height() {
    block_on(async {
        let fixture = Fixture::open("/sidebar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#bottom").await.unwrap();

        let sides: Vec<String> = page
            .evaluate(
                "[['#top', 'Bottom'], ['#bottom', 'Top']].map(([id, edge]) => { \
                   const panel = document.querySelector(id), s = getComputedStyle(panel); \
                   const width = panel.parentElement.clientWidth; \
                   return `${id} ${s.height} ${s['border' + edge + 'Style']} ${panel.offsetWidth === width}`; })",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(sides, ["#top 200px solid true", "#bottom 200px solid true"]);
        fixture.close().await.unwrap();
    });
}

/// Content wider than the panel stays inside it: the panel keeps its width and
/// the 320px column does not scroll sideways (todo 2476 decides on scrolling it).
#[test]
fn wide_content_keeps_the_sidebar_and_its_column_in_size() {
    block_on(async {
        let fixture = Fixture::open("/sidebar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#wide-content").await.unwrap();

        let fit: String = page
            .evaluate(
                "(() => { const column = document.querySelector('#column'); \
                   const wide = document.querySelector('#wide').getBoundingClientRect().width; \
                   return `${wide} ${column.scrollWidth <= column.clientWidth}`; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(fit, "200 true", "panel width, and the 320px column fits");
        fixture.close().await.unwrap();
    });
}

/// The default `md` panel does not shrink in a 320px row: its sibling gets
/// what is left (todo 2477 decides whether it should).
#[test]
fn the_default_sidebar_keeps_its_width_beside_content_at_320px() {
    block_on(async {
        let fixture = Fixture::open("/sidebar/narrow", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#default-rest").await.unwrap();

        let widths: Vec<f64> = page
            .evaluate(
                "['#column', '#default', '#default-rest'].map(id => \
                   document.querySelector(id).getBoundingClientRect().width)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        // 320 less the panel and the row's gap.
        assert_eq!(widths, [320.0, 280.0, 28.0]);
        fixture.close().await.unwrap();
    });
}
