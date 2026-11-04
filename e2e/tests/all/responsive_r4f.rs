//! Responsive Tree size, Stepper label_position, Tooltip and HoverCard gap and CircularProgress
//! thickness (todos 5676-5678, 5680) follow the window across `md`, as their plain twins.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// One reading per family of `id`: the ring's stroke, the tree's gaps and box, the popover's gap.
const PROBES: &str = "const probes = { \
    ring: id => document.querySelector(`#${id} circle`).getAttribute('stroke-width'), \
    tree: id => { const t = document.querySelector(`#${id}`); \
        const g = t.querySelector('[role=group]'); const box = t.querySelector('[data-slot=box]'); \
        return [getComputedStyle(t).rowGap, getComputedStyle(g).paddingBlockStart, \
            getComputedStyle(g).paddingInlineStart, box.getBoundingClientRect().width].join('/'); }, \
    tip: id => { const b = document.querySelector(`#${id}-label`).getBoundingClientRect(); \
        return Math.round(b.top - document.querySelector(`#${id}-trigger`).getBoundingClientRect().bottom); }, \
    card: id => { const b = document.querySelector(`#${id}`).getBoundingClientRect(); \
        return Math.round(b.top - document.querySelector(`#${id}-trigger`).getBoundingClientRect().bottom); } };";

/// `same` when the responsive element reads as its twin (`narrow` or `wide`), else both readings.
fn against(family: &str, twin: &str) -> String {
    format!(
        "(() => {{ {PROBES} const read = probes.{family}; \
         const a = String(read('r4-{family}')), b = String(read('r4-{family}-{twin}')); \
         return a === b ? 'same' : `${{a}} vs ${{b}}`; }})()"
    )
}

/// `differ` when the narrow and wide twins read differently, so a match above proves something.
fn apart(family: &str) -> String {
    format!(
        "(() => {{ {PROBES} return String(probes.{family}('r4-{family}-narrow')) \
         !== String(probes.{family}('r4-{family}-wide')) ? 'differ' : 'equal'; }})()"
    )
}

/// The stepper's label position word.
const LABEL: &str = "(() => { const s = document.querySelector('#r4-stepper').dataset.state; \
    return s.includes('label-below') ? 'below' : s.includes('label-side') ? 'side' : s; })()";

async fn reads(page: &Page, probe: &str, want: &str) {
    let reached = wait::for_js_true(page, &format!("{probe} === '{want}'"), want).await;
    let now: String = page
        .evaluate_bounded(probe)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert!(reached.is_ok(), "{now}, want {want}");
}

async fn narrow(page: &Page) {
    page.execute_bounded(SetDeviceMetricsOverrideParams::new(900, 800, 1.0, false))
        .await
        .unwrap();
}

#[test]
fn responsive_gaps_thickness_size_and_label_position_follow_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-r4f", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let families = ["ring", "tree", "tip", "card"];
        for family in families {
            reads(page, &apart(family), "differ").await;
            reads(page, &against(family, "wide"), "same").await;
        }
        reads(page, LABEL, "side").await;

        narrow(page).await;
        for family in families {
            reads(page, &against(family, "narrow"), "same").await;
        }
        reads(page, LABEL, "below").await;
        fixture.console.assert_clean("responsive r4f").unwrap();
        fixture.close().await.unwrap();
    });
}
