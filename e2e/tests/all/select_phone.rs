//! The list fields at 360px: `Select`, `MultiSelect` and `PhoneField` open the phone sheet
//! (todos 3897, 3925), touch rows and targets (3898, 3901), and a low custom `Cascader` trigger clears the sheet (3914).

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const SHEET: &str = "[data-slot=dropdown]";

async fn phone(route: &str) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Mobile).await.unwrap();
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

async fn closed(page: &Page, trigger: &str) -> Result<()> {
    let js =
        format!("document.querySelector({trigger:?}).getAttribute('aria-expanded') !== 'true'");
    wait::for_js_true(page, &js, "the list closed").await
}

#[test]
fn a_phone_opens_a_select_in_the_sheet() {
    const TRIGGER: &str = "#plain [role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        // A finger's rows (3898), the grabber, and no keyboard ring on a touch open.
        let js = format!(
            "(() => {{ const s = document.querySelector({SHEET:?}); \
             const rows = [...s.querySelectorAll('[role=option]')]; \
             const handle = s.querySelector('[data-slot=sheet-handle]'); \
             const active = s.querySelector('[role=option][data-state~=active]') || rows[0]; \
             return rows.length > 0 && rows.every(r => r.getBoundingClientRect().height >= 47.5) \
               && handle && handle.getBoundingClientRect().height > 0 \
               && getComputedStyle(active).boxShadow === 'none' \
               && !s.querySelector('[data-slot=sheet-footer]'); }})()"
        );
        wait::for_js_true(page, &js, "48px rows, a grabber, no ring")
            .await
            .unwrap();
        tap(page, &format!("{SHEET} [role=option]:nth-child(2)"))
            .await
            .unwrap();
        closed(page, TRIGGER).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#echo').textContent.startsWith('Some(')",
            "the pick",
        )
        .await
        .unwrap();
        fixture
            .console
            .assert_clean("a phone's select sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3925: a pick keeps the sheet open, rows show a check, Done closes it.
#[test]
fn a_phone_multi_select_closes_on_done() {
    const TRIGGER: &str = "#many [role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        tap(page, &format!("{SHEET} [role=option][aria-selected=false]"))
            .await
            .unwrap();
        let js = format!(
            "(() => {{ const s = document.querySelector({SHEET:?}); \
             const done = s && s.querySelector('[data-slot=sheet-footer] button'); \
             const checks = s ? [...s.querySelectorAll('[role=option][aria-selected=true] [data-slot=check]')] : []; \
             const r = done && done.getBoundingClientRect(); \
             return checks.length === 2 && checks.every(c => c.getBoundingClientRect().width > 0) \
               && !!r && r.height > 0 && r.bottom <= innerHeight; }})()"
        );
        wait::for_js_true(page, &js, "two checks and a Done in view")
            .await
            .unwrap();
        tap(page, &format!("{SHEET} [data-slot=sheet-footer] button"))
            .await
            .unwrap();
        closed(page, TRIGGER).await.unwrap();
        fixture
            .console
            .assert_clean("a phone's multi-select")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3925: the search box sits pinned atop the sheet's rows.
#[test]
fn a_phone_sheet_pins_the_search_box_on_top() {
    const SEARCH: &str = "[data-slot=dropdown] [data-slot=search]";
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, "#searched [role=combobox]").await.unwrap();
        sheet_shown(page).await.unwrap();
        let pinned = format!(
            "(() => {{ const box = document.querySelector({SEARCH:?}); \
             const list = document.querySelector('[data-slot=dropdown] [role=listbox]'); \
             return !!box && box === document.activeElement \
               && box.getBoundingClientRect().bottom <= list.getBoundingClientRect().top; }})()"
        );
        wait::for_js_true(page, &pinned, "the box focused above the rows")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a phone's searchable sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Maintainer 2026-10-22: a swipe down from the grabber closes the sheet.
#[test]
fn a_swipe_down_on_the_grabber_closes_the_sheet() {
    const TRIGGER: &str = "#plain [role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        let from = pointer::centre_of(page, "[data-slot=sheet-handle]")
            .await
            .unwrap();
        let to = pointer::Point {
            x: from.x,
            y: from.y + 150.0,
        };
        pointer::touch_drag(page, from, to, 8).await.unwrap();
        closed(page, TRIGGER).await.unwrap();
        fixture.console.assert_clean("a swiped sheet").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn a_phone_opens_the_country_list_in_the_sheet() {
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, "#phone [id$='-picker']").await.unwrap();
        sheet_shown(page).await.unwrap();
        fixture
            .console
            .assert_clean("a phone's country sheet")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3901: a held chip's x is 44px to a finger.
#[test]
fn a_finger_gets_a_44px_chip_remove() {
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        // Turns touch emulation on, so the pointer is coarse.
        tap(page, "#echo").await.unwrap();
        let js = "(() => { const x = document.querySelector('#many [data-slot=remove] button'); \
                  const area = getComputedStyle(x, '::before'); \
                  return parseFloat(area.width) >= 43.5 && parseFloat(area.height) >= 43.5; })()";
        wait::for_js_true(page, js, "a 44px remove target")
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 3914 and 3908: the sheet scrolls a low custom trigger above itself.
#[test]
fn a_low_custom_trigger_clears_the_sheet() {
    const TRIGGER: &str = "button[role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone/low-trigger").await;
        let page = &fixture.page;
        keyboard::tab_to(page, TRIGGER, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        sheet_shown(page).await.unwrap();
        let js = format!(
            "document.querySelector({TRIGGER:?}).getBoundingClientRect().bottom \
             <= document.querySelector({SHEET:?}).getBoundingClientRect().top"
        );
        wait::for_js_true(page, &js, "the trigger above the sheet")
            .await
            .unwrap();
        fixture
            .console
            .assert_clean("a low custom trigger")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Above `sm` the lists stay popovers under the field.
#[test]
fn a_desktop_keeps_the_popover() {
    block_on(async {
        let fixture = Fixture::open("/select-phone", Viewport::Desktop)
            .await
            .unwrap();
        let mut d = e2e::driver::Web { fixture };
        use e2e::driver::Driver;
        d.click("#plain [role=combobox]").await.unwrap();
        let js = format!(
            "(() => {{ const s = document.querySelector({SHEET:?}); \
             if (!s || getComputedStyle(s).visibility !== 'visible') return false; \
             const handle = s.querySelector('[data-slot=sheet-handle]'); \
             const field = document.querySelector('#plain [role=combobox]').getBoundingClientRect(); \
             return s.getBoundingClientRect().top >= field.bottom - 1 \
               && (!handle || getComputedStyle(handle).display === 'none'); }})()"
        );
        wait::for_js_true(&d.fixture.page, &js, "a popover, no grabber")
            .await
            .unwrap();
        d.finish("a desktop select").await.unwrap();
    });
}
