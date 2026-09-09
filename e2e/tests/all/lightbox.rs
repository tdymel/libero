//! `Lightbox`: the overlay archetype, around a `Carousel` of pictures.
//!
//! Beyond the archetype, the contract here is the one `Carousel`'s `inert`
//! slides and `Lightbox`'s effect-driven focus make together: the picture is
//! the tab stop, an arrow moves both the picture and focus, and no Tab ever
//! lands in a slide that is scrolled away (todo 57). And todo 323: a second
//! `open_with` while the viewer is open jumps to the new gallery's index,
//! rather than scrolling past - and so fetching - the pictures between.
//!
//! `contrast_covers` guards todo 327: the scroll lock made axe judge both
//! arrows and every thumbnail off-screen, so nothing below the close button was
//! contrast-checked in the open state.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::focus, passes::keyboard, passes::motion, wait};

const TRIGGER: &str = "#open-lightbox";
const TRIGGER_LAST: &str = "#open-lightbox-last";
const DIALOG: &str = "[role=dialog]";
/// The picture that holds the tab stop: the one showing.
const PICTURE: &str = "[role=dialog] [data-lightbox-frame] img[tabindex='0']";

#[test]
fn it_meets_the_baseline() {
    Suite::new("lightbox", "/lightbox")
        .focusable(TRIGGER)
        .contrast_covers(DIALOG)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: DIALOG,
                traps_focus: true,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the lightbox contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The arrows on the picture move to the next picture and take focus with
/// it; Tab from anywhere in the dialog never reaches a slide that is `inert`;
/// and Escape after moving still returns focus to the trigger.
#[test]
fn arrows_move_the_picture_and_tab_skips_inert_slides() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();
            arrows_and_tab(&fixture.page)
                .await
                .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));
            fixture
                .console
                .assert_clean(&format!("the lightbox arrows at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Under reduced motion, so every slide change is an instant scroll. A smooth
/// one is unreliable in the suite's background pages: at 390px a smooth
/// `scrollTo` from picture 6 to 5 moved nothing and ended in a `scrollend` at
/// the old offset, which put the index back. The same run passes in a
/// foreground page (measured 2026-09-19). The gallery swap below is the test
/// that needs smooth scrolling on.
async fn arrows_and_tab(page: &Page) -> Result<()> {
    motion::set_reduced_motion(page, true).await?;
    motion::assert_reduced_motion_matches(page).await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    assert_tab_skips_inert(page).await?;

    keyboard::tab_to(page, PICTURE, 12).await?;
    assert_picture_focused(page, 0).await?;

    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait_showing(page, 1).await?;
    wait_picture_focused(page, 1).await?;

    keyboard::press(page, keyboard::END).await?;
    wait_showing(page, 5).await?;
    wait_picture_focused(page, 5).await?;
    assert_tab_skips_inert(page).await?;

    keyboard::tab_to(page, PICTURE, 12).await?;
    keyboard::press(page, keyboard::ARROW_LEFT).await?;
    wait_showing(page, 4).await?;
    wait_picture_focused(page, 4).await?;

    keyboard::press(page, keyboard::ESCAPE).await?;
    wait::for_js_true(
        page,
        &format!(
            "document.activeElement === document.querySelector({})",
            serde_json::to_string(TRIGGER)?
        ),
        "focus to return to the trigger after moving through the gallery",
    )
    .await?;
    focus::assert_focused(page, TRIGGER, "Escape after the arrows").await
}

/// Wait until picture `index` is the one on the stage: its frame lies inside
/// the dialog, and nothing above it is `inert`. Both, since a slide can be
/// live before the scroll has brought it in.
async fn wait_showing(page: &Page, index: usize) -> Result<()> {
    let showing = format!(
        r#"(() => {{
            const frame = document.querySelector('{DIALOG} [data-lightbox-frame="{index}"]');
            const dialog = document.querySelector('{DIALOG}');
            if (!frame || !dialog || frame.closest('[inert]')) return false;
            const f = frame.getBoundingClientRect(), d = dialog.getBoundingClientRect();
            return f.left >= d.left - 1 && f.right <= d.right + 1;
        }})()"#
    );
    if wait::for_js_true(page, &showing, "the picture")
        .await
        .is_ok()
    {
        return Ok(());
    }
    let seen: String = page
        .evaluate(format!(
            r#"(() => {{
                const d = document.querySelector('{DIALOG}')?.getBoundingClientRect();
                const frames = [...document.querySelectorAll('{DIALOG} [data-lightbox-frame]')].map(f => {{
                    const r = f.getBoundingClientRect();
                    return `${{f.dataset.lightboxFrame}}:${{Math.round(r.left)}}-${{Math.round(r.right)}}${{f.closest('[inert]') ? ' inert' : ''}}`;
                }});
                const cur = document.querySelector('{DIALOG} button[aria-current]')?.getAttribute('aria-label');
                const cap = document.querySelector('{DIALOG} p')?.textContent;
                const a = document.activeElement;
                const tracks = [...document.querySelectorAll('{DIALOG} [role=group]')].map(t => `${{t.scrollLeft}}/${{t.scrollWidth - t.clientWidth}} ${{getComputedStyle(t).scrollBehavior}}`);
                return `dialog ${{Math.round(d?.left)}}-${{Math.round(d?.right)}}; ${{frames.join(', ')}}; thumb ${{cur}}; caption ${{cap}}; active ${{a?.tagName}}#${{a?.id}}; tracks ${{tracks.join(' | ')}}`;
            }})()"#
        ))
        .await?
        .into_value()?;
    bail!("picture {index} never came to rest on the stage: {seen}")
}

fn picture_focused_js(index: usize) -> String {
    format!(
        r#"(() => {{
            const a = document.activeElement;
            return !!a && a.tagName === 'IMG'
                && a.closest('[data-lightbox-frame]')?.getAttribute('data-lightbox-frame') === '{index}';
        }})()"#
    )
}

async fn wait_picture_focused(page: &Page, index: usize) -> Result<()> {
    if wait::for_js_true(
        page,
        &picture_focused_js(index),
        &format!("focus on picture {index}"),
    )
    .await
    .is_err()
    {
        let actual = focus::active_element(page).await?;
        bail!("moving to picture {index} did not carry focus to it; the document holds {actual:?}");
    }
    Ok(())
}

async fn assert_picture_focused(page: &Page, index: usize) -> Result<()> {
    let on: bool = page
        .evaluate(picture_focused_js(index))
        .await?
        .into_value()?;
    if !on {
        let actual = focus::active_element(page).await?;
        bail!("expected focus on picture {index}; the document holds {actual:?}");
    }
    Ok(())
}

/// Tab round the whole dialog, more than once, and fail on any stop inside
/// an `inert` subtree. The budget is every candidate in the dialog, inert or
/// not, plus two, so the walk wraps whatever the live count is.
///
/// A picture that is not showing already carries `tabindex="-1"`, so on its
/// own the walk could not fail. Every picture in an inert slide is first
/// given `tabindex="0"`: only `inert` is then left to keep Tab out.
async fn assert_tab_skips_inert(page: &Page) -> Result<()> {
    let candidates: usize = page
        .evaluate(format!(
            r#"(() => {{
                const d = document.querySelector('{DIALOG}');
                return d ? d.querySelectorAll('a[href],button,input,select,textarea,[tabindex]').length : 0;
            }})()"#
        ))
        .await?
        .into_value()?;
    let inert: usize = page
        .evaluate(format!(
            r#"(() => {{
                const pictures = document.querySelectorAll('{DIALOG} [inert] [data-lightbox-frame] img');
                pictures.forEach(img => {{ img.setAttribute('tabindex', '0'); img.dataset.planted = ''; }});
                return pictures.length;
            }})()"#
        ))
        .await?
        .into_value()?;
    // A check that cannot fail: with nothing inert there is nothing to skip.
    if inert == 0 {
        bail!("no picture in the dialog is inert, so the Tab walk would prove nothing");
    }
    for step in 1..=candidates + 2 {
        keyboard::press(page, keyboard::TAB).await?;
        let in_inert: bool = page
            .evaluate("!!document.activeElement?.closest('[inert]')")
            .await?
            .into_value()?;
        if in_inert {
            let actual = focus::active_element(page).await?;
            bail!("tab press {step} landed inside an inert slide, on {actual:?}");
        }
    }
    // Put the plant back: dioxus writes `tabindex` only when it changes, so
    // it would outlive the walk and hold a second tab stop once live.
    page.evaluate(
        "document.querySelectorAll('[data-planted]').forEach(img => { img.setAttribute('tabindex', '-1'); delete img.dataset.planted; })",
    )
    .await?;
    Ok(())
}

/// Todo 421: zoom, pan to the edge, then turn the window into a phone. The pan
/// has to be clamped to the new, smaller bounds, or the picture leaves its
/// frame.
#[test]
fn a_resize_keeps_a_zoomed_picture_in_its_frame() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        zoomed_resize(&fixture.page).await.unwrap();
        fixture.console.assert_clean("the lightbox resize").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The shown picture's centre offset from its frame's and how far it may be,
/// per axis: `[dx, dy, x_bound, y_bound]`. The `<img>` box is the frame's, so
/// its width over the frame's is the scale.
const PAN_JS: &str = r#"(() => {
    const frame = document.querySelector('[role=dialog] [data-lightbox-frame="0"]');
    const img = frame.querySelector('img');
    const f = frame.getBoundingClientRect(), i = img.getBoundingClientRect();
    const s = i.width / f.width;
    const r = Math.min(f.width / img.naturalWidth, f.height / img.naturalHeight, 1);
    const bound = (natural, side) => Math.max(0, (natural * r * s - side) / 2);
    return [
        i.left + i.width / 2 - (f.left + f.width / 2),
        i.top + i.height / 2 - (f.top + f.height / 2),
        bound(img.naturalWidth, f.width),
        bound(img.naturalHeight, f.height),
    ];
})()"#;

async fn zoomed_resize(page: &Page) -> Result<()> {
    const Z: keyboard::Key = keyboard::Key {
        key: "z",
        code: "KeyZ",
        vk: 90,
        text: Some("z"),
    };
    // No transform transition, so every rect read is the resting one.
    motion::set_reduced_motion(page, true).await?;
    // Short, so the 400x260 picture at 2x overhangs the 70vh stage (350px)
    // and has somewhere to pan; still over the phone layout's 30rem.
    page.execute(SetDeviceMetricsOverrideParams::new(900, 500, 1.0, false))
        .await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    // Decoded first: without a natural size the bounds are the whole frame.
    wait::for_js_true(
        page,
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img')?.naturalWidth > 0",
        "the first picture to decode",
    )
    .await?;
    keyboard::tab_to(page, PICTURE, 12).await?;
    keyboard::press(page, Z).await?;
    wait::for_js_true(
        page,
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText.includes('scale(2)')",
        "the picture to zoom",
    )
    .await?;
    // At picture 0 an arrow past the edge changes nothing.
    for _ in 0..40 {
        keyboard::press(page, keyboard::ARROW_LEFT).await?;
        keyboard::press(page, keyboard::ARROW_UP).await?;
    }
    // Resting on its edge on both axes, and moved on at least one.
    let at_edge = format!(
        "(() => {{ const [dx, dy, bx, by] = {PAN_JS}; return Math.abs(dx) >= bx - 1 && Math.abs(dy) >= by - 1 && (bx >= 1 || by >= 1); }})()"
    );
    if wait::for_js_true(page, &at_edge, "the pan to reach the picture's edge")
        .await
        .is_err()
    {
        let style: String = page
            .evaluate("document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText")
            .await?
            .into_value()?;
        let actual = focus::active_element(page).await?;
        bail!("the pan never reached the picture's edge: style {style:?}, focus {actual:?}");
    }
    let [dx, dy, ..]: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;

    let (width, height) = Viewport::Mobile.size();
    page.execute(SetDeviceMetricsOverrideParams::new(
        width, height, 1.0, true,
    ))
    .await?;
    let settled = format!(
        "(() => {{ const [dx, dy, bx, by] = {PAN_JS}; return Math.abs(dx) <= bx + 1 && Math.abs(dy) <= by + 1; }})()"
    );
    if wait::for_js_true(page, &settled, "the pan clamped to the resized frame")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        bail!(
            "after the resize the picture sits at {seen:?} ([dx, dy, x_bound, y_bound]); it left its frame"
        );
    }
    // The clamp had to act: the old pan lies outside the new bounds.
    let [.., bx, by]: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
    if dx.abs() <= bx + 1.0 && dy.abs() <= by + 1.0 {
        bail!(
            "the old pan ({dx}, {dy}) still fits the new bounds ({bx}, {by}); the resize tested nothing"
        );
    }
    Ok(())
}

/// Todo 323. Open on the last picture (index 5), then swap in a second
/// gallery at index 2 while the viewer is open. The strip has to jump: every
/// scroll the stage reports after the swap is already at the new offset. A
/// smooth scroll back from 5 passes 4 and 3, and lazy loading fetches what
/// it passes.
///
/// What was fetched is asserted at desktop only. There a frame is 1136px
/// wide, and only the new picture and its `preload: 1` neighbours, indices
/// 1-3, may load. At 390px Chromium's lazy loading also fetches a picture a
/// frame beyond the eager ones: a fresh open at index 0 fetches 0-2 there
/// (measured). At both widths the old current picture, 5, must not load: it
/// is on screen in the render that swaps, until the jump.
#[test]
fn a_gallery_swap_jumps_and_fetches_only_around_the_new_index() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();
            gallery_swap(&fixture.page, viewport)
                .await
                .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));
            fixture
                .console
                .assert_clean(&format!("the lightbox swap at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

async fn gallery_swap(page: &Page, viewport: Viewport) -> Result<()> {
    // Smooth scrolling is the defect, so it has to be on.
    motion::set_reduced_motion(page, false).await?;
    let reduced: bool = page
        .evaluate("matchMedia('(prefers-reduced-motion: reduce)').matches")
        .await?
        .into_value()?;
    if reduced {
        bail!("the page still reduces motion; the emulation did nothing");
    }

    keyboard::tab_to(page, TRIGGER_LAST, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 5).await?;
    keyboard::tab_to(page, PICTURE, 12).await?;
    assert_picture_focused(page, 5).await?;

    // Every offset the stage's track reports from here on.
    page.evaluate(
        r#"(() => {
            window.__stageScrolls = [];
            addEventListener('scroll', e => {
                if (e.target.querySelector?.(':scope [data-lightbox-frame]'))
                    window.__stageScrolls.push(Math.round(e.target.scrollLeft));
            }, true);
        })()"#,
    )
    .await?;
    // The trigger is under the modal, so a script presses it.
    page.evaluate("document.querySelector('#swap-gallery').click()")
        .await?;
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector('{DIALOG} [data-lightbox-frame=\"2\"] img')?.src.endsWith('?second')"
        ),
        "the second gallery to replace the first",
    )
    .await?;
    wait_showing(page, 2).await?;
    wait_picture_focused(page, 2).await?;
    // Past any smooth scroll (about 300ms from 5 to 2) and the fetches it
    // would start.
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    let (scrolls, rest): (Vec<i64>, i64) = page
        .evaluate(
            r#"(() => {
                // The innermost overflowing box round the frames.
                const track = [...document.querySelectorAll('[role=dialog] *')]
                    .filter(el => el.querySelector(':scope [data-lightbox-frame]') && el.scrollWidth > el.clientWidth)
                    .pop();
                return [window.__stageScrolls, Math.round(track.scrollLeft)];
            })()"#,
        )
        .await?
        .into_value()?;
    if scrolls.iter().any(|&offset| offset != rest) {
        bail!("the swap scrolled through {scrolls:?} to rest at {rest}; it should jump");
    }

    let fetched: Vec<usize> = page
        .evaluate(
            r#"(() => performance.getEntriesByType('resource')
                .map(e => e.name)
                .filter(n => n.endsWith('?second'))
                .map(n => Number(n.match(/\/(\d)[^/]*\.svg\?second$/)?.[1]) - 1)
                .sort())()"#,
        )
        .await?
        .into_value()?;
    if fetched.contains(&5) || (viewport == Viewport::Desktop && fetched != [1, 2, 3]) {
        bail!(
            "the swap to index 2 fetched the second gallery's pictures {fetched:?}; expected [1, 2, 3], and never 5"
        );
    }

    Ok(())
}
