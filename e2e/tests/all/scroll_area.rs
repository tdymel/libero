//! `ScrollArea` re-measures when its pane resizes, not only at mount and on
//! scroll (todo 425). The pane is resized by script, as a `Splitter` or a
//! `FloatingWindow` would, with the window left alone.

use e2e::browser::block_on;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

/// The highest row index a `Virtualize` has in the document.
const LAST_ROW: &str = "Math.max(...[...document.querySelectorAll('#list-pane [data-row]')].map(r => Number(r.dataset.row)))";
/// How often the caller's own `onresize` on the area has run.
const RESIZES: &str = "Number(document.querySelector('#list-resizes').textContent)";

/// 20px rows: a 120px pane renders a dozen, a 600px one needs rows to 29. The
/// area now listens for `onresize` itself, and a caller's listener on it, as
/// `Scroller` has, must still run.
#[test]
fn a_taller_pane_renders_rows_to_its_new_bottom() {
    block_on(async {
        let fixture = Fixture::open("/scroll-area", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // The measured window, not the first render's 1080px guess.
        wait::for_js_true(
            page,
            &format!("{LAST_ROW} < 29 && {RESIZES} > 0"),
            "the window to fit the short pane",
        )
        .await
        .unwrap();
        let before: f64 = page.evaluate(RESIZES).await.unwrap().into_value().unwrap();
        page.evaluate("document.querySelector('#list-pane').style.height = '600px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{LAST_ROW} >= 29"),
            "rows down to the taller pane's bottom",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{RESIZES} > {before}"),
            "the caller's own onresize to run",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a list pane resize").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Contrast of the `#region` area's scrollbar thumb against the page, under
/// the scheme the document root is pinned to.
const THUMB_CONTRAST: &str = r#"(() => {
    const lum = c => {
        const [r, g, b] = c.match(/[\d.]+/g).slice(0, 3).map(v => {
            v = v / 255;
            return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
        });
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
    };
    const thumb = getComputedStyle(document.querySelector('#region')).scrollbarColor.match(/rgba?\([^)]*\)/)[0];
    const [a, b] = [lum(thumb), lum(getComputedStyle(document.body).backgroundColor)].sort((x, y) => y - x);
    return (a + 0.05) / (b + 0.05);
})()"#;

/// WCAG 1.4.11: the thumb is the part that shows where the reader is and what
/// they drag, and its colour is ours rather than the browser's.
#[test]
fn the_scrollbar_thumb_has_3_to_1_against_the_page() {
    block_on(async {
        let fixture = Fixture::open("/scroll-area/keyboard", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        for scheme in ["light", "dark"] {
            page.evaluate(format!(
                "document.documentElement.setAttribute('data-lsx-theme', '{scheme}')"
            ))
            .await
            .unwrap();
            let ratio: f64 = page
                .evaluate(THUMB_CONTRAST)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(ratio >= 3.0, "the {scheme} thumb has {ratio:.2}:1");
        }

        fixture.console.assert_clean("reading the thumb").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A `focusable` area of plain text is a tab stop with a visible ring, and the
/// browser scrolls it with the arrow keys once it holds focus.
#[test]
fn a_focusable_area_takes_tab_and_scrolls_with_the_arrows() {
    const REGION: &str = "#region";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/keyboard", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let ring = focus::assert_focus_ring(page, REGION, 5).await.unwrap();
        focus::assert_ring_contrast(&ring).unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#region').scrollTop > 0",
            "ArrowDown to scroll the focused area",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("scrolling by keyboard")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The area is its own tab stop only while it overflows with nothing focusable
/// inside, re-checked when its content grows or its pane resizes (585).
#[test]
fn an_overflowing_area_of_plain_content_is_a_tab_stop() {
    const STOP: &str = "(id => { const a = document.getElementById(id); \
        return a.getAttribute('tabindex') === '0' && a.getAttribute('role') === 'region'; })";
    let stop = |id: &str| format!("{STOP}('{id}')");
    block_on(async {
        let fixture = Fixture::open("/scroll-area/keyboard", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &stop("plain"),
            "the overflowing plain area to take Tab",
        )
        .await
        .unwrap();
        for (id, why) in [
            ("short", "content that fits"),
            ("links", "links inside"),
            ("grow", "two lines"),
        ] {
            let tab_stop: bool = page.evaluate(stop(id)).await.unwrap().into_value().unwrap();
            assert!(!tab_stop, "#{id} is a tab stop with {why}");
        }

        page.evaluate("document.querySelector('#grow-more').click()")
            .await
            .unwrap();
        wait::for_js_true(page, &stop("grow"), "the grown area to take Tab")
            .await
            .unwrap();

        page.evaluate("document.querySelector('#short-pane').style.height = '8px'")
            .await
            .unwrap();
        wait::for_js_true(page, &stop("short"), "the squeezed area to take Tab")
            .await
            .unwrap();

        let ring = focus::assert_focus_ring(page, "#plain", 3).await.unwrap();
        focus::assert_ring_contrast(&ring).unwrap();

        fixture
            .console
            .assert_clean("the automatic tab stop")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A caller's name reaches the area only while it has a role: ARIA prohibits
/// naming a generic (693). It follows the stop both ways on a resize.
#[test]
fn the_name_waits_for_a_role() {
    const NAME: &str = "(id => document.getElementById(id).getAttribute('aria-label') ?? '')";
    let name = |id: &str| format!("{NAME}('{id}')");
    block_on(async {
        let fixture = Fixture::open("/scroll-area/keyboard", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &format!("{} === 'Changelog'", name("plain")),
            "the region's name",
        )
        .await
        .unwrap();
        for (id, expected) in [
            ("short", ""),
            ("listed", "Items"),
            ("region", "Release notes"),
        ] {
            let seen: String = page.evaluate(name(id)).await.unwrap().into_value().unwrap();
            assert_eq!(seen, expected, "#{id}'s name");
        }

        page.evaluate("document.querySelector('#short-pane').style.height = '8px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{} === 'Note'", name("short")),
            "the squeezed area's name",
        )
        .await
        .unwrap();
        page.evaluate("document.querySelector('#short-pane').style.height = '120px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{} === ''", name("short")),
            "the name to go with the stop",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("the name waits for a role")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Content changing inside a child component re-renders nothing in the area:
/// the stop is re-checked all the same, both ways (681).
#[test]
fn a_change_inside_a_child_component_re_checks_the_tab_stop() {
    const STOP: &str = "document.getElementById('nested').getAttribute('tabindex') === '0'";
    const STEP: &str = "document.querySelector('#nested-step').click()";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/keyboard", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // The mount-time check has run once the plain area took its stop.
        wait::for_js_true(page, &STOP.replace("nested", "plain"), "the mount check")
            .await
            .unwrap();
        let tab_stop: bool = page.evaluate(STOP).await.unwrap().into_value().unwrap();
        assert!(!tab_stop, "two lines made a tab stop");

        page.evaluate(STEP).await.unwrap();
        wait::for_js_true(page, STOP, "forty lines to make a tab stop")
            .await
            .unwrap();

        page.evaluate(STEP).await.unwrap();
        wait::for_js_true(page, &format!("!({STOP})"), "a link to take the stop away")
            .await
            .unwrap();

        fixture
            .console
            .assert_clean("a change inside a child")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Under RTL the area starts at its right edge and `scrollLeft` runs negative:
/// the percent counts from the start, 100% lands on the left edge, and left
/// and right stay the physical edges.
#[test]
fn under_rtl_the_percent_counts_from_the_right_edge() {
    const AREA: &str = "document.querySelector('#wide')";
    let text = |id: &str| format!("document.querySelector('#{id}').textContent");
    block_on(async {
        let fixture = Fixture::open("/scroll-area/rtl", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, "#to-end").await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{AREA}.scrollLeft <= -({AREA}.scrollWidth - {AREA}.clientWidth) + 1 \
                 && {} === '100' && {} === '1' && {} === '0'",
                text("x"),
                text("left-reached"),
                text("right-reached")
            ),
            "100% to reach the left edge, and onleftreached alone",
        )
        .await
        .unwrap();

        page.evaluate(format!("{AREA}.scrollLeft = 0"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{} === '0' && {} === '1'", text("x"), text("right-reached")),
            "onrightreached back at the start",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an RTL area").unwrap();
        fixture.close().await.unwrap();
    });
}

/// An area with only `on*reached` handlers still listens for scrolls (todo 29
/// attaches `onscroll` only when something reads it).
#[test]
fn the_reached_callbacks_fire_at_both_ends() {
    const AREA: &str = "document.querySelector('#edges')";
    const COUNT: &str = "Number(document.querySelector('#{}-reached').textContent)";
    let count = |edge: &str| COUNT.replace("{}", edge);

    block_on(async {
        let fixture = Fixture::open("/scroll-area/edges", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate(format!("{AREA}.scrollTop = {AREA}.scrollHeight"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{} === 1", count("bottom")),
            "onbottomreached at the bottom",
        )
        .await
        .unwrap();

        page.evaluate(format!("{AREA}.scrollTop = 0"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{} === 1", count("top")),
            "ontopreached back at the top",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("scrolling to both ends")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
