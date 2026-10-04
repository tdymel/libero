//! `ProgressBar`: under reduced motion the fill jumps instead of easing, and
//! the indeterminate sweep becomes a dimmed full track.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::{motion, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const UPLOAD: &str = "#upload";
const SYNC: &str = "#sync";
const ADD: &str = "#add";

#[test]
fn it_meets_the_baseline() {
    Suite::new("progress_bar", "/progress-bar")
        .focusable(ADD)
        .targets(ADD)
        .run();
}

/// Todo 2167: three segments 2px apart, as wide as their spans, the middle one half
/// filled from the start side; under RTL the first segment and each fill start at the right.
async fn segments_fill<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for (bar, rtl) in [("#segments", false), ("#segments-rtl", true)] {
        let nth = |n: usize| format!("{bar} [data-slot=segment]:nth-child({n})");
        let fill = format!("{} > [data-slot=segment-fill]", nth(2));
        eventually(
            d,
            &format!("{bar}: three segments, the middle half filled"),
            async |d| {
                let (first, second, third) = (
                    d.rect(&nth(1)).await?,
                    d.rect(&nth(2)).await?,
                    d.rect(&nth(3)).await?,
                );
                let filled = d.rect(&fill).await?;
                let (left, right) = if rtl {
                    (&third, &first)
                } else {
                    (&first, &third)
                };
                let gap = second.x - (left.x + left.width);
                let from_start = match rtl {
                    false => filled.x - second.x,
                    true => (second.x + second.width) - (filled.x + filled.width),
                };
                Ok((gap - 2.0).abs() < 1.0
                    && (right.x - (second.x + second.width) - 2.0).abs() < 1.0
                    && (second.width / first.width - 2.0).abs() < 0.1
                    && (filled.width / second.width - 0.5).abs() < 0.05
                    && from_start.abs() < 1.0)
            },
        )
        .await?;
    }
    Ok(())
}

e2e::scenario!(
    a_segmented_bar_fills_up_to_the_value,
    "/progress-bar/segments",
    segments_fill
);

/// Todo 2185, WCAG 1.4.11: a segment's fill keeps 3:1 on its track and on the page
/// in the gaps, and forced colours draw each segment's edge.
#[test]
fn the_segments_keep_their_contrast() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/progress-bar/segments", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelectorAll('#segments [data-slot=segment-fill]').length === 3",
            "the segments",
        )
        .await
        .unwrap();
        let ratios = format!(
            r#"(() => {{ {COLOUR_JS}
            const style = (s) => getComputedStyle(document.querySelector('#segments ' + s));
            const base = OVER(RGBA(getComputedStyle(document.body).backgroundColor), [255, 255, 255, 1]);
            const track = OVER(RGBA(style('[data-slot=segment]').backgroundColor), base);
            const fill = OVER(RGBA(style('[data-slot=segment-fill]').backgroundColor), track);
            return [CONTRAST(fill, track), CONTRAST(fill, base)]; }})()"#
        );
        let [on_track, on_page]: [f64; 2] = page
            .evaluate(ratios.as_str())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            on_track >= 3.0,
            "a segment's fill is {on_track:.2}:1 on its track"
        );
        assert!(
            on_page >= 3.0,
            "a segment's fill is {on_page:.2}:1 on the page"
        );
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "[...document.querySelectorAll('#segments [data-slot=segment]')].every((s) => {
                 const style = getComputedStyle(s);
                 return style.outlineStyle === 'solid' && style.outlineColor !== 'rgba(0, 0, 0, 0)'
                     && getComputedStyle(s.firstElementChild).backgroundColor !== style.backgroundColor; })",
            "an edge on every segment, a fill apart from it",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `[fill width / track width, fill opacity]` for the bar at `selector`.
fn fill(selector: &str) -> String {
    format!(
        "(() => {{ const track = document.querySelector('{selector}'); \
         const fill = track.firstElementChild; \
         return [fill.getBoundingClientRect().width / track.getBoundingClientRect().width, \
                 Number(getComputedStyle(fill).opacity)]; }})()"
    )
}

/// Forced colours paint every background `Canvas`, so the track and the fill
/// vanished into the page (todo 506).
#[test]
fn track_and_fill_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/progress-bar", Viewport::Desktop)
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
        let bare: Vec<String> = page
            .evaluate(format!(
                "(() => {{
                    const track = document.querySelector('{UPLOAD}');
                    const bg = el => getComputedStyle(el).backgroundColor;
                    const outlined = el => {{
                        const s = getComputedStyle(el);
                        return s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) >= 1
                            && s.outlineColor !== 'rgba(0, 0, 0, 0)';
                    }};
                    return [
                        ['track', outlined(track) || bg(track) !== bg(document.body)],
                        ['fill', bg(track.firstElementChild) !== bg(track)],
                    ].filter(([, shows]) => !shows).map(([name]) => name);
                }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bare.is_empty(), "invisible in forced colours: {bare:?}");
        fixture.close().await.unwrap();
    });
}

/// Both settings explicitly, since headless Chromium defaults to reduced: the
/// animated reading is what proves the reduced one measured real motion.
#[test]
fn reduced_motion_stops_the_ease_and_the_sweep() {
    block_on(async {
        for reduced in [false, true] {
            let fixture = Fixture::open("/progress-bar", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            motion::set_reduced_motion(page, reduced).await.unwrap();
            if reduced {
                motion::assert_reduced_motion_matches(page).await.unwrap();
            }

            for bar in [UPLOAD, SYNC] {
                let still = motion::assert_still(page, bar).await;
                match reduced {
                    true => still.unwrap_or_else(|e| panic!("{bar}: {e}")),
                    false => {
                        still.expect_err(&format!("{bar} should move without reduced motion"));
                    }
                }
            }

            if reduced {
                // No 25% stub parked at the left, which would read as a value.
                let (share, opacity): (f64, f64) = page
                    .evaluate(fill(SYNC))
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                assert!(
                    (share - 1.0).abs() < 0.01,
                    "the sweep froze at {share} of the track"
                );
                assert_eq!(opacity, 0.5);

                // The fill is at the new value on the first read after the
                // value lands: no ease to wait out.
                pointer::click(page, ADD).await.unwrap();
                wait::for_js_true(
                    page,
                    &format!(
                        "document.querySelector('{UPLOAD}').getAttribute('aria-valuenow') === '50'"
                    ),
                    "Add 10% to reach 50",
                )
                .await
                .unwrap();
                let (share, _): (f64, f64) = page
                    .evaluate(fill(UPLOAD))
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                assert!((share - 0.5).abs() < 0.01, "the fill is at {share}, easing");
            }

            fixture
                .console
                .assert_clean(&format!("the progress bars, reduced={reduced}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
