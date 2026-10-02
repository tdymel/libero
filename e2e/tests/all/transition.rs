//! `Transition`: enter on mount, exit on `open: false`, and instant under reduced motion.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::{keyboard, motion, pointer};
use e2e::{Fixture, Viewport, clock, wait};

const TOGGLE: &str = "#toggle";
const ROOT: &str = "#panel";
const CONTENT: &str = "#panel-text";

async fn opacity<D: Driver>(d: &mut D) -> Result<f32> {
    Ok(d.style(ROOT, "opacity").await?.parse()?)
}

/// Closed it is unmounted; open it settles at full opacity; closing unmounts it again.
async fn enters_and_exits<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(!d.exists(CONTENT).await?, "closed, it rendered its content");
    d.click(TOGGLE).await?;
    eventually(d, "the open transition to be fully opaque", async |d| {
        Ok(d.exists(CONTENT).await? && opacity(d).await? > 0.99)
    })
    .await?;
    d.click(TOGGLE).await?;
    eventually(
        d,
        "the closed transition to unmount its content",
        async |d| Ok(!d.exists(CONTENT).await?),
    )
    .await
}

e2e::scenario!(
    it_enters_and_unmounts_its_content_once_closed,
    "/transition",
    enters_and_exits
);

/// An omitted `open` renders open at once and enters by the `appear` keyframe, not a state flip.
async fn enters_on_mount<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let state = d.attr(ROOT, "data-state").await?.unwrap_or_default();
    ensure!(state == "open appear", "data-state {state}");
    let animation = d.style(ROOT, "animation-name").await?;
    ensure!(
        animation == "lsx-transition-appear",
        "animation {animation}"
    );
    eventually(d, "the mounted transition to be fully opaque", async |d| {
        Ok(d.exists(CONTENT).await? && opacity(d).await? > 0.99)
    })
    .await
}

e2e::scenario!(
    it_animates_in_on_mount_without_open,
    "/transition-mount",
    enters_on_mount
);

/// Closed it is shrunk to nothing; open it is untransformed, its origin the bottom right corner.
async fn grows_from_its_corner<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let closed = d.style(ROOT, "transform").await?;
    // Chromium resolves the computed transform to a matrix, Blitz keeps the specified `scale(0)`.
    ensure!(
        closed.starts_with("matrix(0, 0, 0, 0") || closed == "scale(0)",
        "closed transform {closed}"
    );
    d.click(TOGGLE).await?;
    eventually(d, "the corner transition to settle open", async |d| {
        Ok(opacity(d).await? > 0.99 && d.style(ROOT, "transform").await? == "none")
    })
    .await?;
    let corner = format!(
        "{} {}",
        d.style(ROOT, "width").await?,
        d.style(ROOT, "height").await?
    );
    let origin = d.style(ROOT, "transform-origin").await?;
    // Blitz keeps the specified percentages and reports width and height as `auto`.
    ensure!(
        origin == corner || origin.starts_with("100% 100%"),
        "origin {origin}, corner {corner}"
    );
    Ok(())
}

e2e::scenario!(
    it_grows_an_origin_kind_from_its_corner,
    "/transition-corner",
    grows_from_its_corner
);

/// A caller's `from` blurs the closed state, animates with the rest, and clears once open.
async fn blurs_from_its_from_state<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let closed = d.style(ROOT, "filter").await?;
    ensure!(closed == "blur(4px)", "closed filter {closed}");
    let properties = d.style(ROOT, "transition-property").await?;
    ensure!(
        properties.contains("filter"),
        "transition-property {properties}"
    );
    d.click(TOGGLE).await?;
    eventually(d, "the from-state to clear once open", async |d| {
        Ok(opacity(d).await? > 0.99 && d.style(ROOT, "filter").await? == "none")
    })
    .await
}

e2e::scenario!(
    it_animates_a_callers_from_state,
    "/transition-from",
    blurs_from_its_from_state
);

/// Under reduced motion nothing transitions. The same check at `no-preference` must see
/// motion first, so green means the reduced arm worked.
#[test]
fn reduced_motion_switches_its_transitions_off() {
    block_on(async {
        let fixture = Fixture::open("/transition", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, TOGGLE).await.unwrap();
        wait::for_visible(page, CONTENT).await.unwrap();
        motion::assert_still(page, ROOT)
            .await
            .expect_err("the transition should animate without reduced motion");

        motion::set_reduced_motion(page, true).await.unwrap();
        motion::assert_reduced_motion_matches(page).await.unwrap();
        motion::assert_still(page, ROOT)
            .await
            .expect("open, under reduced motion");

        pointer::click(page, TOGGLE).await.unwrap();
        wait::for_js_true(
            page,
            &format!("!document.querySelector({CONTENT:?})"),
            "the closed transition to unmount its content",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("the transition").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Finishes what runs, as `Suite` does, and waits for an end event per finished animation.
async fn finish_and_hear(page: &chromiumoxide::Page, what: &str) -> usize {
    page.evaluate("window.__ends = []").await.unwrap();
    let finished = clock::finish_animations(page, std::time::Duration::from_secs(5))
        .await
        .unwrap()
        .finished;
    wait::for_js_true(
        page,
        &format!("window.__ends.length >= {finished}"),
        &format!("{what}: an end event for each of {finished} finished animations"),
    )
    .await
    .unwrap();
    finished
}

/// Todo 1815: an animation `Suite` finishes still sends its end event, which `Collapse`
/// unmounts on and `Toast` and `Modal` animate by. `None` closes with Escape.
#[test]
fn a_finished_animation_still_sends_its_end_event() {
    block_on(async {
        for (route, trigger, shown, close) in [
            (
                "/collapse",
                "#toggle-details",
                "#details-text",
                Some("#toggle-details"),
            ),
            (
                "/notifications",
                "#notify",
                "[aria-live=polite] li",
                Some("[aria-live] li [data-slot=close]"),
            ),
            ("/modal", "#open-modal", "[role=dialog]", None),
        ] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            wait::for_visible(page, trigger).await.unwrap();
            page.evaluate(
                "for (const type of ['transitionend', 'animationend']) document.addEventListener(type, \
                 (e) => window.__ends.push(e.animationName || e.propertyName), true); 1",
            )
            .await
            .unwrap();

            pointer::click(page, trigger).await.unwrap();
            wait::for_selector(page, shown).await.unwrap();
            let opened = finish_and_hear(page, &format!("{route} opening")).await;
            match close {
                Some(selector) => pointer::click(page, selector).await.unwrap(),
                None => {
                    let inside = format!("!!document.activeElement?.closest({shown:?})");
                    wait::for_js_true(page, &inside, "focus in the dialog")
                        .await
                        .unwrap();
                    keyboard::press(page, keyboard::ESCAPE).await.unwrap();
                }
            }
            let closed = finish_and_hear(page, &format!("{route} closing")).await;
            wait::for_js_true(
                page,
                &format!("!document.querySelector({shown:?})"),
                &format!("{route}: the close to unmount"),
            )
            .await
            .unwrap();
            assert!(
                opened + closed > 0,
                "{route}: nothing to finish, so this proves nothing"
            );
            fixture.close().await.unwrap();
        }
    });
}
