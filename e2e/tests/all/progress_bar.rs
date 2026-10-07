//! `ProgressBar`: under reduced motion the fill jumps instead of easing, and
//! the indeterminate sweep becomes a striped full track.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::{motion, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const UPLOAD: &str = "#upload";
const SYNC: &str = "#sync";
const SYNC_RTL: &str = "#sync-rtl";
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
        // Todo 2403: the value at 50 is in the "Send" stretch, said after the percentage.
        let text: String = page
            .evaluate("document.querySelector('#segments').getAttribute('aria-valuetext')")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(text, "50%, Send");
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

/// Todo 2402, WCAG 1.4.11: the reduced-motion bar is stripes of the full fill
/// colour, so each stripe keeps 3:1 on the track and on the page.
async fn striped_fill_keeps_its_contrast(page: &chromiumoxide::Page) {
    let probe = format!(
        r#"(() => {{ {COLOUR_JS}
        const track = document.querySelector('{SYNC}');
        const fill = getComputedStyle(track.firstElementChild);
        const base = OVER(RGBA(getComputedStyle(document.body).backgroundColor), [255, 255, 255, 1]);
        const under = OVER(RGBA(getComputedStyle(track).backgroundColor), base);
        const paint = OVER(RGBA(fill.backgroundColor), under);
        return [fill.maskImage.startsWith('repeating-linear-gradient') ? 1 : 0,
                CONTRAST(paint, under), CONTRAST(paint, base)]; }})()"#
    );
    let [striped, on_track, on_page]: [f64; 3] = page
        .evaluate(probe.as_str())
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(striped, 1.0, "the reduced-motion fill is not striped");
    assert!(on_track >= 3.0, "a stripe is {on_track:.2}:1 on its track");
    assert!(on_page >= 3.0, "a stripe is {on_page:.2}:1 on the page");
}

/// Todo 2401: the sweep enters from the start side, so under RTL its first frame
/// sits past the right edge, not at half way.
async fn sweep_starts_off_the_track(page: &chromiumoxide::Page) {
    for (bar, rtl) in [(SYNC, false), (SYNC_RTL, true)] {
        let probe = format!(
            "(() => {{ const track = document.querySelector('{bar}');
                const fill = track.firstElementChild;
                const sweep = fill.getAnimations()[0];
                sweep.pause(); sweep.currentTime = 0;
                const t = track.getBoundingClientRect(), f = fill.getBoundingClientRect();
                return [f.right - t.left, t.right - f.left]; }})()"
        );
        let [inside_from_left, inside_from_right]: [f64; 2] = page
            .evaluate(probe.as_str())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let inside = if rtl {
            inside_from_right
        } else {
            inside_from_left
        };
        assert!(
            inside <= 0.5,
            "{bar}: the sweep's first frame overlaps the track by {inside:.1}px"
        );
    }
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

            for bar in [UPLOAD, SYNC, SYNC_RTL] {
                let still = motion::assert_still(page, bar).await;
                match reduced {
                    true => still.unwrap_or_else(|e| panic!("{bar}: {e}")),
                    false => {
                        still.expect_err(&format!("{bar} should move without reduced motion"));
                    }
                }
            }

            if !reduced {
                sweep_starts_off_the_track(page).await;
            }

            if reduced {
                // No 25% stub parked at the left, which would read as a value.
                for bar in [SYNC, SYNC_RTL] {
                    let (share, opacity): (f64, f64) = page
                        .evaluate(fill(bar))
                        .await
                        .unwrap()
                        .into_value()
                        .unwrap();
                    assert!(
                        (share - 1.0).abs() < 0.01,
                        "{bar}: the sweep froze at {share} of the track"
                    );
                    assert_eq!(opacity, 1.0, "{bar}: a blended fill drops under 3:1");
                }
                striped_fill_keeps_its_contrast(page).await;

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
