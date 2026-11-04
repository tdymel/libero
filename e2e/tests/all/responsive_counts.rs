//! Responsive counts and layouts (todos 5688-5692): Pagination siblings, Carousel per_view and
//! slides_to_scroll, Cascader and BarChart layout follow the window across `md`.

use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

/// Pages shown, slides per view, the cascade's shape, and distinct bar columns.
const SHAPE: &str = "(() => { \
    const pages = document.querySelectorAll('#rsc-pages [data-slot=page]').length; \
    const view = document.querySelector('#rsc-carousel [data-slot=viewport]').getBoundingClientRect().width; \
    const slide = document.querySelector('#rsc-carousel [data-slot=slide]').getBoundingClientRect().width; \
    const xs = new Set([...document.querySelectorAll('#rsc-bars [data-chart-drawing] path')] \
        .map(p => Math.round(p.getBBox().x))); \
    return `${pages}|${Math.round(view / slide)}|${xs.size}`; })()";

/// The open cascade: one column per level, or one flat list of paths.
const CASCADE: &str = "(() => { \
    if (!document.querySelector('[role=listbox]')) return 'closed'; \
    return document.querySelector('[data-slot=column]') ? 'columns' : 'paths'; })()";

/// The current slide's position.
const CURRENT: &str = "[...document.querySelectorAll('#rsc-carousel [data-slot=slide]')] \
    .findIndex(s => s.dataset.current === 'true').toString()";

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
fn responsive_counts_and_layouts_follow_the_window() {
    block_on(async {
        let fixture = Fixture::open("/responsive-counts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        // 1 … 8 9 10 11 12 … 20, three slides a view, grouped bars side by side.
        reads(page, SHAPE, "7|3|4").await;
        page.evaluate_bounded(
            "[...document.querySelectorAll('#rsc-carousel [data-slot=control]')].at(-1).click()",
        )
        .await
        .unwrap();
        reads(page, CURRENT, "3").await;
        page.find_element("[role=combobox]")
            .await
            .unwrap()
            .click()
            .await
            .unwrap();
        reads(page, CASCADE, "columns").await;

        narrow(page).await;
        // 1 … 10 … 20, one slide a view, stacked bars sharing a column.
        reads(page, SHAPE, "3|1|2").await;
        reads(page, CASCADE, "paths").await;
        fixture.console.assert_clean("responsive counts").unwrap();
        fixture.close().await.unwrap();
    });
}
