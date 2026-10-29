//! The docs Chart page's type switch: one control over a line, a bar, a pie, a funnel chart,
//! a heatmap and a candlestick chart. Each chart has its own unit (`line_chart`, `bar_chart`,
//! `pie_chart`, `funnel_chart`, `heatmap_chart`, `candlestick_chart`); here only what the switch adds. Web and Android: charts are parked on Blitz.

use anyhow::{Result, ensure};
use e2e::Suite;
use e2e::driver::{Driver, eventually};

#[test]
fn it_meets_the_baseline() {
    Suite::new("chart", "/chart").run();
}

/// What the chart's hidden summary names: its series or its slices.
const NAMES: &str = "#chart [data-chart-summary]";

async fn shows<D: Driver>(d: &mut D, what: &str, name: &str) -> Result<()> {
    let label = format!("{what}: the summary to name {name}");
    eventually(d, &label, async |d| Ok(d.text(NAMES).await?.contains(name))).await
}

/// Each segment swaps the chart for its type: one chart at a time, the old one gone.
async fn switches<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    shows(d, "the first chart", "This week").await?;
    for (segment, name, gone) in [
        (1, "2025", "This week"),
        (2, "Chrome", "2025"),
        (3, "Visits", "Chrome"),
        (4, "Heatmap of 2 rows", "Visits"),
        (5, "Candlestick chart of 2", "Heatmap"),
        (0, "This week", "Candlestick"),
    ] {
        d.click(&format!(
            "[role=radiogroup] label[for$=\"-segment-{segment}\"]"
        ))
        .await?;
        shows(d, &format!("segment {segment}"), name).await?;
        ensure!(
            !d.text(NAMES).await?.contains(gone),
            "{gone} stayed after segment {segment}"
        );
        let charts = d.rects("#chart").await?.len();
        ensure!(charts == 1, "{charts} charts after segment {segment}");
    }
    Ok(())
}

e2e::scenario!(the_segments_swap_the_chart, "/chart", switches);

/// Todo 4830: under a finger every legend key takes presses 21px above and below its centre.
#[test]
fn a_finger_presses_a_legend_key_in_a_44px_box() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    use e2e::wait::PageBounded;
    use e2e::{Fixture, Viewport, wait};
    e2e::browser::block_on(async {
        let fixture = Fixture::open("/chart", Viewport::Mobile).await.unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();
        for segment in [0, 1, 2] {
            page.evaluate_bounded(format!(
                "document.querySelector('[role=radiogroup] label[for$=\"-segment-{segment}\"]').click()"
            ))
            .await
            .unwrap();
            wait::for_visible(page, "#chart [data-slot=legend] button")
                .await
                .unwrap();
            let misses: Vec<String> = page
                .evaluate_bounded(
                    "(() => [...document.querySelectorAll('#chart [data-slot=legend] button')].flatMap((el) => {
                        el.scrollIntoView({ block: 'center' });
                        const r = el.getBoundingClientRect();
                        const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                        return [-21, 0, 21]
                            .filter((dy) => { const at = document.elementFromPoint(cx, cy + dy); return !at || !el.contains(at); })
                            .map((dy) => `${el.textContent} ${dy}`);
                    }))()",
                )
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(
                misses.is_empty(),
                "segment {segment}: legend points outside their key: {misses:?}"
            );
        }
        fixture.console.assert_clean("the legend").unwrap();
        fixture.close().await.unwrap();
    });
}
