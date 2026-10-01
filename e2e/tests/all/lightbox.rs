//! `Lightbox`: the overlay archetype around a `Carousel`. An arrow moves picture and focus,
//! no Tab lands in a hidden slide (57), and a second `open_with` jumps (323).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::suite::Step;
use e2e::{
    Fixture, Suite, Viewport, passes::focus, passes::keyboard, passes::motion, passes::pointer,
    wait,
};

const TRIGGER: &str = "#open-lightbox";
const TRIGGER_LAST: &str = "#open-lightbox-last";
const DIALOG: &str = "[role=dialog]";
/// The picture that holds the tab stop: the one showing.
const PICTURE: &str = "[role=dialog] [data-lightbox-frame] img[tabindex='0']";
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
            .evaluate(
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
    page.execute(
        SetEmulatedMediaParams::builder()
            .features(vec![MediaFeature::new("forced-colors", "active")])
            .build(),
    )
    .await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    let [current, other, canvas]: [String; 3] = page
        .evaluate(
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

/// Tabs round the dialog and fails on any stop in an `inert` subtree. Hidden pictures get
/// `tabindex="0"` first, so only `inert` keeps Tab out.
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

/// `+` and `-` step like the wheel, so the keyboard reaches `max_zoom` (the
/// theme's 8x) and back in fine steps (WCAG 2.1.1).
#[test]
fn plus_and_minus_zoom_to_max_zoom_and_back() {
    const PLUS: keyboard::Key = keyboard::Key {
        key: "+",
        code: "Equal",
        vk: 187,
        text: Some("+"),
    };
    const MINUS: keyboard::Key = keyboard::Key {
        key: "-",
        code: "Minus",
        vk: 189,
        text: Some("-"),
    };
    const STYLE: &str =
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText";
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        motion::set_reduced_motion(page, true).await.unwrap();
        keyboard::tab_to(page, TRIGGER, 5).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_visible(page, DIALOG).await.unwrap();
        wait_showing(page, 0).await.unwrap();
        keyboard::tab_to(page, PICTURE, 12).await.unwrap();
        // 1.25^10 passes 8, so the tenth press clamps.
        for _ in 0..10 {
            keyboard::press(page, PLUS).await.unwrap();
        }
        if wait::for_js_true(
            page,
            &format!("{STYLE}.includes('scale(8)')"),
            "the picture at 8x",
        )
        .await
        .is_err()
        {
            let style: String = page.evaluate(STYLE).await.unwrap().into_value().unwrap();
            panic!("ten presses of + left the picture at {style:?}, not the theme's max_zoom 8x");
        }
        for _ in 0..10 {
            keyboard::press(page, MINUS).await.unwrap();
        }
        wait::for_js_true(
            page,
            &format!("!{STYLE}.includes('scale(')"),
            "the picture fitted again",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("the lightbox keyboard zoom")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 864: the zoom buttons step like `+`/`-` and turn `aria-disabled` at a limit, keeping
/// focus; a double-click steps 2x, 4x, 8x, then back to fit.
#[test]
fn the_zoom_buttons_and_double_click_step_the_zoom() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        zoom_buttons(&fixture.page).await.unwrap();
        fixture
            .console
            .assert_clean("the lightbox zoom buttons")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn zoom_buttons(page: &Page) -> Result<()> {
    const ZOOM_IN: &str = "[role=dialog] button[aria-label='Zoom in']";
    const ZOOM_OUT: &str = "[role=dialog] button[aria-label='Zoom out']";
    const STYLE: &str =
        "document.querySelector('[role=dialog] [data-lightbox-frame=\"0\"] img').style.cssText";
    // Enabled drops the attribute rather than writing `false`.
    let disabled = |selector: &str, value: bool| {
        let expected = if value { "'true'" } else { "null" };
        format!(
            "document.querySelector({selector:?})?.getAttribute('aria-disabled') === {expected}"
        )
    };
    let wait_style = async |check: String, what: &str| -> Result<()> {
        if wait::for_js_true(page, &check, what).await.is_err() {
            let style: String = page.evaluate(STYLE).await?.into_value()?;
            bail!("waiting for {what}, the picture's style is {style:?}");
        }
        Ok(())
    };
    motion::set_reduced_motion(page, true).await?;
    keyboard::tab_to(page, TRIGGER, 5).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait_showing(page, 0).await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_OUT, true),
        "zoom out disabled when fitted",
    )
    .await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_IN, false),
        "zoom in enabled when fitted",
    )
    .await?;
    // Disabled looks it, and a hover does not light it up.
    let look = format!(
        "(() => {{ const s = getComputedStyle(document.querySelector({ZOOM_OUT:?})); return [s.opacity, s.backgroundColor, s.color].join(' '); }})()"
    );
    let rest: String = page.evaluate(look.as_str()).await?.into_value()?;
    pointer::hover(page, ZOOM_OUT).await?;
    let hovered: String = page.evaluate(look.as_str()).await?.into_value()?;
    if !rest.starts_with("0.5 ") || hovered != rest {
        bail!("disabled zoom out: {rest:?} at rest, {hovered:?} hovered");
    }

    pointer::click(page, ZOOM_IN).await?;
    wait_style(
        format!("{STYLE}.includes('scale(1.25)')"),
        "one wheel step in",
    )
    .await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_OUT, false),
        "zoom out enabled once zoomed",
    )
    .await?;

    // 1.25^10 passes 8: the button clamps, disables, and keeps the focus.
    keyboard::tab_to(page, ZOOM_IN, 12).await?;
    for _ in 0..10 {
        keyboard::press(page, keyboard::ENTER).await?;
    }
    wait_style(format!("{STYLE}.includes('scale(8)')"), "the picture at 8x").await?;
    wait::for_js_true(page, &disabled(ZOOM_IN, true), "zoom in disabled at 8x").await?;
    focus::assert_focused(page, ZOOM_IN, "zoom in at its limit").await?;

    keyboard::tab_to(page, ZOOM_OUT, 12).await?;
    for _ in 0..12 {
        keyboard::press(page, keyboard::ENTER).await?;
    }
    wait_style(format!("!{STYLE}.includes('scale(')"), "the picture fitted").await?;
    wait::for_js_true(
        page,
        &disabled(ZOOM_OUT, true),
        "zoom out disabled when fitted again",
    )
    .await?;
    focus::assert_focused(page, ZOOM_OUT, "zoom out at its limit").await?;

    let picture = "[role=dialog] [data-lightbox-frame=\"0\"] img";
    for scale in ["scale(2)", "scale(4)", "scale(8)"] {
        pointer::double_click(page, picture).await?;
        wait_style(
            format!("{STYLE}.includes('{scale}')"),
            &format!("a double-click to {scale}"),
        )
        .await?;
    }
    pointer::double_click(page, picture).await?;
    wait_style(
        format!("!{STYLE}.includes('scale(')"),
        "a double-click back to fit",
    )
    .await
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
        let selected: String = page.evaluate(SELECTED).await?.into_value()?;
        if !selected.is_empty() {
            bail!("a double-click on {target} selected {selected:?}");
        }
    }
    pointer::double_click(page, "[role=dialog] p").await?;
    let selected: String = page.evaluate(SELECTED).await?.into_value()?;
    if selected.is_empty() {
        bail!("a double-click on the caption selected nothing");
    }
    Ok(())
}

/// Todo 421: zoom, pan to the edge, then shrink to a phone: the pan clamps to the new bounds.
#[test]
fn a_resize_keeps_a_zoomed_picture_in_its_frame() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        zoomed_resize(&fixture.page).await.unwrap();
        fixture.console.assert_clean("the lightbox resize").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The shown picture's centre offset from its frame's, and its bound, per axis:
/// `[dx, dy, x_bound, y_bound]`.
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

/// Todo 565: a click on a zoomed picture centres the spot clicked, while a
/// drag stays a pan; the zoom is announced and the keys are described.
#[test]
fn a_click_pans_a_zoomed_picture_and_the_zoom_is_announced() {
    block_on(async {
        let fixture = Fixture::open("/lightbox", Viewport::Desktop).await.unwrap();
        click_pan(&fixture.page).await.unwrap();
        fixture
            .console
            .assert_clean("the lightbox click pan")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn click_pan(page: &Page) -> Result<()> {
    const Z: keyboard::Key = keyboard::Key {
        key: "z",
        code: "KeyZ",
        vk: 90,
        text: Some("z"),
    };
    const STATUS: &str = "[...document.querySelectorAll('[role=dialog] [role=status]')].map(s => s.textContent).join('|')";
    motion::set_reduced_motion(page, true).await?;
    // As in `zoomed_resize`: the picture at 2x overhangs the stage vertically.
    page.execute(SetDeviceMetricsOverrideParams::new(900, 500, 1.0, false))
        .await?;
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
    keyboard::tab_to(page, PICTURE, 12).await?;
    let described: bool = page
        .evaluate(format!(
            "(() => {{ const ids = document.querySelector({PICTURE:?}).getAttribute('aria-describedby') || ''; \
             return ids.split(' ').some(id => document.getElementById(id)?.textContent.startsWith('Z, plus or minus to zoom')); }})()"
        ))
        .await?
        .into_value()?;
    if !described {
        bail!("the picture's aria-describedby does not reach the keys hint");
    }
    keyboard::press(page, Z).await?;
    wait::for_js_true(
        page,
        &format!("{STATUS}.includes('Zoomed to 200%')"),
        "the zoom to be announced",
    )
    .await?;

    let frame = pointer::centre_of(page, "[role=dialog] [data-lightbox-frame=\"0\"]").await?;
    let below = pointer::Point {
        x: frame.x,
        y: frame.y + 60.0,
    };
    pointer::drag(page, below, below, 1).await?;
    let dy_is = |target: f64| {
        format!("(() => {{ const [, dy] = {PAN_JS}; return Math.abs(dy - ({target})) <= 1; }})()")
    };
    if wait::for_js_true(page, &dy_is(-60.0), "a click to centre the spot clicked")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        bail!("a click 60px below the centre left the picture at {seen:?}, not dy -60");
    }
    // A drag's release is no click: the pan stays where the drag put it.
    let up = pointer::Point {
        x: frame.x,
        y: frame.y + 30.0,
    };
    pointer::drag(page, frame, up, 5).await?;
    if wait::for_js_true(page, &dy_is(-30.0), "a drag to pan by its own distance")
        .await
        .is_err()
    {
        let seen: [f64; 4] = page.evaluate(PAN_JS).await?.into_value()?;
        bail!("a 30px drag down from dy -60 left the picture at {seen:?}, not dy -30");
    }

    // Todo 864: `z` steps on to 4x and 8x, then back to fit.
    for said in ["Zoomed to 400%", "Zoomed to 800%", "Zoom reset"] {
        keyboard::press(page, Z).await?;
        wait::for_js_true(
            page,
            &format!("{STATUS}.includes('{said}')"),
            &format!("z to announce {said:?}"),
        )
        .await?;
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
    let [dialog, frame, picture]: [[f64; 2]; 3] = page.evaluate(CENTRES_JS).await?.into_value()?;
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

/// Waits for the stage's track to scroll smoothly again, for 2s at most.
async fn wait_stage_smooth(page: &Page) -> Result<()> {
    // The open's jump keeps `seam` up until its scroll settles: 0-470ms after
    // the picture takes focus (measured).
    const SMOOTH: &str = "(() => { const track = [...document.querySelectorAll('[role=dialog] *')] \
        .filter(el => el.querySelector(':scope [data-lightbox-frame]') && el.scrollWidth > el.clientWidth) \
        .pop(); return !!track && getComputedStyle(track).scrollBehavior === 'smooth'; })()";
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while !page.evaluate(SMOOTH).await?.into_value::<bool>()? {
        if std::time::Instant::now() > deadline {
            bail!("the stage's track still jumps 2s after the open: `seam` never came down");
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    Ok(())
}

async fn gallery_swap(page: &Page, viewport: Viewport) -> Result<()> {
    // Smooth scrolling is the defect, so it has to be on. The suite's Chromium
    // scrolls instantly (todo 687): the spy below reads what was asked for.
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

    // A smooth swap is caught only on a track that animates at rest.
    wait_stage_smooth(page).await?;
    motion::spy_scrolls(page, STAGE, false).await?;
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
    // Past any smooth scroll (about 300ms from 5 to 2): the track holds still
    // between two polls, and every eager picture of the new gallery has loaded. At desktop
    // also its resource entry: `complete` came before it under load (fetched [], 1513).
    let timed = viewport == Viewport::Desktop;
    wait::for_js_true(
        page,
        &format!(
            "(() => {{ const track = [...document.querySelectorAll('[role=dialog] *')] \
                .filter(el => el.querySelector(':scope [data-lightbox-frame]') && el.scrollWidth > el.clientWidth) \
                .pop(); \
              const still = !!track && window.__swapLeft === track.scrollLeft; \
              window.__swapLeft = track?.scrollLeft; \
              const eager = [...document.querySelectorAll('[role=dialog] img')] \
                .filter(img => img.src.endsWith('?second') && img.loading !== 'lazy'); \
              return still && eager.length > 0 && eager.every(img => img.complete \
                  && (!{timed} || performance.getEntriesByName(img.src).length > 0)); }})()"
        ),
        "the swapped track to settle and its eager pictures to load",
    )
    .await?;

    // None at all when the picture's focus scroll got the track there first.
    let scrolls = motion::scrolls(page).await?;
    if scrolls.iter().any(|scroll| scroll.smooth) {
        bail!("the swap scrolled the track with {scrolls:?}; it should jump");
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

const ZOOM_IN: &str = "[role=dialog] button[aria-label='Zoom in']";

/// The current picture's scale, read off its inline transform; 1 when fitted.
async fn scale<D: Driver>(d: &mut D) -> Result<f64> {
    let style = d.attr(PICTURE, "style").await?.unwrap_or_default();
    let Some((_, after)) = style.split_once("scale(") else {
        return Ok(1.0);
    };
    let number = after.split(')').next().unwrap_or_default();
    Ok(number.parse()?)
}

/// Opens the gallery and waits for its picture to take presses.
async fn open<D: Driver>(d: &mut D) -> Result<()> {
    d.click(TRIGGER).await?;
    eventually(d, "the lightbox's picture", async |d| {
        d.exists(PICTURE).await
    })
    .await?;
    // The stage's first scroll settles before a gesture lands on it: the
    // picture is centred and holds still from one idle to the next.
    let (width, _) = d.viewport().await?;
    eventually(d, "the stage's first scroll to settle", async |d| {
        let before = d.rect(PICTURE).await?;
        d.idle().await;
        let after = d.rect(PICTURE).await?;
        let centre = after.x + after.width / 2.0;
        Ok(after.x == before.x && (centre - width / 2.0).abs() < 1.0)
    })
    .await
}

/// Todo 1068: a tap on "Zoom in" zooms (a WebView answered the frame query
/// with nothing).
async fn zoom_in_tapped<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    // Todo 1069: the toolbar is `sm`, `Button`'s 30px.
    let height = d.rect(ZOOM_IN).await?.height;
    if (height - 30.0).abs() > 0.5 {
        bail!("{:?}: zoom in is {height}px tall, not 30px", d.platform());
    }
    d.click(ZOOM_IN).await?;
    eventually(d, "one zoom step in", async |d| Ok(scale(d).await? == 1.25)).await
}

e2e::scenario!(
    a_tap_on_zoom_in_zooms_the_picture,
    "/lightbox",
    zoom_in_tapped
);

/// Todo 1067: two fingers spread zoom the picture and do not close the viewer;
/// pinched back together it fits again.
async fn pinch_zooms<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open(d).await?;
    // 2.5x: no power of the wheel's 1.25, so no page pinch turned into wheel steps.
    d.pinch(PICTURE, 80.0, 200.0).await?;
    eventually(d, "the pinch to 2.5x", async |d| {
        Ok((scale(d).await? - 2.5).abs() < 0.01)
    })
    .await?;
    ensure_open(d).await?;
    d.pinch(PICTURE, 200.0, 40.0).await?;
    eventually(d, "the pinch back to fit", async |d| {
        Ok(scale(d).await? == 1.0)
    })
    .await?;
    ensure_open(d).await
}

async fn ensure_open<D: Driver>(d: &mut D) -> Result<()> {
    match d.exists(DIALOG).await? {
        true => Ok(()),
        false => bail!("{:?}: the pinch closed the lightbox", d.platform()),
    }
}

e2e::scenario!(
    a_pinch_zooms_the_picture,
    "/lightbox",
    pinch_zooms,
    native: skip("Blitz takes no multi-touch input"),
    desktop: skip("1126: no touch input under Xvfb")
);

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
                .evaluate(
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
