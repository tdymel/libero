//! `Stepper`: a "Continue" inside the current step's content moves the step
//! on, and focus returns to the step the user is now on (todo 406).

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{focus, keyboard};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const ROUTES: [&str; 2] = ["/stepper", "/stepper-vertical"];

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
    android: skip("958: element identity on the WebView")
);
e2e::scenario!(
    moving_on_returns_focus_to_the_current_step_vertically,
    "/stepper-vertical",
    enter_on_continue_returns_focus,
    android: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_clicked_continue_returns_focus_horizontally,
    "/stepper",
    a_click_on_continue_returns_focus,
    android: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_clicked_continue_returns_focus_vertically,
    "/stepper-vertical",
    a_click_on_continue_returns_focus,
    android: skip("958: element identity on the WebView")
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
        .targets("button[data-step-header]")
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
                   const m = li.querySelector('[data-step-marker]').getBoundingClientRect(); \
                   const rail = getComputedStyle(li, '::before'); \
                   const x = box.left + parseFloat(rail.left) \
                     + (parseFloat(rail.borderLeftWidth) + parseFloat(rail.borderRightWidth)) / 2; \
                   return Math.abs(x - (m.left + m.width / 2)); }); \
                 const c = getComputedStyle(document.querySelector('[data-step-content]')); \
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
        assert!(
            content.starts_with("0px /"),
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

/// WCAG 1.4.11: a pending marker is a ring alone, and the connector into it
/// shows the path; both part from the page at 3:1, as Timeline's (todo 599).
#[test]
fn a_pending_ring_and_connector_part_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/stepper",
        "const item = document.querySelector('ol > li:nth-child(3)');
         const ring = item.querySelector('[data-step-marker]');
         return [
             ['pending ring on the page', RATIO(CSS(ring, 'borderTopColor'), PAGE(item))],
             ['pending connector on the page', RATIO(CSS(item, 'borderTopColor', '::before'), PAGE(item))],
         ];",
    );
}

/// The page's and each stepper's scroll and client widths, and the widest
/// step header's right edge.
const WIDTHS: &str = "(() => {
    const steppers = ['#side', '#below', '#vertical', '#plain'].map(id => {
        const s = document.querySelector(id);
        const right = Math.max(...[...s.querySelectorAll('[data-step-header]')]
            .map(h => h.getBoundingClientRect().right));
        return [s.scrollWidth, s.clientWidth, right];
    });
    return [[document.documentElement.scrollWidth, innerWidth, 0], ...steppers];
})()";

/// A label with no break opportunity wraps in its step at 390px and 320px, in every arm:
/// nothing scrolls sideways (1.4.10, 518, 525).
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
            let [page_width, viewport, _] = widths[0];
            assert!(
                page_width <= viewport,
                "{width}px: the page scrolls sideways: {widths:?}"
            );
            for (arm, [scroll, client, right]) in ["side", "below", "vertical", "plain"]
                .iter()
                .zip(&widths[1..])
            {
                assert!(
                    scroll <= client,
                    "{width}px {arm}: the stepper overflows: {widths:?}"
                );
                assert!(
                    *right <= viewport,
                    "{width}px {arm}: a step is off screen: {widths:?}"
                );
            }
        }
        fixture.console.assert_clean("a long label").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Words of the `#plain` stepper's labels and descriptions split across lines.
const SPLIT_WORDS: &str = "(() => {
    const split = [];
    for (const el of document.querySelectorAll('#plain [data-step-label] > :not([aria-hidden]), #plain [data-step-description], #plain [data-step-label]:not(:has(*))')) {
        const text = el.firstChild;
        if (!text || text.nodeType !== 3) continue;
        for (const m of text.data.matchAll(/\\S+/g)) {
            const r = document.createRange();
            r.setStart(text, m.index);
            r.setEnd(text, m.index + m[0].length);
            if (r.getClientRects().length > 1) split.push(m[0]);
        }
    }
    return split;
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
        let split: Vec<String> = page
            .evaluate(SPLIT_WORDS)
            .await
            .unwrap()
            .into_value()
            .unwrap();
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
        let split: Vec<String> = page
            .evaluate(SPLIT_WORDS.replace("#plain", "#boxed"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
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
            "#plain-step-1 [data-step-marker]",
            "#plain-step-2 [data-step-marker]",
        )
        .await;
        fixture.close().await.unwrap();
    });
}

/// In forced colours the completed fill and the current ring must still differ from a
/// pending step's (524).
#[test]
fn completed_and_current_steps_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/stepper-long", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let [completed, canvas, current, pending]: [String; 4] = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const marker = (i) => getComputedStyle(document.querySelector(`#plain-step-${i} [data-step-marker]`)); \
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
                "[1, 2].map(i => getComputedStyle(document.querySelector(`#plain-step-${i} [data-step-marker]`)).outlineStyle)",
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

/// Press `button` from the keyboard, then focus must be on `landing`.
async fn continue_from(page: &Page, button: &str, landing: &str, route: &str) {
    keyboard::tab_to(page, button, 4)
        .await
        .unwrap_or_else(|e| panic!("{route}: {e}"));
    keyboard::press(page, keyboard::ENTER).await.unwrap();
    focus::wait_for_focus(page, landing, &format!("pressing {button} on {route}"))
        .await
        .unwrap();
}

/// Each "Continue" unmounts itself; focus lands on the next, `aria-current` header, and
/// after "Finish" on the step that just closed.
#[test]
fn moving_on_returns_focus_to_the_current_step() {
    block_on(async {
        for route in ROUTES {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;

            continue_from(page, "#next-0", &header(1), route).await;
            let current: Option<String> = page
                .evaluate(format!(
                    "document.querySelector('{}').getAttribute('aria-current')",
                    header(1)
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(current.as_deref(), Some("step"), "{route}");

            continue_from(page, "#next-1", &header(2), route).await;
            continue_from(page, "#finish", &header(2), route).await;

            fixture
                .console
                .assert_clean(&format!("stepping through {route}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
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
