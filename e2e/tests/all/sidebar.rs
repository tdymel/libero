//! `Sidebar`: the overflow tab stop and its forwarded name, `top` / `bottom`, the
//! default size beside content at 320px, and content wider than the panel scrolling.

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

/// A top or bottom panel takes half its column at most (todo 2645): two default
/// `md` panel in a 256px column leaves the content half of it, not 16px.
#[test]
fn top_and_bottom_sidebars_leave_content_room_at_256px() {
    block_on(async {
        let fixture = Fixture::open("/sidebar/short", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#short-rest").await.unwrap();

        let heights: Vec<f64> = page
            .evaluate(
                "['#short-a', '#short-top', '#short-rest', '#short-bottom'].map(id => \
                   document.querySelector(id).getBoundingClientRect().height)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(heights[..2], [256.0, 128.0]);
        // The content gets the other half less the column's gap, not 16px.
        assert!(heights[2] >= 100.0, "{heights:?}");
        assert_eq!(heights[3], 128.0, "{heights:?}");
        fixture.close().await.unwrap();
    });
}

/// Content wider than the panel scrolls sideways inside it (todo 2476): the panel
/// and the 320px column keep their size, and the far end of the line is reachable.
#[test]
fn wide_content_scrolls_inside_the_sidebar() {
    block_on(async {
        let fixture = Fixture::open("/sidebar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#wide-content").await.unwrap();

        let fit: String = page
            .evaluate(
                "(() => { const column = document.querySelector('#column'); \
                   const wide = document.querySelector('#wide'); \
                   const scroll = wide.querySelector('[data-slot=scroll]'); \
                   scroll.scrollLeft = scroll.scrollWidth; \
                   const content = document.querySelector('#wide-content').getBoundingClientRect(); \
                   const area = scroll.getBoundingClientRect(); \
                   return `${wide.getBoundingClientRect().width} ${column.scrollWidth <= column.clientWidth} \
                     ${getComputedStyle(scroll).overflowX} ${content.right <= area.right}`; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            fit, "160 true auto true",
            "panel width, the column fits, the area scrolls, the line's end is reachable"
        );
        fixture.close().await.unwrap();
    });
}

/// A start or end panel takes half its row at most (todo 2477), so the default `md`
/// panel leaves its sibling room beside content at 320px.
#[test]
fn the_default_sidebar_leaves_content_room_at_320px() {
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
        // Half the row for the panel; the sibling gets the rest less the row's gap.
        assert_eq!(widths[..2], [320.0, 160.0]);
        assert!(widths[2] >= 140.0, "{widths:?}");
        fixture.close().await.unwrap();
    });
}
