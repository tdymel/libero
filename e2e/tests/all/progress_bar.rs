//! `ProgressBar`: under reduced motion the fill jumps instead of easing, and
//! the indeterminate sweep becomes a dimmed full track.

use e2e::browser::block_on;
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
