//! `Loader`: every variant stops under reduced motion and stays drawn.

use e2e::browser::block_on;
use e2e::passes::{keyboard, motion, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const UPLOAD: &str = "#upload";
const VARIANTS: [&str; 3] = ["#oval", "#bars", "#dots"];

#[test]
fn it_meets_the_baseline() {
    Suite::new("loader", "/loader")
        .focusable(UPLOAD)
        .targets(UPLOAD)
        // Keys, not a click: a resting pointer measures the hover colours.
        .state(
            "busy",
            &[Step::TabTo(UPLOAD), Step::Press(keyboard::ENTER)],
            "#dots",
        )
        .run();
}

/// Forced colours paint every background `Canvas`, so the bars and the dots
/// vanished into the page while the oval's border stayed.
#[test]
fn every_variant_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/loader", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        pointer::click(page, UPLOAD).await.unwrap();
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
        wait::for_visible(page, "#dots").await.unwrap();
        let bare: Vec<String> = page
            .evaluate(
                "(() => {
                    const page = getComputedStyle(document.body).backgroundColor;
                    const oval = getComputedStyle(document.querySelector('#oval'), '::after');
                    const rows = [['#oval', oval.borderTopColor !== page]];
                    for (const el of document.querySelectorAll('#bars > span, #dots > span'))
                        rows.push([el.parentElement.id, getComputedStyle(el).backgroundColor !== page]);
                    return rows.filter(([, shows]) => !shows).map(([what]) => what);
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bare.is_empty(), "invisible in forced colours: {bare:?}");
        fixture.close().await.unwrap();
    });
}

/// What each variant draws with, as `[what, opacity, transform]`: the oval's
/// ring on `::after`, one row per bar or dot.
const INK: &str = "[\
     ['#oval::after', getComputedStyle(document.querySelector('#oval'), '::after')],\
     ...[...document.querySelectorAll('#bars > span, #dots > span')]\
        .map((el, i) => [`${el.parentElement.id} > span ${i % 3 + 1}`, getComputedStyle(el)])\
   ].map(([what, s]) => [what, s.opacity, s.transform])";

/// `bars` starts at `opacity: 0`, so a bare `animation: none` would hide it: reduced means
/// still and drawn. Both settings are explicit; headless Chromium defaults to reduced.
#[test]
fn reduced_motion_stops_every_variant_and_keeps_it_drawn() {
    block_on(async {
        for reduced in [false, true] {
            let fixture = Fixture::open("/loader", Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            motion::set_reduced_motion(page, reduced).await.unwrap();
            if reduced {
                motion::assert_reduced_motion_matches(page).await.unwrap();
            }

            pointer::click(page, UPLOAD).await.unwrap();
            for variant in VARIANTS {
                wait::for_visible(page, variant).await.unwrap();
                let still = motion::assert_still(page, variant).await;
                match reduced {
                    true => still.unwrap_or_else(|e| panic!("{variant}: {e}")),
                    false => {
                        still.expect_err(&format!(
                            "{variant} should animate without reduced motion"
                        ));
                    }
                }
            }

            if reduced {
                let ink: Vec<(String, String, String)> =
                    page.evaluate(INK).await.unwrap().into_value().unwrap();
                assert_eq!(ink.len(), 7, "{ink:?}");
                for (what, opacity, transform) in ink {
                    assert_eq!(opacity, "1", "{what} under reduced motion");
                    assert!(
                        transform == "none" || transform == "matrix(1, 0, 0, 1, 0, 0)",
                        "{what} is drawn at {transform} under reduced motion"
                    );
                }
            }

            fixture
                .console
                .assert_clean(&format!("the loaders, reduced={reduced}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
