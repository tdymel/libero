//! `Stepper`: a "Continue" inside the current step's content moves the step
//! on, and focus returns to the step the user is now on (todo 406).

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::{focus, keyboard};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const ROUTES: [&str; 2] = ["/stepper", "/stepper-vertical"];

fn header(index: usize) -> String {
    format!("#stepper-step-{index}")
}

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

/// A label with no break opportunity wraps inside its step at 390px and at
/// 320px, in every arm: neither the page nor the stepper scrolls sideways
/// (1.4.10, todos 518, 525).
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

/// Forced colours paint every fill `Canvas` and every ring `CanvasText`: the
/// completed marker's fill and the current step's ring must stay apart from a
/// pending step's (todo 524).
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

/// Each "Continue" is unmounted by its own press. Focus lands on the next
/// step's header, which is `aria-current`; "Finish" leaves no current step,
/// so it lands on the step that just closed.
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
