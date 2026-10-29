//! The picker fields at 360px: the phone sheet (todo 3900), a finger's keyboardless tap and the
//! picker's own text box (3905, 3923), the clock face (3926), 44px targets (3901), short ranges (3904),
//! the scrolling range (3903) and the swiped month (3924).

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::pointer::tap;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, frames, wait};

const SHEET: &str = "[role=dialog]";
const TYPED: &str = "[role=dialog] [data-slot=text-input]";

async fn phone() -> Fixture {
    let fixture = Fixture::open("/picker-phone", Viewport::Mobile)
        .await
        .unwrap();
    fixture
        .page
        .execute_bounded(SetDeviceMetricsOverrideParams::new(360, 780, 1.0, true))
        .await
        .unwrap();
    fixture
}

fn input(field: &str) -> String {
    format!("#{field} input[data-controlled]")
}

/// The sheet is placed, edge to edge on the screen's foot.
async fn sheet_shown(page: &Page) -> Result<()> {
    let js = format!(
        "(() => {{ const s = document.querySelector({SHEET:?}); \
         if (!s || getComputedStyle(s).visibility !== 'visible') return false; \
         const r = s.getBoundingClientRect(); \
         return Math.abs(r.left) < 1 && Math.abs(r.width - innerWidth) < 1 \
           && Math.abs(r.bottom - innerHeight) < 1; }})()"
    );
    wait::for_js_true(page, &js, "the sheet at the foot, edge to edge").await
}

/// A tap opens only the picker; the picker's own text box takes typing and Enter commits (3905).
#[test]
fn a_finger_opens_the_sheet_and_types_in_it() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        let field = input("stay");
        // Short while unfocused (3904).
        let short = format!("document.querySelector({field:?}).value === '14–16 Sep'");
        wait::for_js_true(page, &short, "the short range")
            .await
            .unwrap();
        tap(page, &field).await.unwrap();
        sheet_shown(page).await.unwrap();
        let keyboardless = format!(
            "(() => {{ const i = document.querySelector({field:?}); \
             const own = document.querySelector({TYPED:?}); \
             return i.getAttribute('inputmode') === 'none' && i === document.activeElement \
               && !!own && !own.hasAttribute('inputmode') && own.value === i.value \
               && own.getAttribute('aria-labelledby').split(' ') \
                 .map((id) => document.getElementById(id).textContent).join(' ') === 'Stay Type it in'; }})()"
        );
        wait::for_js_true(page, &keyboardless, "inputmode none, and a named text box")
            .await
            .unwrap();
        tap(page, TYPED).await.unwrap();
        let focused = format!("document.querySelector({TYPED:?}) === document.activeElement");
        wait::for_js_true(page, &focused, "the text box focused by the tap")
            .await
            .unwrap();
        let next: String = page
            .evaluate(format!(
                "(() => {{ const own = document.querySelector({TYPED:?}); own.select(); \
                 return own.value.replace('14', '20'); }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        keyboard::insert_text(page, &next).await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        let committed = format!(
            "!document.querySelector({SHEET:?}) \
             && document.querySelector({field:?}) === document.activeElement \
             && document.querySelector('#echo').textContent.includes('2026-09-20')"
        );
        wait::for_js_true(page, &committed, "the typed range, the sheet closed")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a finger's picker field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3926: the clock face fills the sheet's width.
#[test]
fn the_clock_face_fills_the_sheet() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("at")).await.unwrap();
        sheet_shown(page).await.unwrap();
        let js = format!(
            "(() => {{ const f = document.querySelector('{SHEET} [data-slot=face]'); \
             if (!f) return false; const r = f.getBoundingClientRect(); \
             return r.width >= 300 && Math.abs(r.width - r.height) < 1; }})()"
        );
        wait::for_js_true(page, &js, "a face 300px or wider, round")
            .await
            .unwrap();
        fixture.console.assert_clean("a phone's clock").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3901: a digital clock's neighbours are 44px to a finger.
#[test]
fn a_finger_gets_44px_clock_neighbours() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("digital")).await.unwrap();
        sheet_shown(page).await.unwrap();
        let js = format!(
            "(() => {{ const n = [...document.querySelectorAll('{SHEET} [data-slot=neighbour]')]; \
             return n.length > 0 && n.every(e => e.getBoundingClientRect().height >= 43.5); }})()"
        );
        wait::for_js_true(page, &js, "44px neighbours")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3923: the color sheet's 44px tracks and swatches in one row, and its hex line types.
#[test]
fn a_phone_spreads_the_color_picker_over_the_sheet() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        let field = input("color");
        tap(page, &field).await.unwrap();
        sheet_shown(page).await.unwrap();
        let js = format!(
            "(() => {{ const s = document.querySelector({SHEET:?}); \
             const tracks = [...s.querySelectorAll('[data-slot=sliders] [data-slot=track]')]; \
             const swatches = [...s.querySelectorAll('[data-slot=swatches] > [data-slot=swatch]')]; \
             const pad = s.querySelector('[data-slot=saturation]').getBoundingClientRect(); \
             const tops = new Set(swatches.map(e => Math.round(e.getBoundingClientRect().top))); \
             return tracks.length > 0 && tracks.every(t => t.getBoundingClientRect().height >= 43.5) \
               && swatches.length === 3 && swatches.every(e => e.getBoundingClientRect().width >= 43.5) \
               && tops.size === 1 && pad.width >= 300 \
               && document.querySelector({field:?}).getAttribute('inputmode') === 'none' \
               && document.querySelector({TYPED:?}).value === '#228be6'; }})()"
        );
        wait::for_js_true(
            page,
            &js,
            "44px tracks, one row of 44px swatches, a hex line",
        )
        .await
        .unwrap();
        tap(page, TYPED).await.unwrap();
        let focused = format!("document.querySelector({TYPED:?}) === document.activeElement");
        wait::for_js_true(page, &focused, "the hex line focused by the tap")
            .await
            .unwrap();
        page.evaluate(format!("document.querySelector({TYPED:?}).select()"))
            .await
            .unwrap();
        keyboard::insert_text(page, "#40c057").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        let committed = format!(
            "!document.querySelector({SHEET:?}) \
             && document.querySelector({field:?}).value === '#40c057'"
        );
        wait::for_js_true(page, &committed, "the typed color, the sheet closed")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a phone's color sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4320: a second tap on the focused text box is no press for the blur that follows.
#[test]
fn a_tap_outside_closes_the_sheet_after_two_taps_in_the_text_box() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("color")).await.unwrap();
        sheet_shown(page).await.unwrap();
        let focused = format!("document.querySelector({TYPED:?}) === document.activeElement");
        for _ in 0..2 {
            tap(page, TYPED).await.unwrap();
            wait::for_js_true(page, &focused, "the text box focused")
                .await
                .unwrap();
        }
        // The page's top padding, above the sheet.
        let top = pointer::Point { x: 180.0, y: 4.0 };
        pointer::tap_at(page, top).await.unwrap();
        let closed = format!("!document.querySelector({SHEET:?})");
        wait::for_js_true(page, &closed, "the sheet closed by the tap outside")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

fn day(date: &str) -> String {
    format!("{SHEET} [data-slot=day][data-date='{date}']:not([data-outside])")
}

/// Todo 3903: a range scrolls its months between its readout and Save, its text box in view;
/// picks stay open until Save.
/// Todo 3901: its days are 44px to a finger.
#[test]
fn a_phone_range_scrolls_its_months_and_saves() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        let field = input("stay");
        tap(page, &field).await.unwrap();
        sheet_shown(page).await.unwrap();
        let readout = format!("document.querySelector('{SHEET} [data-slot=readout]')");
        let js = format!(
            "(() => {{ const r = {readout}; if (!r) return false; \
             const grids = document.querySelectorAll('{SHEET} [data-slot=months][data-scroll] [role=grid]'); \
             const days = [...document.querySelectorAll('{SHEET} [data-slot=day]')]; \
             const box = document.querySelector('{SHEET} [data-slot=scroller]'); \
             const typed = document.querySelector({TYPED:?}).getBoundingClientRect(); \
             return box.scrollHeight > box.clientHeight && typed.bottom <= innerHeight \
               && r.getBoundingClientRect().top >= 0 && r.textContent === '14 Sep–16 Sep' \
               && grids.length === 12 && grids[0].getAttribute('aria-label') === 'September 2026' \
               && days.every(d => d.getBoundingClientRect().width >= 43.5) \
               && !document.querySelector('{SHEET} [data-slot=nav]'); }})()"
        );
        wait::for_js_true(
            page,
            &js,
            "twelve months from September, 44px days, the readout",
        )
        .await
        .unwrap();
        tap(page, &day("2026-09-20")).await.unwrap();
        let started = format!("{readout}.textContent === '20 Sep–End date'");
        wait::for_js_true(page, &started, "a start, the end unset")
            .await
            .unwrap();
        // Scrolls the end into view first: CDP taps at the element's centre on screen.
        page.evaluate(format!(
            "document.querySelector(\"{}\").scrollIntoView({{block: 'center'}})",
            day("2026-10-02")
        ))
        .await
        .unwrap();
        tap(page, &day("2026-10-02")).await.unwrap();
        let picked = format!(
            "{readout} && {readout}.textContent === '20 Sep–2 Oct' \
             && document.querySelector('#echo').textContent.includes('2026-10-02')"
        );
        wait::for_js_true(page, &picked, "both ends, the sheet still open")
            .await
            .unwrap();
        tap(page, &format!("{SHEET} [data-slot=save] button"))
            .await
            .unwrap();
        let saved = format!("!document.querySelector({SHEET:?})");
        wait::for_js_true(page, &saved, "Save closed the sheet")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a phone's scrolling range")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4186: a start alone stays in the sheet; Save then keeps the old range.
#[test]
fn a_half_picked_range_saved_keeps_the_old_one() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("stay")).await.unwrap();
        sheet_shown(page).await.unwrap();
        tap(page, &day("2026-09-20")).await.unwrap();
        let old = "document.querySelector('#echo').textContent\
                   .includes('start: 2026-09-14, end: Some(2026-09-16)')";
        let started = format!(
            "document.querySelector('{SHEET} [data-slot=readout]')?.textContent \
             === '20 Sep–End date' && {old}"
        );
        wait::for_js_true(page, &started, "a start in the sheet, the old range kept")
            .await
            .unwrap();
        tap(page, &format!("{SHEET} [data-slot=save] button"))
            .await
            .unwrap();
        let kept = format!("!document.querySelector({SHEET:?}) && {old}");
        wait::for_js_true(page, &kept, "Save closed the sheet, the old range kept")
            .await
            .unwrap();
        fixture.console.assert_clean("a half-picked range").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3924: a sideways swipe over a day's month pages it; the title opens the month grid.
#[test]
fn a_swipe_pages_the_day_sheet() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("due")).await.unwrap();
        sheet_shown(page).await.unwrap();
        let title = |month: &str| {
            format!(
                "document.querySelector('{SHEET} [data-slot=title]')?.textContent === '{month}'"
            )
        };
        wait::for_js_true(page, &title("September 2026"), "September")
            .await
            .unwrap();
        let months = format!("{SHEET} [data-slot=months]");
        let at = pointer::centre_of(page, &months).await.unwrap();
        let by = |dx: f64| pointer::Point {
            x: at.x + dx,
            y: at.y,
        };
        pointer::swipe(page, by(60.0), by(-60.0), 6).await.unwrap();
        wait::for_js_true(page, &title("October 2026"), "a swipe left, October")
            .await
            .unwrap();
        pointer::swipe(page, by(-60.0), by(60.0), 6).await.unwrap();
        wait::for_js_true(page, &title("September 2026"), "a swipe right, September")
            .await
            .unwrap();
        tap(page, &format!("{SHEET} [data-slot=title]"))
            .await
            .unwrap();
        let grid = format!(
            "document.querySelectorAll('{SHEET} [data-slot=cells] [data-slot=cell]').length === 12"
        );
        wait::for_js_true(page, &grid, "the month grid")
            .await
            .unwrap();
        fixture.console.assert_clean("a swiped day sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The sheet's `translateY`, `null` once it is gone.
fn sheet_shift() -> String {
    format!(
        "(() => {{ const s = document.querySelector({SHEET:?}); \
         return s ? new DOMMatrix(getComputedStyle(s).transform).m42 : null; }})()"
    )
}

/// Records in `window.slid` whether the sheet was ever sent off the screen.
fn slide_watch() -> String {
    format!(
        "window.slid = false; new MutationObserver(() => {{ \
         const s = document.querySelector({SHEET:?}); \
         if (s && s.style.getPropertyValue('--lsx-swipe-drag').startsWith('calc')) window.slid = true; \
         }}).observe(document.body, {{ subtree: true, attributes: true, attributeFilter: ['style'] }})"
    )
}

/// Todo 3957: the sheet follows a finger down its strip, springs back from a short swipe and
/// slides off from a long one.
#[test]
fn a_swipe_down_slides_the_sheet_away() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        let strip = format!("{SHEET} [data-slot=sheet-handle]");
        let shifted = |by: f64| format!("Math.abs({}  - {by}) < 1", sheet_shift());
        tap(page, &input("due")).await.unwrap();
        sheet_shown(page).await.unwrap();
        let at = pointer::centre_of(page, &strip).await.unwrap();
        let down = |dy: f64| pointer::Point {
            x: at.x,
            y: at.y + dy,
        };
        frames::in_front(page, async {
            pointer::touch_down(page, &strip).await?;
            pointer::touch_move(page, down(10.0)).await?;
            pointer::touch_move(page, down(30.0)).await?;
            wait::for_js_true(
                page,
                &shifted(30.0),
                "the sheet 30px down, under the finger",
            )
            .await?;
            pointer::touch_up(page, down(30.0)).await
        })
        .await
        .unwrap();
        wait::for_js_true(page, &shifted(0.0), "a short swipe sprung back")
            .await
            .unwrap();
        page.evaluate(slide_watch()).await.unwrap();
        pointer::swipe(page, at, down(120.0), 2).await.unwrap();
        let gone = format!("!document.querySelector({SHEET:?}) && window.slid");
        wait::for_js_true(
            page,
            &gone,
            "a long swipe slid the sheet off, then closed it",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a swiped-away sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3957: with reduced motion a long swipe closes the sheet without sliding.
#[test]
fn reduced_motion_closes_a_swiped_sheet_at_once() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        e2e::browser::emulate_media(page, e2e::browser::Scheme::Light, Some(true))
            .await
            .unwrap();
        e2e::passes::motion::assert_reduced_motion_matches(page)
            .await
            .unwrap();
        tap(page, &input("due")).await.unwrap();
        sheet_shown(page).await.unwrap();
        page.evaluate(slide_watch()).await.unwrap();
        let at = pointer::centre_of(page, &format!("{SHEET} [data-slot=sheet-handle]"))
            .await
            .unwrap();
        let to = pointer::Point {
            x: at.x,
            y: at.y + 120.0,
        };
        pointer::swipe(page, at, to, 2).await.unwrap();
        let closed = format!("!document.querySelector({SHEET:?}) && !window.slid");
        wait::for_js_true(page, &closed, "closed without a slide")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a reduced-motion swiped sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3958: an explicit `columns: 2` range at 360px pages one month, no sideways scroll.
#[test]
fn a_phone_cuts_explicit_columns_to_the_months_that_fit() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("pair")).await.unwrap();
        sheet_shown(page).await.unwrap();
        let js = format!(
            "(() => {{ const s = document.querySelector({SHEET:?}); \
             const grids = s.querySelectorAll('[data-slot=months] > [role=grid]'); \
             const root = grids[0]?.closest('[data-state]'); \
             return grids.length === 1 && !s.querySelector('[data-scroll]') \
               && s.querySelectorAll('[data-slot=nav]').length === 2 \
               && !root.matches('[data-state~=measuring]') && getComputedStyle(root).opacity === '1' \
               && s.scrollWidth <= s.clientWidth \
               && grids[0].getAttribute('aria-label') === 'September 2026'; }})()"
        );
        wait::for_js_true(page, &js, "one paged month, nothing sideways")
            .await
            .unwrap();
        tap(page, &format!("{SHEET} [data-slot=nav]:last-of-type"))
            .await
            .unwrap();
        let paged = format!(
            "document.querySelector('{SHEET} [data-slot=months] > [role=grid]')\
             ?.getAttribute('aria-label') === 'October 2026'"
        );
        wait::for_js_true(page, &paged, "the next month, one at a time")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a phone's explicit columns")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Above `sm`, with a mouse: a popover under the field, typed in directly, no text box of its own.
#[test]
fn a_desktop_keeps_the_popover_and_the_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/picker-phone", Viewport::Desktop)
            .await
            .unwrap();
        let mut d = e2e::driver::Web { fixture };
        let field = input("stay");
        d.click(&field).await.unwrap();
        let js = format!(
            "(() => {{ const s = document.querySelector({SHEET:?}); \
             if (!s || getComputedStyle(s).visibility !== 'visible') return false; \
             const i = document.querySelector({field:?}); \
             return s.getBoundingClientRect().top >= i.getBoundingClientRect().bottom - 1 \
               && !i.hasAttribute('inputmode') && !s.querySelector('[data-slot=text-input]'); }})()"
        );
        wait::for_js_true(&d.fixture.page, &js, "a popover, typing as ever")
            .await
            .unwrap();
        d.finish("a desktop picker field").await.unwrap();
    });
}

/// Records the `sheet`'s `translateY` per frame, the finger's lift and the sheet's removal.
fn slide_probe(sheet: &str) -> String {
    format!(
        "(() => {{ const slide = window.slide = {{ frames: [], up: null, gone: null, events: [] }}; \
         addEventListener('pointerup', () => {{ slide.up ??= performance.now(); }}, {{ capture: true }}); \
         const box = e => {{ const r = e?.getBoundingClientRect(); return r && [r.x, r.y, r.width, r.height].map(Math.round); }}; \
         slide.strip = box(document.querySelector({sheet:?} + ' [data-slot=sheet-handle]')); \
         slide.title = box(document.querySelector({sheet:?} + ' [data-slot=title]')); \
         for (const type of ['pointerdown', 'pointercancel']) \
           addEventListener(type, e => slide.events.push(`${{type}}@${{e.target.dataset?.slot ?? e.target.tagName}}(${{Math.round(e.clientX)}},${{Math.round(e.clientY)}})`), {{ capture: true }}); \
         const tick = () => {{ const s = document.querySelector({sheet:?}); \
           if (!s) {{ slide.gone = performance.now(); return; }} \
           slide.frames.push([performance.now(), new DOMMatrix(getComputedStyle(s).transform).m42]); \
           requestAnimationFrame(tick); }}; \
         requestAnimationFrame(tick); }})()"
    )
}

/// Todo 3971: a finger's swipe down the middle of `sheet`'s strip slides the sheet off and
/// removes it (3957); the frames are only reported.
async fn swipe_slides_off<D: Driver>(d: &mut D, trigger: &str, sheet: &str) -> Result<()> {
    d.click(trigger).await?;
    eventually(d, "the sheet", async |d| {
        Ok(d.exists(sheet).await? && d.style(sheet, "visibility").await? == "visible")
    })
    .await?;
    d.evaluate(&slide_probe(sheet)).await?;
    // Not `drag`: its scroll into view moves the sheet's own scroller under the strip.
    let strip = d.rect(&format!("{sheet} [data-slot=sheet-handle]")).await?;
    d.drag_at(
        strip.x + strip.width / 2.0,
        strip.y + strip.height / 2.0,
        0.0,
        150.0,
    )
    .await?;
    // The end state only: under emulator load the finger's moves and the slide may draw no frame (4044).
    let Err(error) = eventually(d, "the swiped sheet gone", async |d| {
        Ok(d.evaluate("window.slide.gone !== null").await? == serde_json::Value::Bool(true))
    })
    .await
    else {
        return Ok(());
    };
    let slide = d.evaluate("JSON.stringify(window.slide)").await?;
    let slide: serde_json::Value = serde_json::from_str(slide.as_str().unwrap_or("{}"))?;
    let frames: Vec<(f64, f64)> = serde_json::from_value(slide["frames"].clone())?;
    let up = slide["up"].as_f64();
    // The last frames before the lift and all after it, as (ms from the lift, px down).
    let trace: Vec<(i64, i64)> = frames
        .iter()
        .filter(|&&(t, _)| up.is_none_or(|up| t > up - 100.0))
        .map(|&(t, y)| ((t - up.unwrap_or(0.0)) as i64, y as i64))
        .collect();
    bail!(
        "{error}: strip {}, title {}, events {}, frames {trace:?}",
        slide["strip"],
        slide["title"],
        slide["events"]
    )
}

async fn a_date_sheet_slides_off<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    swipe_slides_off(d, &input("due"), SHEET).await
}

async fn a_select_sheet_slides_off<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    swipe_slides_off(d, "#plain [role=combobox]", "[data-slot=dropdown]").await
}

e2e::scenario!(
    android_a_swipe_slides_a_date_sheet_off,
    "/picker-phone",
    a_date_sheet_slides_off,
    android_only("a phone's finger; the web's is a_swipe_down_slides_the_sheet_away")
);
e2e::scenario!(
    android_a_swipe_slides_a_select_sheet_off,
    "/select-phone",
    a_select_sheet_slides_off,
    android_only(
        "a phone's finger; the web's is select_phone::a_swipe_down_on_the_grabber_closes_the_sheet"
    )
);

const TALLY: &str = "#tally";

fn tally_is(taps: u32) -> String {
    format!("document.querySelector({TALLY:?}).textContent.trim() === 'Tapped {taps}'")
}

/// Todo 4826: a tap above the sheet only closes it, as a scrim would; the button under it stays untouched.
#[test]
fn a_tap_outside_the_sheet_only_closes_it() {
    block_on(async {
        let fixture = phone().await;
        let page = &fixture.page;
        tap(page, &input("due")).await.unwrap();
        sheet_shown(page).await.unwrap();
        tap(page, TALLY).await.unwrap();
        let closed = format!("!document.querySelector({SHEET:?})");
        wait::for_js_true(page, &closed, "the sheet closed by the tap outside")
            .await
            .unwrap();
        crate::settle::painted(page).await.unwrap();
        wait::for_js_true(page, &tally_is(0), "the tap's click swallowed")
            .await
            .unwrap();
        // Closed, the next tap lands.
        tap(page, TALLY).await.unwrap();
        wait::for_js_true(page, &tally_is(1), "the next tap clicks")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a tap outside the sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4826: above `sm` the dropdown is a popover, and a click outside both closes it and lands.
#[test]
fn a_click_outside_the_dropdown_above_sm_still_lands() {
    block_on(async {
        let fixture = Fixture::open("/picker-phone", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        pointer::click(page, &input("due")).await.unwrap();
        wait::for_visible(page, SHEET).await.unwrap();
        pointer::click(page, TALLY).await.unwrap();
        let closed = format!("!document.querySelector({SHEET:?})");
        wait::for_js_true(page, &closed, "the dropdown closed by the click outside")
            .await
            .unwrap();
        wait::for_js_true(page, &tally_is(1), "the click landed")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a click outside the dropdown")
            .unwrap();
        fixture.close().await.unwrap();
    });
}
