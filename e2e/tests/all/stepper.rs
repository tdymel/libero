//! `Stepper`: a "Continue" inside the current step's content moves the step
//! on, and focus returns to the step the user is now on (todo 406).

use anyhow::Result;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

fn header(index: usize) -> String {
    format!("#stepper-step-{index}")
}

async fn current<D: Driver>(d: &mut D, index: usize, after: &str) -> Result<()> {
    let step = header(index);
    eventually(d, &format!("{step} current after {after}"), async |d| {
        Ok(d.attr(&step, "aria-current").await?.as_deref() == Some("step"))
    })
    .await
}

async fn enter_on_continue_returns_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for (button, landing) in [("#next-0", 1), ("#next-1", 2), ("#finish", 2)] {
        d.focus(button).await?;
        d.press(keyboard::ENTER).await?;
        eventually_focused(d, &header(landing), &format!("Enter on {button}")).await?;
        if button == "#next-0" {
            current(d, 1, "Enter on #next-0").await?;
        }
    }
    Ok(())
}

async fn a_click_on_continue_returns_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#next-0").await?;
    eventually_focused(d, &header(1), "a click on #next-0").await
}

async fn a_done_header_click_moves_back<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#next-0").await?;
    eventually(d, "the second step", async |d| d.exists("#next-1").await).await?;
    d.click(&header(0)).await?;
    current(d, 0, "a click on the first header").await?;
    eventually(d, "the first step's content", async |d| {
        d.exists("#next-0").await
    })
    .await
}

e2e::scenario!(
    moving_on_returns_focus_to_the_current_step_horizontally,
    "/stepper",
    enter_on_continue_returns_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    moving_on_returns_focus_to_the_current_step_vertically,
    "/stepper-vertical",
    enter_on_continue_returns_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_clicked_continue_returns_focus_horizontally,
    "/stepper",
    a_click_on_continue_returns_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_clicked_continue_returns_focus_vertically,
    "/stepper-vertical",
    a_click_on_continue_returns_focus,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_click_on_a_done_step_header_moves_back_there,
    "/stepper",
    a_done_header_click_moves_back
);

fn suite(name: &'static str, route: &'static str) -> Suite {
    Suite::new(name, route)
        .focusable("#next-0")
        .focusable("#stepper-step-0")
        .targets("#next-0")
        .targets("button[data-slot=header]")
        .state(
            "second",
            &[Step::TabTo("#next-0"), Step::Press(keyboard::ENTER)],
            "#next-1",
        )
}

/// Under RTL the vertical rail runs down under the markers, which sit at the
/// right, and the content clears them on that side.
#[test]
fn under_rtl_the_vertical_rail_stays_under_the_markers() {
    block_on(async {
        let fixture = Fixture::open("/stepper-vertical", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "getComputedStyle(document.body).direction === 'rtl'",
            "the page to turn RTL",
        )
        .await
        .unwrap();
        let (off, content): (Vec<f64>, String) = page
            .evaluate(
                "(() => { const items = [...document.querySelectorAll('ol > li')].slice(0, -1); \
                 const off = items.map(li => { \
                   const box = li.getBoundingClientRect(); \
                   const m = li.querySelector('[data-slot=marker]').getBoundingClientRect(); \
                   const rail = getComputedStyle(li, '::before'); \
                   const x = box.left + parseFloat(rail.left) \
                     + (parseFloat(rail.borderLeftWidth) + parseFloat(rail.borderRightWidth)) / 2; \
                   return Math.abs(x - (m.left + m.width / 2)); }); \
                 const c = getComputedStyle(document.querySelector('[data-slot=panel]')); \
                 return [off, c.paddingLeft + ' / ' + c.paddingRight]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!off.is_empty(), "no connector measured");
        assert!(
            off.iter().all(|px| *px < 2.0),
            "rail centre off the marker centre by {off:?}px"
        );
        // The end side keeps only the ring's reach (todo 2464).
        assert!(
            content.starts_with("6px /"),
            "content padding (left / right): {content}"
        );
        fixture
            .console
            .assert_clean("RTL vertical stepper")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    suite("stepper", "/stepper").run();
}

#[test]
fn it_meets_the_baseline_vertically() {
    suite("stepper_vertical", "/stepper-vertical").run();
}

/// Long labels and the two named step lists (todo 1773).
#[test]
fn the_long_and_named_steppers_meet_the_baseline() {
    // Neither page has a header to press, so nothing to tab to or measure.
    Suite::new("stepper-long", "/stepper-long").run();
    Suite::new("stepper-named", "/stepper-named").run();
}

/// WCAG 1.4.11: a pending marker is a ring alone, and the connector into it
/// shows the path; both part from the page at 3:1, as Timeline's (todo 599).
#[test]
fn a_pending_ring_and_connector_part_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/stepper",
        "const item = document.querySelector('ol > li:nth-child(3)');
         const ring = item.querySelector('[data-slot=marker]');
         return [
             ['pending ring on the page', RATIO(CSS(ring, 'borderTopColor'), PAGE(item))],
             ['pending connector on the page', RATIO(CSS(item, 'borderTopColor', '::before'), PAGE(item))],
         ];",
    );
}

/// The page's and each stepper's scroll and client widths, and the widest
/// step header's width.
const WIDTHS: &str = "(() => {
    const steppers = ['#side', '#below', '#vertical', '#plain'].map(id => {
        const s = document.querySelector(id);
        const widest = Math.max(...[...s.querySelectorAll('[data-slot=header]')]
            .map(h => h.getBoundingClientRect().width));
        return [s.scrollWidth, s.clientWidth, widest];
    });
    return [[document.documentElement.scrollWidth, innerWidth, 0], ...steppers];
})()";

/// A label with no break opportunity wraps in its step at 390px and 320px, in every arm:
/// the page never scrolls sideways (1.4.10, 518, 525, 2029).
#[test]
fn a_long_label_wraps_instead_of_widening_the_page() {
    block_on(async {
        let fixture = Fixture::open("/stepper-long", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#vertical-step-1").await.unwrap();
        for width in [390, 320] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, true))
                .await
                .unwrap();
            let widths: Vec<[f64; 3]> = page.evaluate(WIDTHS).await.unwrap().into_value().unwrap();
            // Not `innerWidth`: mobile emulation widens it to the content (todo 2029).
            let viewport = f64::from(width);
            let page_width = widths[0][0];
            assert!(
                page_width <= viewport,
                "{width}px: the page scrolls sideways: {widths:?}"
            );
            // The strip may scroll inside the stepper (todo 2030), but no step is wider than it.
            for (arm, [scroll, client, widest]) in ["side", "below", "vertical", "plain"]
                .iter()
                .zip(&widths[1..])
            {
                assert!(
                    scroll <= client,
                    "{width}px {arm}: the stepper overflows: {widths:?}"
                );
                assert!(
                    widest <= client,
                    "{width}px {arm}: a step is wider than the stepper: {widths:?}"
                );
            }
        }
        fixture.console.assert_clean("a long label").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Per stepper of `/stepper-many`: its scroll and client widths, its strip's, and its first
/// header's flex direction.
const MANY: &str = "(() => ['#side-5', '#below-5', '#side-9', '#below-9'].map(id => {
    const s = document.querySelector(id);
    const ol = s.querySelector('ol');
    const header = getComputedStyle(s.querySelector('[data-slot=header]')).flexDirection;
    return [id, s.scrollWidth, s.clientWidth, ol.scrollWidth, ol.clientWidth, header];
}))()";

type Many = Vec<(String, f64, f64, f64, f64, String)>;

/// Steps that no longer fit once the connectors shrank scroll in their strip rather than
/// split a word; the page never scrolls sideways (1.4.10, 1573, 2030).
#[test]
fn many_steps_shrink_then_scroll_in_their_strip() {
    block_on(async {
        let fixture = Fixture::open("/stepper-many", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#below-9-step-8").await.unwrap();
        for width in [390, 320] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, true))
                .await
                .unwrap();
            let page_width: f64 = page
                .evaluate("document.documentElement.scrollWidth")
                .await
                .unwrap()
                .into_value()
                .unwrap();
            let many: Many = page.evaluate(MANY).await.unwrap().into_value().unwrap();
            assert!(
                page_width <= f64::from(width),
                "{width}px: the page scrolls sideways: {many:?}"
            );
            for (id, scroll, client, ..) in &many {
                assert!(
                    scroll <= client,
                    "{width}px {id}: the stepper overflows: {many:?}"
                );
            }
            // The strip scrolls before a word splits (todo 2030).
            for id in ["#side-5", "#below-5"] {
                let (split, words): (Vec<String>, usize) = page
                    .evaluate(SPLIT_WORDS.replace("#plain", id))
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                assert!(words > 0, "{width}px {id}: no label word examined");
                assert!(
                    split.is_empty(),
                    "{width}px {id}: words broken across lines: {split:?}"
                );
            }
        }
        fixture.console.assert_clean("many steps").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Whether `STEP` (a stepper id and step index) sits inside its strip with the strip's
/// `scroll-padding` free on both sides, and the strip has scrolled.
const ROOMY: &str = "(() => {
    const [id, index] = STEP;
    const ol = document.querySelector(`#${id} ol`);
    const h = document.getElementById(`${id}-step-${index}`).getBoundingClientRect();
    const box = ol.getBoundingClientRect();
    const style = getComputedStyle(ol);
    const left = parseFloat(style.scrollPaddingLeft), right = parseFloat(style.scrollPaddingRight);
    return left > 0 && right > 0 && Math.abs(ol.scrollLeft) > 0
        && h.left >= box.left + left - 1 && h.right <= box.right - right + 1;
})()";

fn roomy(id: &str, index: usize) -> String {
    ROOMY.replace("STEP", &format!("['{id}', {index}]"))
}

/// A step that focus scrolls into the strip keeps room for its ring on both sides (todo 2373).
#[test]
fn a_step_scrolled_in_by_focus_keeps_its_ring_room() {
    block_on(async {
        let fixture = Fixture::open("/stepper-many", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#below-9-step-8").await.unwrap();
        page.execute(SetDeviceMetricsOverrideParams::new(320, 800, 1.0, true))
            .await
            .unwrap();
        // From the start, a step comes in on the right; from the end, on the left.
        for (scroll, index) in [("0", 6), ("ol.scrollWidth", 2)] {
            page.evaluate(format!(
                "(() => {{ const ol = document.querySelector('#below-9 ol'); \
                 ol.scrollLeft = {scroll}; \
                 document.getElementById('below-9-step-{index}').focus(); }})()"
            ))
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &roomy("below-9", index),
                "the focused step's ring room",
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("focus scroll").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The current step scrolls into the strip on mount and when `value` moves from outside,
/// with no focus move (todo 2374).
#[test]
fn the_current_step_scrolls_into_its_strip() {
    block_on(async {
        let fixture = Fixture::open("/stepper-many", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#late").await.unwrap();
        wait::for_js_true(page, &roomy("late", 5), "the mounted current step in view")
            .await
            .unwrap();
        page.evaluate("document.getElementById('late-last').click()")
            .await
            .unwrap();
        wait::for_js_true(page, &roomy("late", 8), "the moved current step in view")
            .await
            .unwrap();
        let focus: bool = page
            .evaluate("!document.querySelector('#late')?.contains(document.activeElement)")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(focus, "the reveal moved focus into the stepper");
        fixture.console.assert_clean("current step reveal").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Side labels stack under the width their step count needs, not a fixed 360px (todo 1573).
#[test]
fn the_side_fallback_follows_the_step_count() {
    block_on(async {
        let fixture = Fixture::open("/stepper-many", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#side-5-step-4").await.unwrap();
        for (width, five, nine) in [(500, "column", "column"), (1280, "row", "row")] {
            page.execute(SetDeviceMetricsOverrideParams::new(width, 800, 1.0, false))
                .await
                .unwrap();
            let many: Many = page.evaluate(MANY).await.unwrap().into_value().unwrap();
            assert_eq!(many[0].5, five, "{width}px: {many:?}");
            assert_eq!(many[2].5, nine, "{width}px: {many:?}");
        }
        fixture.close().await.unwrap();
    });
}

/// An overflowing strip with no clickable step is a tab stop; one with buttons, or
/// that fits, is not (WCAG 2.1.1, todo 2375).
#[test]
fn a_scrolling_strip_without_buttons_takes_a_tab_stop() {
    block_on(async {
        let fixture = Fixture::open("/stepper-many", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#plain-9-step-0").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#plain-9 > ol').getAttribute('tabindex') === '0'",
            "the overflowing plain strip to take a tab stop",
        )
        .await
        .unwrap();
        let clickable: bool = page
            .evaluate("document.querySelector('#side-9 > ol').hasAttribute('tabindex')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(!clickable, "a strip of buttons took a tab stop");

        page.execute(SetDeviceMetricsOverrideParams::new(1600, 800, 1.0, false))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "!document.querySelector('#plain-9 > ol').hasAttribute('tabindex')",
            "the fitting plain strip to drop its tab stop",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the plain strip").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Words of the `#plain` stepper's labels and descriptions split across lines, and how many
/// words it examined: a label without a direct text node is skipped (todo 1829).
const SPLIT_WORDS: &str = "(() => {
    const split = [];
    let words = 0;
    for (const el of document.querySelectorAll('#plain [data-slot=label] > :not([aria-hidden]), #plain [data-slot=description], #plain [data-slot=label]:not(:has(*))')) {
        const text = el.firstChild;
        if (!text || text.nodeType !== 3) continue;
        for (const m of text.data.matchAll(/\\S+/g)) {
            const r = document.createRange();
            r.setStart(text, m.index);
            r.setEnd(text, m.index + m[0].length);
            words++;
            if (r.getClientRects().length > 1) split.push(m[0]);
        }
    }
    return [split, words];
})()";

/// Three ordinary steps with descriptions fit a 320px page in the side arm
/// without breaking a word (1.4.10, todo 525).
#[test]
fn ordinary_labels_fit_320px_whole() {
    block_on(async {
        let fixture = Fixture::open("/stepper-long", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#plain-step-1").await.unwrap();
        page.execute(SetDeviceMetricsOverrideParams::new(320, 800, 1.0, true))
            .await
            .unwrap();
        let (split, words): (Vec<String>, usize) = page
            .evaluate(SPLIT_WORDS)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(words > 0, "no label or description word examined");
        assert!(split.is_empty(), "words broken across lines: {split:?}");
        fixture.close().await.unwrap();
    });
}

/// Side labels in a box under 360px stack under the marker with whole words; the
/// full-width `#side` keeps them beside it (542).
#[test]
fn side_labels_stack_in_a_narrow_box() {
    block_on(async {
        let fixture = Fixture::open("/stepper-long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#boxed-step-1").await.unwrap();
        let directions: Vec<String> = page
            .evaluate(
                "['#boxed-step-0', '#side-step-0'].map(s => \
                 getComputedStyle(document.querySelector(s)).flexDirection)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(directions, ["column", "row"]);
        let (split, words): (Vec<String>, usize) = page
            .evaluate(SPLIT_WORDS.replace("#plain", "#boxed"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(words > 0, "no label or description word examined");
        assert!(split.is_empty(), "words broken across lines: {split:?}");
        fixture.close().await.unwrap();
    });
}

/// Todo 540: the current marker differed from a pending one by colour only.
#[test]
fn the_current_marker_carries_the_on_state_ring() {
    block_on(async {
        let fixture = Fixture::open("/stepper-long", Viewport::Desktop)
            .await
            .unwrap();
        crate::button::assert_on_marker(
            &fixture.page,
            "#plain-step-1 [data-slot=marker]",
            "#plain-step-2 [data-slot=marker]",
        )
        .await;
        fixture.close().await.unwrap();
    });
}

/// In forced colours the completed fill and the current ring must still differ from a
/// pending step's (524).
#[test]
fn completed_and_current_steps_show_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/stepper-long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::browser::force_colours(page).await.unwrap();
        let [completed, canvas, current, pending]: [String; 4] = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const marker = (i) => getComputedStyle(document.querySelector(`#plain-step-${i} [data-slot=marker]`)); \
                 return [marker(0).backgroundColor, canvas, marker(1).borderTopColor, marker(2).borderTopColor]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(
            completed, canvas,
            "the completed marker's fill is the page's own"
        );
        assert_ne!(
            current, pending,
            "the current ring looks like a pending one"
        );
        // Todo 715: forcing drops the on-state `box-shadow`; an outline stands in.
        let outlines: [String; 2] = page
            .evaluate(
                "[1, 2].map(i => getComputedStyle(document.querySelector(`#plain-step-${i} [data-slot=marker]`)).outlineStyle)",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            outlines,
            ["solid", "none"],
            "the current marker's shape cue under forced colours"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 541: the caller's name reaches the step list, by label and by heading.
#[test]
fn the_step_list_takes_the_callers_name() {
    block_on(async {
        let fixture = Fixture::open("/stepper-named", Viewport::Desktop)
            .await
            .unwrap();
        for (root, name) in [("#labelled", "Checkout"), ("#labelledby", "Onboarding")] {
            let tree = e2e::ax::snapshot(&fixture.page, &format!("{root} > ol"))
                .await
                .unwrap();
            assert!(tree.starts_with(&format!("list \"{name}\"")), "{tree}");
        }
        fixture.console.assert_clean("named steppers").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2464: in a vertical step's clipped panel an unbreakable word wraps, a 600px child
/// scrolls at 390px, and the first child's ring stays inside the clip.
#[test]
fn a_vertical_panel_keeps_wide_content_and_rings_usable() {
    block_on(async {
        let fixture = Fixture::open("/stepper-wide", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#wide-child").await.unwrap();
        let widths: Vec<f64> = e2e::js(
            page,
            "(() => {
                const panel = document.querySelector(\"#wide [data-slot='panel']\");
                const word = document.querySelector('#word');
                panel.scrollLeft = 1e4;
                const scrolled = panel.scrollLeft;
                panel.scrollLeft = 0;
                return [document.documentElement.scrollWidth, innerWidth,
                        word.scrollWidth, word.clientWidth, scrolled];
            })()",
        )
        .await;
        let [page_width, viewport, word, word_box, scrolled] = widths[..] else {
            panic!("{widths:?}");
        };
        assert!(
            page_width <= viewport,
            "the page scrolls sideways: {widths:?}"
        );
        assert!(word <= word_box, "the word overflows: {widths:?}");
        assert!(
            scrolled > 0.0,
            "the wide child cannot be scrolled to: {widths:?}"
        );
        keyboard::tab_to(page, "#first", 10).await.unwrap();
        let clipped: f64 = e2e::js(page, &crate::image_list::ring_clipped("#wide")).await;
        assert!(
            clipped <= 0.0,
            "the ring runs {clipped}px past the panel's clip"
        );
        fixture
            .console
            .assert_clean("a wide vertical panel")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
