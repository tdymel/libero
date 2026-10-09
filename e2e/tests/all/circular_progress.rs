//! `CircularProgress`: the arc follows the value, the label sits in the ring, and
//! under reduced motion the spin becomes a still dashed ring.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::contrast::COLOUR_JS;
use e2e::passes::motion;
use e2e::{Fixture, Suite, Viewport, wait};

const UPLOAD: &str = "#upload";
const SYNC: &str = "#sync";
const ADD: &str = "#add";

#[test]
fn it_meets_the_baseline() {
    Suite::new("circular_progress", "/circular-progress")
        .focusable(ADD)
        .targets(ADD)
        .run();
}

/// "Add 10%" moves `aria-valuenow` and the arc's offset; the ring stays square
/// with its label in the middle.
async fn arc_follows_the_value<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let circle = format!("{UPLOAD} > [data-slot=arc] > circle");
    let before = d.attr(&circle, "stroke-dashoffset").await?;
    d.click(ADD).await?;
    eventually(d, "the ring at 50", async |d| {
        Ok(
            d.attr(UPLOAD, "aria-valuenow").await?.as_deref() == Some("50")
                && d.attr(&circle, "stroke-dashoffset").await? != before,
        )
    })
    .await?;
    let ring = d.rect(UPLOAD).await?;
    let label = d.rect(&format!("{UPLOAD} > [data-slot=label]")).await?;
    let middle = |x: f64, width: f64| x + width / 2.0;
    anyhow::ensure!(
        (ring.width - ring.height).abs() < 0.5
            && (middle(label.x, label.width) - middle(ring.x, ring.width)).abs() < 1.0
            && (middle(label.y, label.height) - middle(ring.y, ring.height)).abs() < 1.0,
        "ring {ring:?}, label {label:?}"
    );
    Ok(())
}

e2e::scenario!(
    the_arc_follows_the_value,
    "/circular-progress",
    arc_follows_the_value
);

/// `[arc colour, track colour, page colour]` contrast probe: arc on track, arc on page.
fn ratios(selector: &str) -> String {
    format!(
        r#"(() => {{ {COLOUR_JS}
        const ring = document.querySelector('{selector}');
        const base = OVER(RGBA(getComputedStyle(document.body).backgroundColor), [255, 255, 255, 1]);
        const track = OVER(RGBA(getComputedStyle(ring, '::before').borderTopColor), base);
        const arc = OVER(RGBA(getComputedStyle(ring.querySelector('[data-slot=arc]')).color), track);
        return [CONTRAST(arc, track), CONTRAST(arc, base)]; }})()"#
    )
}

/// WCAG 1.4.11: the arc keeps 3:1 on its track and the page, and forced colours
/// keep the track and an arc apart from it.
#[test]
fn the_arc_keeps_its_contrast_and_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/circular-progress", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, "!!document.querySelector('#upload svg')", "the ring")
            .await
            .unwrap();
        let [on_track, on_page]: [f64; 2] = page
            .evaluate(ratios(UPLOAD).as_str())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(on_track >= 3.0, "the arc is {on_track:.2}:1 on its track");
        assert!(on_page >= 3.0, "the arc is {on_page:.2}:1 on the page");

        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!(
                "(() => {{ const ring = document.querySelector('{UPLOAD}');
                    const track = getComputedStyle(ring, '::before').borderTopColor;
                    const arc = getComputedStyle(ring.querySelector('[data-slot=arc]')).color;
                    return matchMedia('(forced-colors: active)').matches
                        && track !== getComputedStyle(document.body).backgroundColor
                        && arc !== track; }})()"
            ),
            "a track and an arc apart from it in forced colours",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Both settings explicitly, since headless Chromium defaults to reduced: the
/// animated reading is what proves the reduced one measured real motion.
#[test]
fn reduced_motion_stops_the_spin_and_dashes_the_ring() {
    let spin = format!("{SYNC} > [data-slot=arc]");
    block_on(async {
        for reduced in [false, true] {
            let fixture = Fixture::open("/circular-progress", Viewport::Desktop)
                .await
                .unwrap();
            let page = &fixture.page;
            motion::set_reduced_motion(page, reduced).await.unwrap();
            if reduced {
                motion::assert_reduced_motion_matches(page).await.unwrap();
            }
            let still = motion::assert_still(page, &spin).await;
            match reduced {
                true => still.unwrap_or_else(|e| panic!("{spin}: {e}")),
                false => {
                    still.expect_err("the spinner should move without reduced motion");
                }
            }
            if reduced {
                // No frozen quarter, which would read as 25% done.
                let dashes: String = page
                    .evaluate(format!(
                        "getComputedStyle(document.querySelector('{spin} > circle')).strokeDasharray"
                    ))
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                // Chromium keeps the `calc()` in the computed value.
                let dash: f64 = dashes
                    .trim_start_matches("calc(")
                    .split(|c: char| !(c.is_ascii_digit() || c == '.'))
                    .next()
                    .and_then(|number| number.parse().ok())
                    .unwrap_or(0.0);
                assert!((dash - 7.0).abs() < 0.1, "the still ring is {dashes:?}");
            }
            fixture
                .console
                .assert_clean(&format!("the rings, reduced={reduced}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
