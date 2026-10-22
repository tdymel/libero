//! The picker fields at 360px: the phone sheet (todo 3900), a finger's keyboardless tap and the
//! picker's own text box (3905, 3923), the clock face (3926), 44px targets (3901), short ranges (3904),
//! the scrolling range (3903) and the swiped month (3924).

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

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

async fn tap(page: &Page, selector: &str) -> Result<()> {
    let at = pointer::touch_down(page, selector).await?;
    pointer::touch_up(page, at).await
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
               && document.getElementById(own.getAttribute('aria-labelledby')).textContent === 'Stay'; }})()"
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
        pointer::touch_drag(page, by(60.0), by(-60.0), 6)
            .await
            .unwrap();
        wait::for_js_true(page, &title("October 2026"), "a swipe left, October")
            .await
            .unwrap();
        pointer::touch_drag(page, by(-60.0), by(60.0), 6)
            .await
            .unwrap();
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

/// Above `sm`, with a mouse: a popover under the field, typed in directly, no text box of its own.
#[test]
fn a_desktop_keeps_the_popover_and_the_keyboard() {
    block_on(async {
        let fixture = Fixture::open("/picker-phone", Viewport::Desktop)
            .await
            .unwrap();
        let mut d = e2e::driver::Web { fixture };
        use e2e::driver::Driver;
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
