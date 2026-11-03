//! `Sidebar`: the overflow tab stop and its forwarded name, `top` / `bottom`, the
//! default size beside content at 320px, and content wider than the panel scrolling.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
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
            .evaluate_bounded(
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
            .evaluate_bounded(
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

/// Todo 2893: under a parent with an auto height the half-height cap is `none`, so the
/// default `md` panel keeps its full 280px.
#[test]
fn a_top_sidebar_in_an_auto_height_parent_is_not_capped() {
    block_on(async {
        let fixture = Fixture::open("/sidebar/short", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#auto-top").await.unwrap();

        let height: f64 = page
            .evaluate_bounded("document.querySelector('#auto-top').getBoundingClientRect().height")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(height, 280.0);
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
            .evaluate_bounded(
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
            .evaluate_bounded(
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

/// Todo 5469: a start or end panel pads the physical side it sits on by a landscape notch;
/// top and bottom leave the insets to the Header or NavigationBar.
#[test]
fn a_side_panel_clears_the_notch_on_its_side() {
    use chromiumoxide::cdp::browser_protocol::emulation::{
        SafeAreaInsets, SetSafeAreaInsetsOverrideParams,
    };
    // The extra over the base padding of each scroll area: [left, right].
    const EXTRA: &str = "['#start', '#end', '#top', '#bottom'].map(id => { \
         const s = getComputedStyle(document.querySelector(id + \" > [data-slot='scroll']\")); \
         const [left, right, top] = [s.paddingLeft, s.paddingRight, s.paddingTop].map(parseFloat); \
         return [left - top, right - top]; })";

    block_on(async {
        for (dir, start, end) in [
            ("ltr", [20.0, 0.0], [0.0, 30.0]),
            ("rtl", [0.0, 30.0], [20.0, 0.0]),
        ] {
            let fixture = Fixture::open("/sidebar", Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            page.execute_bounded(SetSafeAreaInsetsOverrideParams::new(SafeAreaInsets {
                top: Some(47),
                bottom: Some(34),
                left: Some(20),
                right: Some(30),
                ..Default::default()
            }))
            .await
            .unwrap();
            page.evaluate_bounded(format!("document.documentElement.dir = '{dir}'"))
                .await
                .unwrap();
            wait::for_visible(page, "#bottom").await.unwrap();
            let extra: Vec<[f64; 2]> = page
                .evaluate_bounded(EXTRA)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(extra, [start, end, [0.0, 0.0], [0.0, 0.0]], "{dir}");
            fixture.close().await.unwrap();
        }
    });
}

/// Todos 5519, 5525: a Container is not edge-bound and pads no inset; a start Sidebar inside a
/// start drawer leaves the notch to the drawer, so neither clears it twice.
#[test]
fn only_the_outermost_edge_panel_clears_the_notch() {
    use chromiumoxide::cdp::browser_protocol::emulation::{
        SafeAreaInsets, SetSafeAreaInsetsOverrideParams,
    };
    // [left, right] padding of one element.
    const PADDING: &str = "(sel => { const s = getComputedStyle(document.querySelector(sel)); \
         return [s.paddingLeft, s.paddingRight].map(parseFloat); })";
    const SCROLL: &str = " > [data-slot='scroll']";

    block_on(async {
        for (dir, beside) in [("ltr", [63.0, 16.0]), ("rtl", [16.0, 63.0])] {
            let fixture = Fixture::open("/sidebar/safe-area", Viewport::Mobile)
                .await
                .unwrap();
            let page = &fixture.page;
            page.execute_bounded(SetSafeAreaInsetsOverrideParams::new(SafeAreaInsets {
                left: Some(47),
                right: Some(47),
                ..Default::default()
            }))
            .await
            .unwrap();
            page.evaluate_bounded(format!("document.documentElement.dir = '{dir}'"))
                .await
                .unwrap();
            wait::for_visible(page, "#content").await.unwrap();
            let padding = async |selector: String| -> [f64; 2] {
                page.evaluate_bounded(format!("{PADDING}({selector:?})"))
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap()
            };

            // The start Sidebar at the edge: lg (16px) plus the 47px inset on its side.
            assert_eq!(padding(format!("#beside{SCROLL}")).await, beside, "{dir}");
            // Plain gutters on both sides, the inset left to the panel beside it.
            let gutters = padding("#content".into()).await;
            assert!(
                gutters[0] == gutters[1] && gutters[0] < 47.0,
                "{dir}: the container {gutters:?}"
            );

            pointer::click(page, "#open").await.unwrap();
            wait::for_visible(page, "#in-drawer").await.unwrap();
            let start = match dir {
                "ltr" => 0,
                _ => 1,
            };
            let dialog = padding("[role=dialog]".into()).await;
            assert_eq!(dialog[start], 63.0, "{dir}: the drawer {dialog:?}");
            let nested = padding(format!("#in-drawer{SCROLL}")).await;
            assert_eq!(nested, [16.0, 16.0], "{dir}: the nested sidebar");

            fixture
                .console
                .assert_clean(&format!("safe-area panels, {dir}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
