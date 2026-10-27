//! `Lightbox`: the overlay archetype around a `Carousel`. An arrow moves picture and focus,
//! no Tab lands in a hidden slide (57), and a second `open_with` jumps (323).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::wait::PageBounded;
use e2e::{
    Fixture, Suite, Viewport, passes::focus, passes::keyboard, passes::motion, passes::pointer,
    wait,
};

pub(super) const TRIGGER: &str = "#open-lightbox";
const TRIGGER_LAST: &str = "#open-lightbox-last";
pub(super) const DIALOG: &str = "[role=dialog]";
/// The picture that holds the tab stop: the one showing.
pub(super) const PICTURE: &str = "[role=dialog] [data-lightbox-frame] img[tabindex='0']";
/// The stage's carousel track, which takes no tab stop with `zoom` on (todo 2833).
const STAGE_TRACK: &str = "[role=dialog] [aria-roledescription=carousel] [role=group]";
/// The stage's track and the boxes round it, for the scroll spy.
const STAGE: &str = "[role=dialog] :has([data-lightbox-frame])";

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
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}

/// Todo 564: six thumbnails all fit, so their strip has no status and its
/// track is no tab stop; the thumbnails themselves still are.
#[test]
fn a_strip_whose_thumbnails_all_fit_is_quiet() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        let [status, stops, thumbnails]: [usize; 3] = page
            .evaluate_bounded(
                "(() => { const strip = [...document.querySelectorAll('[role=dialog] [aria-roledescription=carousel]')].at(-1); \
                 return [strip.querySelectorAll('[role=status]').length, \
                 strip.querySelectorAll('[tabindex=\"0\"]:not(button)').length, \
                 strip.querySelectorAll('button[tabindex=\"0\"]').length]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            (status, stops, thumbnails),
            (0, 0, 1),
            "status, track stops, thumbnail stops"
        );
        fixture
            .console
            .assert_clean("the quiet thumbnail strip")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2577: the focused picture is a replaced `img`, so the frame's sibling overlay draws the
/// inset ring over it, covering the frame.
#[test]
fn a_focused_picture_is_ringed_by_the_frame_overlay() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        keyboard::tab_to(page, PICTURE, 10).await.unwrap();
        let [shadow, gap]: [String; 2] = e2e::js(
            page,
            "(() => { const el = document.activeElement; const frame = el.parentElement; \
               const ring = frame.querySelector('[data-ring]'); const f = frame.getBoundingClientRect(); \
               const r = ring.getBoundingClientRect(); \
               return [getComputedStyle(ring).boxShadow, \
                 [r.left - f.left, r.top - f.top, f.right - r.right, f.bottom - r.bottom].map(Math.abs).join()]; })()",
        )
        .await;
        assert!(
            shadow.contains("inset"),
            "no ring over the picture: {shadow}"
        );
        assert!(
            gap.split(',')
                .all(|side| side.parse::<f64>().unwrap() < 0.5),
            "the overlay does not cover the frame: {gap}"
        );
        fixture.console.assert_clean("a focused picture").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Forced colours paint every fill `Canvas`: the current thumbnail's frame
/// must still set it apart from the others (1.4.1).
#[test]
fn the_current_thumbnail_shows_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        the_current_thumbnail_stands_out(&fixture.page)
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Opens the viewer in forced colours; the current thumbnail's fill is neither `Canvas`
/// nor the other thumbnails' (an unstyled one is transparent, not `Canvas`).
pub async fn the_current_thumbnail_stands_out(page: &Page) -> Result<()> {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    page.execute_bounded(
        SetEmulatedMediaParams::builder()
            .features(vec![MediaFeature::new("forced-colors", "active")])
            .build(),
    )
    .await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    let [current, other, canvas]: [String; 3] = page
        .evaluate_bounded(
            "(() => { const probe = document.createElement('div'); \
             probe.style.background = 'Canvas'; document.body.append(probe); \
             const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
             const thumbs = [...document.querySelectorAll('[role=dialog] button:has(img)')]; \
             const current = thumbs.find((t) => t.hasAttribute('aria-current')); \
             const other = thumbs.find((t) => !t.hasAttribute('aria-current')); \
             return [getComputedStyle(current).backgroundColor, \
             getComputedStyle(other).backgroundColor, canvas]; })()",
        )
        .await?
        .into_value()?;
    if current == canvas {
        bail!("the current thumbnail's frame is the page's own colour, {canvas}");
    }
    if current == other {
        bail!("the current thumbnail looks like the others, {other}");
    }
    Ok(())
}

/// The arrows move picture and focus; Tab never reaches an `inert` slide; Escape after
/// moving still returns focus to the trigger.
#[test]
fn arrows_move_the_picture_and_tab_skips_inert_slides() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();
            arrows_and_tab(&fixture.page)
                .await
                .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));
            fixture
                .console
                .assert_clean(&format!("the lightbox arrows at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Ctrl/Alt/Meta with an arrow, Home or End is the browser's, on the picture
/// and on a thumbnail alike (todo 503's family).
#[test]
fn shortcut_chords_pass_through() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        wait_showing(page, 0).await.unwrap();
        let keys = [
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_LEFT,
            keyboard::HOME,
            keyboard::END,
        ];
        let probe = "document.activeElement.outerHTML.slice(0, 80)";
        for stop in [PICTURE, "[role=dialog] button[aria-current=true]"] {
            keyboard::tab_to(page, stop, 12).await.unwrap();
            keyboard::assert_chords_ignored(page, &keys, probe)
                .await
                .unwrap_or_else(|e| panic!("on {stop}: {e}"));
        }
        fixture
            .console
            .assert_clean("chords in the lightbox")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Under reduced motion: a smooth scroll in a background page ends at the old offset and
/// puts the index back (2026-09-19). The gallery swap below needs smooth on.
async fn arrows_and_tab(page: &Page) -> Result<()> {
    motion::set_reduced_motion(page, true).await?;
    motion::assert_reduced_motion_matches(page).await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    assert_tab_skips_inert(page).await?;
    wait::for_js_true(
        page,
        &format!("document.querySelector('{STAGE_TRACK}').tabIndex < 0"),
        "the stage track to leave the tab order while the picture holds the stop",
    )
    .await?;

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

/// Waits until picture `index` is on the stage: inside the dialog and not `inert` (a slide
/// can be live before the scroll brings it in).
pub(super) async fn wait_showing(page: &Page, index: usize) -> Result<()> {
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
        .evaluate_bounded(format!(
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
        .evaluate_bounded(picture_focused_js(index))
        .await?
        .into_value()?;
    if !on {
        let actual = focus::active_element(page).await?;
        bail!("expected focus on picture {index}; the document holds {actual:?}");
    }
    Ok(())
}

/// Tabs round the dialog and fails on any stop in an `inert` subtree. Hidden pictures get
/// `tabindex="0"` first, so only `inert` keeps Tab out.
async fn assert_tab_skips_inert(page: &Page) -> Result<()> {
    let candidates: usize = page
        .evaluate_bounded(format!(
            r#"(() => {{
                const d = document.querySelector('{DIALOG}');
                return d ? d.querySelectorAll('a[href],button,input,select,textarea,[tabindex]').length : 0;
            }})()"#
        ))
        .await?
        .into_value()?;
    let inert: usize = page
        .evaluate_bounded(format!(
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
            .evaluate_bounded("!!document.activeElement?.closest('[inert]')")
            .await?
            .into_value()?;
        if in_inert {
            let actual = focus::active_element(page).await?;
            bail!("tab press {step} landed inside an inert slide, on {actual:?}");
        }
    }
    // Put the plant back: dioxus writes `tabindex` only when it changes, so
    // it would outlive the walk and hold a second tab stop once live.
    page.evaluate_bounded(
        "document.querySelectorAll('[data-planted]').forEach(img => { img.setAttribute('tabindex', '-1'); delete img.dataset.planted; })",
    )
    .await?;
    Ok(())
}

/// Todo 917: a double-click on the picture or round it selects nothing; the
/// caption, the one real text, still selects.
#[test]
fn a_double_click_selects_only_the_caption() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        double_click_selection(&fixture.page).await.unwrap();
        fixture.close().await.unwrap();
    });
}

async fn double_click_selection(page: &Page) -> Result<()> {
    const SELECTED: &str =
        "getSelection().isCollapsed ? '' : `${getSelection().rangeCount} ${getSelection()}`";
    pointer::click(page, TRIGGER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    // The toolbar row's centre is empty: the dialog's own surface.
    for target in [PICTURE, "[role=dialog] > div:first-child"] {
        pointer::double_click(page, target).await?;
        let selected: String = page.evaluate_bounded(SELECTED).await?.into_value()?;
        if !selected.is_empty() {
            bail!("a double-click on {target} selected {selected:?}");
        }
    }
    pointer::double_click(page, "[role=dialog] p").await?;
    let selected: String = page.evaluate_bounded(SELECTED).await?.into_value()?;
    if selected.is_empty() {
        bail!("a double-click on the caption selected nothing");
    }
    Ok(())
}

/// Todo 652: the shown picture is centred on both axes at every viewport; dialog, frame
/// and picture are each measured.
#[test]
fn the_current_picture_is_centred_in_the_viewport() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();
            centred(&fixture.page)
                .await
                .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// Centre offsets from the viewport's, `[x, y]`, of the dialog, the frame and
/// the picture as `object-fit` draws it.
const CENTRES_JS: &str = r#"(() => {
    const dialog = document.querySelector('[role=dialog]');
    const frame = dialog.querySelector('[data-lightbox-frame] img[tabindex="0"]').parentElement;
    const img = frame.querySelector('img');
    const vw = document.documentElement.clientWidth, vh = document.documentElement.clientHeight;
    const off = r => [r.left + r.width / 2 - vw / 2, r.top + r.height / 2 - vh / 2];
    const f = frame.getBoundingClientRect(), i = img.getBoundingClientRect();
    const r = Math.min(i.width / img.naturalWidth, i.height / img.naturalHeight, 1);
    const w = img.naturalWidth * r, h = img.naturalHeight * r;
    const drawn = { left: i.left + (i.width - w) / 2, top: i.top + (i.height - h) / 2, width: w, height: h };
    return [off(dialog.getBoundingClientRect()), off(f), off(drawn)].map(p => p.map(Math.round));
})()"#;

async fn centred(page: &Page) -> Result<()> {
    motion::set_reduced_motion(page, true).await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    wait::for_js_true(
        page,
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img')?.naturalWidth > 0",
        "the first picture to decode",
    )
    .await?;
    assert_centred(page, 0).await?;
    keyboard::tab_to(page, PICTURE, 12).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    wait_showing(page, 1).await?;
    assert_centred(page, 1).await?;
    keyboard::press(page, keyboard::END).await?;
    wait_showing(page, 5).await?;
    assert_centred(page, 5).await
}

async fn assert_centred(page: &Page, index: usize) -> Result<()> {
    let [dialog, frame, picture]: [[f64; 2]; 3] =
        page.evaluate_bounded(CENTRES_JS).await?.into_value()?;
    // The frame is below the close button, so only its width is centred.
    if dialog.iter().any(|d| d.abs() > 1.0) || frame[0].abs() > 1.0 || picture[0].abs() > 1.0 {
        bail!(
            "picture {index} off the viewport's centre by [x, y]: dialog {dialog:?}, frame {frame:?}, picture {picture:?}"
        );
    }
    Ok(())
}

/// Todo 323: swapping galleries from index 5 to 2 jumps, so nothing between is fetched.
/// Fetches are asserted at desktop only: at 390px lazy loading reaches one frame further.
#[test]
fn a_gallery_swap_jumps_and_fetches_only_around_the_new_index() {
    block_on(async {
        // One at a time: a fetch the other page made first may be served without an entry.
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();
            let swap = Swap {
                button: "swap-gallery",
                gallery: "second",
                index: 2,
                fetched: [1, 2, 3],
                never: Some(5),
            };
            gallery_swap(&fixture.page, viewport, swap)
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

/// Todo 4374: a swap from 5 to 4 moves the stage while the strip's clamped index stays put,
/// so the strip's own jump must not release the stage's lazy pictures early (4364).
#[test]
fn a_swap_the_strip_does_not_follow_still_jumps_the_stage() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/lightbox", viewport).await.unwrap();
            let swap = Swap {
                button: "swap-gallery-near",
                gallery: "near",
                index: 4,
                fetched: [3, 4, 5],
                never: None,
            };
            gallery_swap(&fixture.page, viewport, swap)
                .await
                .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));
            fixture
                .console
                .assert_clean(&format!("the near swap at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Waits for the stage's track to scroll smoothly again.
async fn wait_stage_smooth(page: &Page) -> Result<()> {
    // The open's jump keeps `seam` up until its scroll settles: 0-470ms after
    // the picture takes focus (measured).
    const SMOOTH: &str = "(() => { const track = [...document.querySelectorAll('[role=dialog] *')] \
        .filter(el => el.querySelector(':scope [data-lightbox-frame]') && el.scrollWidth > el.clientWidth) \
        .pop(); return !!track && getComputedStyle(track).scrollBehavior === 'smooth'; })()";
    wait::for_js_true(
        page,
        SMOOTH,
        "the stage's track to scroll smoothly after the open (`seam` to come down)",
    )
    .await
}

/// A function finding the stage's track: the innermost scroller round the frames.
const FIND_STAGE_TRACK: &str = "() => [...document.querySelectorAll('[role=dialog] *')] \
    .filter(el => el.querySelector(':scope [data-lightbox-frame]') && el.scrollWidth > el.clientWidth) \
    .pop()";

/// A second `open_with` while open: the button that sends it, the gallery and index it opens,
/// what the desktop fetches, and an index it must never fetch.
struct Swap {
    button: &'static str,
    gallery: &'static str,
    index: usize,
    fetched: [usize; 3],
    never: Option<usize>,
}

async fn gallery_swap(page: &Page, viewport: Viewport, swap: Swap) -> Result<()> {
    let Swap {
        button,
        gallery,
        index,
        fetched: expected,
        never,
    } = swap;
    // Smooth scrolling is the defect, so it has to be on. The suite's Chromium
    // scrolls instantly (todo 687): the spy below reads what was asked for.
    motion::set_reduced_motion(page, false).await?;
    let reduced: bool = page
        .evaluate_bounded("matchMedia('(prefers-reduced-motion: reduce)').matches")
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

    // A smooth swap is caught only on a track that animates at rest.
    wait_stage_smooth(page).await?;
    motion::spy_scrolls(page, STAGE, false).await?;
    // An observer, not `performance.getEntries`: its buffer holds 250 entries and drops the rest.
    page.evaluate_bounded(
        "window.__swapFetched = new Set(); new PerformanceObserver((list) => \
         list.getEntries().forEach((entry) => window.__swapFetched.add(entry.name))) \
         .observe({ type: 'resource' })",
    )
    .await?;
    // Where the track stood as each new `src` was set: Chromium defers a lazy fetch, so
    // an early `src` fetched nothing and only this shows it (todo 4363).
    page.evaluate_bounded(format!(
        "(() => {{ window.__srcAt = []; \
           const note = (img, value) => {{ if (String(value).endsWith('?{gallery}')) \
             window.__srcAt.push([img, ({FIND_STAGE_TRACK})()?.scrollLeft]); }}; \
           const setAttribute = Element.prototype.setAttribute; \
           Element.prototype.setAttribute = function (name, value) {{ \
             if (this instanceof HTMLImageElement && name === 'src') note(this, value); \
             return setAttribute.call(this, name, value); }}; \
           const src = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src'); \
           Object.defineProperty(HTMLImageElement.prototype, 'src', {{ ...src, \
             set(value) {{ note(this, value); src.set.call(this, value); }} }}); }})()"
    ))
    .await?;
    // The trigger is under the modal, so a script presses it.
    page.evaluate_bounded(format!("document.querySelector('#{button}').click()"))
        .await?;
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector('{DIALOG} [data-lightbox-frame=\"{index}\"] img')?.src.endsWith('?{gallery}')"
        ),
        "the swapped gallery to replace the first",
    )
    .await?;
    wait_showing(page, index).await?;
    wait_picture_focused(page, index).await?;
    // Past any smooth scroll (about 300ms from 5 to 2): the track holds still
    // between two polls, and every eager picture of the new gallery has loaded. At desktop
    // also its resource entry: `complete` came before it under load (fetched [], 1513).
    let timed = viewport == Viewport::Desktop;
    let eager = format!(
        "[...document.querySelectorAll('[role=dialog] img')] \
        .filter(img => img.src.endsWith('?{gallery}') && img.loading !== 'lazy')"
    );
    let settled = wait::for_js_true(
        page,
        &format!(
            "(() => {{ const track = [...document.querySelectorAll('[role=dialog] *')] \
                .filter(el => el.querySelector(':scope [data-lightbox-frame]') && el.scrollWidth > el.clientWidth) \
                .pop(); \
              const still = !!track && window.__swapLeft === track.scrollLeft; \
              window.__swapLeft = track?.scrollLeft; \
              const eager = {eager}; \
              return still && eager.length > 0 && eager.every(img => img.complete \
                  && (!{timed} || window.__swapFetched.has(img.src))); }})()"
        ),
        "the swapped track to settle and its eager pictures to load",
    )
    .await;
    if let Err(error) = settled {
        // Per eager picture: loading, complete, and whether its fetch was seen.
        let state: Vec<String> = page
            .evaluate_bounded(format!(
                "{eager}.map(img => `${{img.loading}}:${{img.complete}}:${{window.__swapFetched.has(img.src)}}`)"
            ))
            .await?
            .into_value()?;
        bail!("{error}; the eager pictures read {state:?}");
    }
    // The lazy pictures wait for their `src` only until the jump landed (todo 4308).
    wait::for_js_true(
        page,
        &format!(
            "[...document.querySelectorAll('[role=dialog] [data-lightbox-frame] img')] \
             .every(img => img.getAttribute('src')?.endsWith('?{gallery}'))"
        ),
        "every swapped picture to take its src once the track jumped",
    )
    .await?;
    let early: Vec<String> = page
        .evaluate_bounded(format!(
            "(() => {{ const left = ({FIND_STAGE_TRACK})().scrollLeft; \
               return window.__srcAt \
                 .filter(([img, at]) => img.loading === 'lazy' && img.closest('[data-lightbox-frame]') \
                   && !(Math.abs(at - left) <= 1)) \
                 .map(([img, at]) => `${{img.closest('[data-lightbox-frame]').dataset.lightboxFrame}} at ${{at}}, not ${{left}}`); }})()"
        ))
        .await?
        .into_value()?;
    if !early.is_empty() {
        bail!("lazy pictures took their src before the track jumped: {early:?}");
    }

    // None at all when the picture's focus scroll got the track there first.
    let scrolls = motion::scrolls(page).await?;
    if scrolls.iter().any(|scroll| scroll.smooth) {
        bail!("the swap scrolled the track with {scrolls:?}; it should jump");
    }

    let fetched: Vec<usize> = page
        .evaluate_bounded(format!(
            r#"(() => [...window.__swapFetched]
                .filter(n => n.endsWith('?{gallery}'))
                .map(n => Number(n.match(/\/(\d)[^/]*\.svg\?{gallery}$/)?.[1]) - 1)
                .sort())()"#
        ))
        .await?
        .into_value()?;
    if never.is_some_and(|never| fetched.contains(&never))
        || (viewport == Viewport::Desktop && fetched != expected)
    {
        bail!(
            "the swap to index {index} fetched the {gallery} gallery's pictures {fetched:?}; expected {expected:?}, and never {never:?}"
        );
    }

    Ok(())
}

/// `LightboxOptions::sx` reaches the portaled dialog, `parts` the caption and thumbnails.
#[test]
fn the_options_sx_and_parts_style_the_viewer() {
    block_on(async {
        let fixture = Fixture::open("/lightbox-parts", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, "[role=dialog] [data-slot=caption]").await?;
            let [spacing, caption, thumbnail]: [String; 3] = page
                .evaluate_bounded(
                    "(() => { const d = document.querySelector('[role=dialog]'); \
                     return [getComputedStyle(d).letterSpacing, \
                     getComputedStyle(d.querySelector('[data-slot=caption]')).fontStyle, \
                     getComputedStyle(d.querySelector('[data-slot=thumbnail]')).opacity]; })()",
                )
                .await?
                .into_value()?;
            anyhow::ensure!(spacing == "2px", "sx missed the dialog: {spacing}");
            anyhow::ensure!(caption == "italic", "parts missed the caption: {caption}");
            anyhow::ensure!(thumbnail == "0.5", "parts missed a thumbnail: {thumbnail}");
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("lightbox parts");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 1307: a long caption leaves the strip and the close button reachable.
#[test]
fn a_tall_lightbox_stays_reachable() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/lightbox-tall", viewport).await.unwrap();
            let page = &fixture.page;
            pointer::click(page, TRIGGER).await.unwrap();
            wait::for_visible(page, DIALOG).await.unwrap();
            let name = viewport.name();
            crate::modal::assert_reachable(
                page,
                "[...document.querySelectorAll('[role=dialog] button')].at(-1)",
                &format!("the last thumbnail at {name}"),
            )
            .await;
            crate::modal::assert_reachable(
                page,
                "document.querySelector('[role=dialog] button')",
                &format!("the first button at {name}"),
            )
            .await;
            fixture
                .console
                .assert_clean(&format!("/lightbox-tall at {name}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}
