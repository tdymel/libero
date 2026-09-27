//! `BottomNavigation` (todo 1209): a `nav` of items, each its own Tab stop, the
//! selected one `aria-current`; 48px targets and no overflow at 320px; RTL;
//! a fixed bar at the viewport's bottom that publishes its height.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, wait};

async fn js(page: &chromiumoxide::Page, expression: &str) -> serde_json::Value {
    page.evaluate(expression)
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("bottom_navigation", "/bottom-navigation")
        .focusable("#home")
        .focusable("#search")
        .focusable("#link")
        .targets("#home")
        .targets("#search")
        .targets("#inbox")
        .targets("#long")
        .targets("#profile")
        .targets("#link")
        .tab_budget(20)
        .run();
}

/// Every item is a Tab stop in order, and Enter on one selects it.
#[test]
fn each_item_is_a_tab_stop_and_enter_selects_it() {
    block_on(async {
        let fixture = Fixture::open("/bottom-navigation", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#home").await.unwrap();

        keyboard::tab_to(page, "#home", 5).await.unwrap();
        for next in ["search", "inbox", "long", "profile"] {
            keyboard::press(page, keyboard::TAB).await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.activeElement.id === '{next}'"),
                "Tab to the next item",
            )
            .await
            .unwrap();
        }

        keyboard::tab_to(page, "#search", 10).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#search').getAttribute('aria-current') === 'page' \
             && !document.querySelector('#home').hasAttribute('aria-current') \
             && document.querySelectorAll('#bar [aria-current]').length === 1",
            "Enter to select the item",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("selecting an item").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1.4.10 and 2.5.8: five items fit a 320px column, each at least 48px square,
/// and a long label stops at two lines.
#[test]
fn five_items_fit_320px_with_48px_targets() {
    block_on(async {
        let fixture = Fixture::open("/bottom-navigation", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#home").await.unwrap();

        let report = js(
            page,
            "(() => { \
                const bar = document.querySelector('#bar'); \
                const items = [...bar.querySelectorAll('[data-slot=item]')]; \
                const small = items.map(i => i.getBoundingClientRect()) \
                    .filter(r => r.width < 48 || r.height < 48).length; \
                const label = document.querySelector('#long [data-slot=label]'); \
                const lines = Math.round(label.getBoundingClientRect().height \
                    / parseFloat(getComputedStyle(label).lineHeight)); \
                return { items: items.length, small, lines, \
                    overflow: bar.scrollWidth > bar.clientWidth, \
                    width: bar.getBoundingClientRect().width }; \
            })()",
        )
        .await;
        assert_eq!(report["items"], 5, "{report}");
        assert_eq!(report["small"], 0, "{report}");
        assert_eq!(report["overflow"], false, "{report}");
        assert_eq!(report["width"], 320.0, "{report}");
        assert!(report["lines"].as_f64().unwrap() <= 2.0, "{report}");

        fixture.close().await.unwrap();
    });
}

/// The selected pill is a fill, which forced colours would drop: it takes Highlight.
#[test]
fn the_selected_pill_shows_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/bottom-navigation", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#home").await.unwrap();
        crate::calendar::force_colours(page).await;

        let differs = js(
            page,
            "(() => { \
                const fill = id => getComputedStyle(document.querySelector(id + ' > [data-slot=icon]')).backgroundColor; \
                return [fill('#home'), fill('#search')]; \
            })()",
        )
        .await;
        assert!(
            differs[0] != differs[1] && differs[1].as_str().unwrap().ends_with(", 0)"),
            "{differs}"
        );

        fixture.close().await.unwrap();
    });
}

#[test]
fn rtl_puts_the_first_item_at_the_right() {
    block_on(async {
        let fixture = Fixture::open("/bottom-navigation", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#rtl-home").await.unwrap();

        let ordered = js(
            page,
            "(() => { \
                const x = id => document.querySelector(id).getBoundingClientRect().left; \
                return x('#rtl-home') > x('#rtl-search') && x('#rtl-search') > x('#rtl-inbox'); \
            })()",
        )
        .await;
        assert_eq!(ordered, true);

        fixture.close().await.unwrap();
    });
}

/// The name comes from the label, the badge's count from the caller's
/// `aria-label`; the icon stays out of the tree.
#[test]
fn items_are_named_by_their_labels() {
    block_on(async {
        let fixture = Fixture::open("/bottom-navigation", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#home").await.unwrap();

        let tree = e2e::ax::snapshot(page, "#bar").await.unwrap();
        for line in [
            "navigation \"Main\"",
            "button \"Home\"",
            "button \"Inbox, 3 unread\"",
            "button \"Notifications and messages\"",
        ] {
            assert!(tree.contains(line), "no {line:?} in\n{tree}");
        }
        let links = e2e::ax::snapshot(page, "#link").await.unwrap();
        assert!(links.contains("link \"Here\""), "{links}");

        fixture.close().await.unwrap();
    });
}

/// A fixed bar sits on the viewport's bottom edge, publishes its height, and its
/// hidden labels still name the items.
#[test]
fn a_fixed_bar_docks_and_publishes_its_height() {
    block_on(async {
        let fixture = Fixture::open("/bottom-navigation/fixed", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#fixed-home").await.unwrap();

        let report = js(
            page,
            "(() => { \
                const bar = document.querySelector('#fixed').getBoundingClientRect(); \
                const root = getComputedStyle(document.documentElement); \
                const label = document.querySelector('#fixed-search [data-slot=label]').getBoundingClientRect(); \
                return { bottom: Math.round(bar.bottom), height: innerHeight, width: bar.width, \
                    full: innerWidth, published: root.getPropertyValue('--lsx-bottom-navigation-height').trim(), \
                    padding: root.scrollPaddingBottom, hidden: label.width <= 1 }; \
            })()",
        )
        .await;
        assert_eq!(report["bottom"], report["height"], "{report}");
        assert_eq!(report["width"], report["full"], "{report}");
        // The env() is resolved: 0px here, no notch.
        assert_eq!(report["published"], "calc(56px + 0px)", "{report}");
        assert_ne!(report["padding"], "auto", "{report}");
        assert_eq!(report["hidden"], true, "{report}");

        let tree = e2e::ax::snapshot(page, "#fixed").await.unwrap();
        assert!(tree.contains("button \"Search\""), "{tree}");

        fixture.close().await.unwrap();
    });
}
