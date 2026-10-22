//! The picker fields at 360px: the phone sheet (todo 3900), a finger's keyboardless tap and the
//! picker's own text box (3905, 3923), the clock face (3926), 44px targets (3901), short ranges (3904).

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
