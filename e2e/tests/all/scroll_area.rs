//! `ScrollArea` re-measures when its pane resizes (425), resized by script as a `Splitter`
//! would, with the window left alone.

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType,
};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use e2e::browser::block_on;
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::{focus, keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

/// The highest row index a `Virtualize` has in the document.
const LAST_ROW: &str = "Math.max(...[...document.querySelectorAll('#list-pane [data-row]')].map(r => Number(r.dataset.row)))";
/// How often the caller's own `onresize` on the area has run.
const RESIZES: &str = "Number(document.querySelector('#list-resizes').textContent)";

/// 20px rows: a 120px pane shows a dozen, 600px needs 29. The area listens for `onresize`
/// itself; a caller's listener must still run.
/// Todo 1773: none before; axe's `scrollable-region-focusable` among the rest.
#[test]
fn it_meets_the_baseline() {
    for (name, route) in [
        ("scroll_area", "/scroll-area"),
        ("scroll_area_bars", "/scroll-area/bars"),
        ("scroll_area_rtl", "/scroll-area/rtl"),
    ] {
        Suite::new(name, route).run();
    }
}

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

/// The `#region` area's scrollbar thumb colour, as the `thumb` of [`thumb_contrast`].
const REGION_THUMB: &str = "getComputedStyle(document.querySelector('#region')).scrollbarColor";

/// Contrast of the first colour in the JS string `thumb` against the page, under
/// the scheme the document root is pinned to.
fn thumb_contrast(thumb: &str) -> String {
    format!(
        r#"(() => {{ {COLOUR_JS}
    const thumb = {thumb}.match(/rgba?\([^)]*\)/)[0];
    return CONTRAST(RGBA(thumb), RGBA(getComputedStyle(document.body).backgroundColor));
}})()"#
    )
}

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
                .evaluate(thumb_contrast(REGION_THUMB))
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

/// Under RTL `scrollLeft` runs negative from the right edge: the percent counts from the
/// start, 100% is the left edge; left and right stay physical.
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

/// Where `#id`'s drawn part `part` sits, in px from each edge of the area's border box.
const PART: &str = r#"((id, part) => {
    const area = document.getElementById(id);
    const el = area.querySelector(part);
    if (!el) return null;
    const [a, b] = [area.getBoundingClientRect(), el.getBoundingClientRect()];
    return { top: b.top - a.top, left: b.left - a.left, right: a.right - b.right,
        bottom: a.bottom - b.bottom, width: b.width, height: b.height };
})"#;
const V_TRACK: &str = "[data-slot=scrollbar][data-orientation=vertical]";
const V_THUMB: &str = "[data-slot=scrollbar][data-orientation=vertical] > [data-slot=thumb]";
const H_TRACK: &str = "[data-slot=scrollbar][data-orientation=horizontal]";
const H_THUMB: &str = "[data-slot=scrollbar][data-orientation=horizontal] > [data-slot=thumb]";

/// A JS expression: `#id`'s `part` satisfies `test`, a JS expression over `p`.
fn part(id: &str, part: &str, test: &str) -> String {
    format!("(p => !!p && ({test}))({PART}('{id}', '{part}'))")
}

/// The drawn bars are out of the area's flow: a flex column and a two-column grid, both
/// with 10px gaps and 8px padding, lay their items out exactly as with no bar at all.
#[test]
fn flex_and_grid_areas_lay_out_as_without_the_bars() {
    let item = |id: &str, n: u32, test: &str| part(id, &format!("[data-item=\"{n}\"]"), test);
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        for id in ["bars-flex", "bars-grid"] {
            wait::for_js_true(page, &part(id, V_TRACK, "true"), "the drawn track")
                .await
                .unwrap();
        }

        for (id, n, left, top) in [
            ("bars-flex", 0, 8.0, 8.0),
            ("bars-flex", 1, 8.0, 38.0),
            ("bars-grid", 0, 8.0, 8.0),
            // (240 - 16 - 10) / 2 = 107px columns.
            ("bars-grid", 1, 125.0, 8.0),
            ("bars-grid", 2, 8.0, 38.0),
        ] {
            let test = format!("Math.abs(p.left - {left}) < 1 && Math.abs(p.top - {top}) < 1");
            let at: bool = page
                .evaluate(item(id, n, &test))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(at, "#{id} item {n} is not at ({left}, {top})");
        }

        fixture.console.assert_clean("flex and grid areas").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The vertical thumb of `#bars-v` sits where its share of the padding box and the
/// scroll offset put it.
const V_THUMB_PLACED: &str = "(() => { const a = document.querySelector('#bars-v'); \
     const thumb = Math.max(a.clientHeight ** 2 / a.scrollHeight, 20); \
     const at = (a.clientHeight - thumb) * a.scrollTop / (a.scrollHeight - a.clientHeight); \
     return Math.abs(p.top - at) < 1 && Math.abs(p.height - thumb) < 1; })()";

/// Where scroll timelines are missing (Firefox), inline styles keep the track in place
/// and move the thumb: here with `CSS.supports` denying them and their animations off.
#[test]
fn without_scroll_timelines_the_track_still_stays_put() {
    const AREA: &str = "document.querySelector('#bars-v')";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // Before the app reads it: a reload runs the script first.
        page.execute(AddScriptToEvaluateOnNewDocumentParams::new(
            "(() => { const supports = CSS.supports.bind(CSS); \
             CSS.supports = (...a) => !String(a.join(':')).includes('animation-timeline') && supports(...a); \
             addEventListener('DOMContentLoaded', () => { const s = document.createElement('style'); \
             s.textContent = '[data-slot=scrollbars],[data-scrollbars-x],[data-scrollbars-x] [data-slot=thumb]{animation:none!important}'; \
             document.head.append(s); }); })()",
        ))
        .await
        .unwrap();
        page.reload().await.unwrap();
        wait::for_js_true(
            page,
            "!CSS.supports('animation-timeline: scroll()') && !!document.querySelector('#bars-v')",
            "the reloaded app",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &part("bars-v", V_TRACK, "true"), "the drawn track")
            .await
            .unwrap();

        page.evaluate(format!("{AREA}.scrollTop = 300"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &part(
                "bars-v",
                V_TRACK,
                "Math.abs(p.top) < 1 && Math.abs(p.right) < 1",
            ),
            "the track back in the corner after a scroll",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, V_THUMB_PLACED),
            "the thumb placed by its inline style",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("the fallback").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A WebView runs the timeline animations while the app writes inline offsets (todo 2019):
/// the thumb moved twice, hung past the end and grew the scroll range on every read.
#[test]
fn timelines_the_app_misses_never_grow_the_range() {
    const AREA: &str = "document.querySelector('#bars-v')";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(AddScriptToEvaluateOnNewDocumentParams::new(
            "(() => { const supports = CSS.supports.bind(CSS); \
             CSS.supports = (...a) => !String(a.join(':')).includes('animation-timeline') && supports(...a); })()",
        ))
        .await
        .unwrap();
        page.reload().await.unwrap();
        wait::for_js_true(
            page,
            "!CSS.supports('animation-timeline: scroll()') && !!document.querySelector('#bars-v')",
            "the reloaded app",
        )
        .await
        .unwrap();
        wait::for_js_true(page, &part("bars-v", V_TRACK, "true"), "the drawn track")
            .await
            .unwrap();

        let content: f64 = page
            .evaluate(format!("{AREA}.scrollHeight"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        page.evaluate(format!("{AREA}.scrollTop = 1e6"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, V_THUMB_PLACED),
            "the thumb at the end of its track",
        )
        .await
        .unwrap();
        // A second scroll to the end reads the range the first one left.
        page.evaluate(format!("{AREA}.scrollTop = 1e6"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "{AREA}.scrollHeight === {content} && \
                 {AREA}.scrollTop + {AREA}.clientHeight >= {content} - 1"
            ),
            "the range as long as the content",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("the doubled thumb").unwrap();
        fixture.close().await.unwrap();
    });
}

/// One 120px wheel notch down over `selector`'s centre, waited until it scrolled.
async fn wheel_step(page: &chromiumoxide::Page, selector: &str) {
    let at = pointer::centre_of(page, selector).await.unwrap();
    let top = format!("document.querySelector('{selector}').scrollTop");
    let before: f64 = page
        .evaluate(top.as_str())
        .await
        .unwrap()
        .into_value()
        .unwrap();
    page.execute(
        DispatchMouseEventParams::builder()
            .r#type(DispatchMouseEventType::MouseWheel)
            .x(at.x)
            .y(at.y)
            .delta_x(0.0)
            .delta_y(120.0)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();
    wait::for_js_true(page, &format!("{top} !== {before}"), "the wheel to scroll")
        .await
        .unwrap();
}

/// A scroll redraws nothing under scroll timelines (1954): a resize between two scrolls
/// must still bring the thumb's size and travel up to date.
#[test]
fn a_resize_between_scrolls_keeps_the_thumb_on_its_track() {
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &part("bars-v", V_THUMB, "true"), "the drawn thumb")
            .await
            .unwrap();

        wheel_step(page, "#bars-v").await;
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, V_THUMB_PLACED),
            "the thumb after the first wheel step",
        )
        .await
        .unwrap();
        page.evaluate("document.querySelector('#bars-pane').style.height = '300px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_TRACK, "Math.abs(p.height - 300) < 1"),
            "the track to follow the pane",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, V_THUMB_PLACED),
            "the resized thumb in place",
        )
        .await
        .unwrap();

        wheel_step(page, "#bars-v").await;
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, V_THUMB_PLACED),
            "the resized thumb after the second wheel step",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("a resize between scrolls")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `Always` hides the native bar and draws its own, hidden from assistive technology and
/// never a tab stop. Padding or not, the track stays in the scrollport's end corner while
/// the thumb travels (1010).
#[test]
fn always_draws_its_own_bar_pinned_to_the_corner() {
    const AREA: &str = "document.querySelector('#bars-v')";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // 40 rows of 20px plus 32px padding in 120px: the thumb's share is under the floor.
        let pinned = "Math.abs(p.top) < 1 && Math.abs(p.right) < 1 && Math.abs(p.height - 120) < 1";
        wait::for_js_true(page, &part("bars-v", V_TRACK, pinned), "the drawn track")
            .await
            .unwrap();
        let hidden: bool = page
            .evaluate(format!(
                "getComputedStyle({AREA}).scrollbarWidth === 'none' \
                 && {AREA}.querySelector('[data-slot=scrollbars]').getAttribute('aria-hidden') === 'true' \
                 && [...{AREA}.querySelectorAll('[data-slot=scrollbars] *')].every(e => e.tabIndex < 0)"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            hidden,
            "the native bar shows, or the drawn one is exposed or focusable"
        );
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, "Math.abs(p.top) < 1 && p.height === 20"),
            "the thumb at the top",
        )
        .await
        .unwrap();

        page.evaluate(format!(
            "{AREA}.scrollTop = ({AREA}.scrollHeight - {AREA}.clientHeight) / 2"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, "Math.abs(p.top - 50) < 1"),
            "the thumb half way",
        )
        .await
        .unwrap();
        page.evaluate(format!("{AREA}.scrollTop = {AREA}.scrollHeight"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_THUMB, "Math.abs(p.bottom) < 1"),
            "the thumb at the bottom",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_TRACK, pinned),
            "the track still in the corner",
        )
        .await
        .unwrap();

        let none: bool = page
            .evaluate("!document.querySelector('#bars-hover [data-slot=scrollbars]')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(none, "`hover` drew a bar");

        fixture.console.assert_clean("the drawn bar").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Dragging the thumb scrolls by its share of the range; a press on the track brings the
/// thumb to the pointer; a taller pane re-measures the track.
#[test]
fn the_drawn_thumb_drags_and_the_track_takes_a_press() {
    const AREA: &str = "document.querySelector('#bars-v')";
    const RANGE: &str = "(document.querySelector('#bars-v').scrollHeight - document.querySelector('#bars-v').clientHeight)";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &part("bars-v", V_THUMB, "true"), "the drawn thumb")
            .await
            .unwrap();

        // Half the thumb's 100px of travel is half the range.
        let from = pointer::centre_of(page, &format!("#bars-v {V_THUMB}"))
            .await
            .unwrap();
        let to = pointer::Point {
            x: from.x,
            y: from.y + 50.0,
        };
        pointer::drag(page, from, to, 5).await.unwrap();
        wait::for_js_true(
            page,
            &format!("Math.abs({AREA}.scrollTop - {RANGE} / 2) < 2"),
            "a 50px drag to scroll half way",
        )
        .await
        .unwrap();

        let track = pointer::centre_of(page, &format!("#bars-v {V_TRACK}"))
            .await
            .unwrap();
        pointer::click_at(
            page,
            pointer::Point {
                x: track.x,
                y: track.y + 55.0,
            },
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{AREA}.scrollTop >= {RANGE} - 1"),
            "a press at the track's end to scroll there",
        )
        .await
        .unwrap();

        page.evaluate("document.querySelector('#bars-pane').style.height = '300px'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &part("bars-v", V_TRACK, "Math.abs(p.height - 300) < 1"),
            "the track to follow the pane",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("dragging the drawn thumb")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Both axes leave the corner free; under RTL the horizontal thumb starts at the right,
/// and a border and padding leave the track on the padding box's edge.
#[test]
fn both_axes_share_the_corner_and_rtl_starts_at_the_right() {
    const RTL: &str = "document.querySelector('#bars-rtl')";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            &part(
                "bars-both",
                V_TRACK,
                "Math.abs(p.height - 112) < 1 && Math.abs(p.right) < 1",
            ),
            "the vertical track short of the corner",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &part(
                "bars-both",
                H_TRACK,
                "Math.abs(p.width - 232) < 1 && Math.abs(p.bottom) < 1 && Math.abs(p.left) < 1",
            ),
            "the horizontal track along the bottom",
        )
        .await
        .unwrap();

        // Inside the 3px border, over the 12px padding.
        wait::for_js_true(
            page,
            &part(
                "bars-rtl",
                H_TRACK,
                &format!(
                    "Math.abs(p.bottom - 3) < 1 && Math.abs(p.right - 3) < 1 \
                     && Math.abs(p.width - {RTL}.clientWidth) < 1"
                ),
            ),
            "the RTL track along the padding box's bottom",
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &part("bars-rtl", H_THUMB, "Math.abs(p.right - 3) < 1"),
            "the RTL thumb at the right",
        )
        .await
        .unwrap();
        // Its share of the track is the padding box's, not the border box's.
        wait::for_js_true(
            page,
            &part(
                "bars-rtl",
                H_THUMB,
                &format!("Math.abs(p.width - {RTL}.clientWidth ** 2 / {RTL}.scrollWidth) < 1"),
            ),
            "the RTL thumb sized by the padding box",
        )
        .await
        .unwrap();
        page.evaluate(format!("{RTL}.scrollLeft = -{RTL}.scrollWidth"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &part("bars-rtl", H_THUMB, "Math.abs(p.left - 3) < 1"),
            "the RTL thumb at the left end",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("two axes and RTL").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The drawn thumb keeps WCAG 1.4.11's 3:1 in both schemes, and forced colours paint it
/// in `CanvasText` rather than dropping it to the page's `Canvas`.
#[test]
fn the_drawn_thumb_has_contrast_and_shows_in_forced_colours() {
    const THUMB: &str =
        "getComputedStyle(document.querySelector('#bars-v [data-slot=thumb]')).backgroundColor";
    block_on(async {
        let fixture = Fixture::open("/scroll-area/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, &part("bars-v", V_THUMB, "true"), "the drawn thumb")
            .await
            .unwrap();

        let contrast = thumb_contrast(THUMB);
        for scheme in ["light", "dark"] {
            page.evaluate(format!(
                "document.documentElement.setAttribute('data-lsx-theme', '{scheme}')"
            ))
            .await
            .unwrap();
            let ratio: f64 = page
                .evaluate(contrast.as_str())
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(ratio >= 3.0, "the drawn {scheme} thumb has {ratio:.2}:1");
        }

        e2e::browser::force_colours(page).await.unwrap();
        let [thumb, canvas]: [String; 2] = page
            .evaluate(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return [{THUMB}, canvas]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(
            thumb, canvas,
            "the thumb is filled with the page's own colour"
        );

        fixture
            .console
            .assert_clean("the drawn thumb's colours")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
