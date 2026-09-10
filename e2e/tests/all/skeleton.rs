//! `Skeleton`: the pulse stops under reduced motion and the grey stays drawn.

use e2e::browser::block_on;
use e2e::passes::{keyboard, motion, pointer};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, wait};

const RELOAD: &str = "#reload";
const PROFILE: &str = "#profile";

#[test]
fn it_meets_the_baseline() {
    Suite::new("skeleton", "/skeleton")
        .focusable(RELOAD)
        .targets(RELOAD)
        // Keys, not a click: a resting pointer measures the hover colours.
        // Not the skeleton itself: its root is `visibility: hidden` while it covers.
        .state(
            "loading",
            &[Step::TabTo(RELOAD), Step::Press(keyboard::ENTER)],
            "#profile[aria-busy=true]",
        )
        .run();
}

/// Watches every DOM change from here on: `window.__grey` is the ms after the
/// watch started at which a grey was first shown, `null` if never. Mutations,
/// not frames: a `background: true` page throttles `requestAnimationFrame`.
const WATCH_GREY: &str = "(() => {
    const start = performance.now();
    window.__grey = null;
    const check = () => {
        const card = document.querySelector('#card[data-state~=visible]');
        const region = document.querySelector('#region');
        if (card && region && getComputedStyle(region).opacity !== '0' && window.__grey === null)
            window.__grey = performance.now() - start;
    };
    new MutationObserver(check).observe(document.body, { subtree: true, childList: true, attributes: true });
})()";

/// Todo 107's docs recipe: a 50 ms fetch never paints the grey, a 1 s one
/// paints it once the 200 ms grace is over. The slow run is the control that
/// shows the watch can see a grey at all.
#[test]
fn the_grace_recipe_keeps_a_fast_fetch_from_flashing() {
    block_on(async {
        for (route, flashes) in [
            ("/skeleton-grace/fast", false),
            ("/skeleton-grace/slow", true),
        ] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, "#load", 3).await.unwrap();
            page.evaluate(WATCH_GREY).await.unwrap();
            keyboard::press(page, keyboard::ENTER).await.unwrap();

            if flashes {
                wait::for_js_true(page, "window.__grey !== null", "the slow fetch's grey")
                    .await
                    .unwrap();
                let at: f64 = page
                    .evaluate("window.__grey")
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                assert!(at >= 190.0, "the grey showed at {at} ms, inside the grace");
            } else {
                wait::for_js_true(
                    page,
                    "!document.querySelector('#card[data-state~=visible]')",
                    "the fast fetch to answer",
                )
                .await
                .unwrap();
                // Past the grace, so a late grey would have been sampled too.
                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                // `-1` for never: a `null` result reads back as no value at all.
                let at: f64 = page
                    .evaluate("window.__grey === null ? -1 : window.__grey")
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                assert_eq!(at, -1.0, "a 50 ms fetch flashed the grey at {at} ms");
            }

            fixture
                .console
                .assert_clean(&format!("the grace recipe at {route}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The opacity each skeleton's `::after` is drawn at, as `[id, opacity]`.
const GREYS: &str = "[...document.querySelectorAll('#profile [data-state~=visible]')]\
     .map(el => [el.id, getComputedStyle(el, '::after').opacity])";

/// The pulse runs, then stops under reduced motion on a grey that is still
/// there: stopped at `0.7`, not at the invisible end of the pulse.
///
/// Both settings explicitly, since headless Chromium defaults to reduced: the
/// animated reading is what proves the reduced one measured a real pulse.
#[test]
fn reduced_motion_stops_the_pulse_and_keeps_the_grey() {
    block_on(async {
        for reduced in [false, true] {
            let fixture = Fixture::open("/skeleton", Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            motion::set_reduced_motion(page, reduced).await.unwrap();
            if reduced {
                motion::assert_reduced_motion_matches(page).await.unwrap();
            }

            pointer::click(page, RELOAD).await.unwrap();
            wait::for_js_true(
                page,
                "document.querySelectorAll('#profile [aria-hidden=true][inert]').length === 2",
                "Reload to cover both skeletons",
            )
            .await
            .unwrap();

            let still = motion::assert_still(page, PROFILE).await;
            if reduced {
                still.expect("the loading skeletons under reduced motion");
                let greys: Vec<(String, String)> =
                    page.evaluate(GREYS).await.unwrap().into_value().unwrap();
                assert_eq!(greys.len(), 2, "{greys:?}");
                for (id, opacity) in greys {
                    assert_eq!(opacity, "0.7", "#{id}'s grey under reduced motion");
                }
            } else {
                let error = still.expect_err("the skeleton should pulse without reduced motion");
                assert!(error.to_string().contains("::after: animation"), "{error}");
            }

            // Loaded: the content shows, and nothing moves either way.
            pointer::click(page, RELOAD).await.unwrap();
            wait::for_visible(page, "#name-text").await.unwrap();
            motion::assert_still(page, PROFILE)
                .await
                .expect("the loaded skeletons");

            fixture
                .console
                .assert_clean(&format!("the skeleton pulse, reduced={reduced}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}
