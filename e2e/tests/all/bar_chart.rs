//! `BarChart`: drawn at its box's size, categories top down when lying, values on bars.

use e2e::{Fixture, Suite, Viewport, wait};

#[test]
fn it_meets_the_baseline() {
    Suite::new("bar_chart", "/bar-chart").run();
}

/// Once measured, the SVG is drawn again at the plot's width: text keeps its size.
#[test]
fn the_bars_are_drawn_at_the_plot_s_width() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            "document.querySelector('#chart svg')?.getAttribute('viewBox')?.startsWith('0 0 480 ')",
            "the chart drawn at 480 px",
        )
        .await
        .unwrap();
        // A gap in a series draws no bar.
        let bars: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart svg rect').length",
        )
        .await;
        assert_eq!(bars, 5);
        let rows: usize = e2e::js(
            &fixture.page,
            "document.querySelectorAll('#chart tbody tr').length",
        )
        .await;
        assert_eq!(rows, 3);
        fixture.close().await.unwrap();
    });
}

/// Lying, the first category is the top row, a stack's bars share it end to end, and
/// each value is written on its bar.
#[test]
fn stacked_lying_bars_read_top_down() {
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/bar-chart/stacked", Viewport::Desktop)
            .await
            .unwrap();
        wait::for_visible(&fixture.page, "#chart svg rect")
            .await
            .unwrap();
        let [q1, q3]: [f64; 2] = e2e::js(
            &fixture.page,
            "['Q1', 'Q3'].map(q => [...document.querySelectorAll('#chart svg text')] \
             .find(t => t.textContent.trim() === q).getBBox().y)",
        )
        .await;
        assert!(q1 < q3, "Q1 at {q1}, Q3 at {q3}");
        // The renderer draws the rows bottom up: Q1 is each series' last bar.
        let [a, b]: [[f64; 3]; 2] = e2e::js(
            &fixture.page,
            "(() => { const r = [...document.querySelectorAll('#chart svg rect')] \
             .map(r => [r.x.baseVal.value, r.y.baseVal.value, r.width.baseVal.value]); \
             return [r[2], r[4]]; })()",
        )
        .await;
        assert_eq!(a[1], b[1], "{a:?} {b:?}");
        assert!((a[0] + a[2] - b[0]).abs() < 1.0, "{a:?} {b:?}");
        let labels: Vec<String> = e2e::js(
            &fixture.page,
            "[...document.querySelectorAll('#chart svg text')].map(t => t.textContent.trim())",
        )
        .await;
        assert!(labels.contains(&"140".to_string()), "{labels:?}");
        fixture.close().await.unwrap();
    });
}
