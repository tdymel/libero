//! The list fields at 360px: `Select`, `MultiSelect` and `PhoneField` open the phone sheet
//! (todos 3897, 3925), touch rows and targets (3898, 3901), no search focus on a touch open (3899),
//! and a low custom `Cascader` trigger clears the sheet (3914).

use anyhow::Result;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::pointer::tap;
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

/// Todo 3982: with `open_on_focus`, closing the sheet with Done does not reopen it.
#[test]
fn an_open_on_focus_sheet_stays_closed_after_done() {
    const TRIGGER: &str = "[role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone/open-on-focus").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        tap(page, &format!("{SHEET} [data-slot=sheet-footer] button"))
            .await
            .unwrap();
        closed(page, TRIGGER).await.unwrap();
        // Past the effects after the close: a late reopen would land by now.
        crate::settle::painted(page).await.unwrap();
        closed(page, TRIGGER).await.unwrap();
        fixture
            .console
            .assert_clean("an open_on_focus sheet closed by Done")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

const SEARCH: &str = "[data-slot=dropdown] [data-slot=search]";

/// Todo 3899: a touch open leaves focus, and the combobox role, on the trigger in `field`,
/// with the search box pinned atop the rows; its own tap focuses it and keeps the sheet.
async fn touch_open_holds_the_trigger(field: &str, trigger: &str, roles: bool) -> Result<()> {
    let fixture = phone("/select-phone").await;
    let page = &fixture.page;
    tap(page, &format!("{field} {trigger}")).await?;
    sheet_shown(page).await?;
    let held = format!(
        "(() => {{ const box = document.querySelector({SEARCH:?}); \
         const list = document.querySelector('[data-slot=dropdown] [role=listbox]'); \
         const on = document.activeElement; \
         return !!box && !!list && !!on.closest({field:?}) && on.matches({trigger:?}) \
           && (!{roles} || (on.getAttribute('role') === 'combobox' && !box.hasAttribute('role'))) \
           && box.getBoundingClientRect().bottom <= list.getBoundingClientRect().top; }})()"
    );
    wait::for_js_true(page, &held, "the trigger focused, the box above the rows").await?;
    tap(page, SEARCH).await?;
    let searching = format!(
        "(() => {{ const box = document.querySelector({SEARCH:?}); \
         return !!box && box === document.activeElement \
           && (!{roles} || box.getAttribute('role') === 'combobox'); }})()"
    );
    wait::for_js_true(page, &searching, "the box focused by its tap").await?;
    sheet_shown(page).await?;
    fixture.console.assert_clean("a touch-opened search")?;
    fixture.close().await
}

#[test]
fn a_touch_open_keeps_the_select_trigger_focused() {
    block_on(touch_open_holds_the_trigger(
        "#searched",
        "[role=combobox]",
        true,
    ))
    .unwrap();
}

#[test]
fn a_touch_open_keeps_the_country_picker_focused() {
    block_on(touch_open_holds_the_trigger(
        "#phone",
        "[id$='-picker']",
        false,
    ))
    .unwrap();
}

#[test]
fn a_touch_open_keeps_the_cascader_trigger_focused() {
    block_on(touch_open_holds_the_trigger(
        "#cascade",
        "[role=combobox]",
        true,
    ))
    .unwrap();
}

/// Todo 3949: a press on the search box that turns into a scroll leaves no mark, so the
/// held trigger's next blur still closes the list.
#[test]
fn a_cancelled_search_press_keeps_the_blur_closing() {
    const TRIGGER: &str = "#searched [role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        // A press, then the cancel a browser sends when it turns into a scroll; CDP has no touch cancel.
        let press = format!(
            "(() => {{ const box = document.querySelector({SEARCH:?}); \
             for (const type of ['pointerdown', 'pointercancel']) \
               box.dispatchEvent(new PointerEvent(type, {{ bubbles: true, pointerType: 'touch', isPrimary: true }})); \
             return true; }})()"
        );
        wait::for_js_true(page, &press, "a cancelled press on the search box")
            .await
            .unwrap();
        crate::settle::painted(page).await.unwrap();
        let blur = format!(
            "(() => {{ const on = document.activeElement; \
             if (!on.matches({TRIGGER:?})) return false; on.blur(); return true; }})()"
        );
        wait::for_js_true(page, &blur, "the held trigger blurred")
            .await
            .unwrap();
        closed(page, TRIGGER).await.unwrap();
        fixture
            .console
            .assert_clean("a cancelled search press")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todos 3902, 3940: the keyboard the search box's tap raises shrinks the WebView, and the sheet
/// sits flush above it with the box and a row in view.
async fn sheet_above_the_keyboard<D: Driver>(d: &mut D, field: &str) -> Result<()> {
    let (_, tall) = d.viewport().await?;
    d.click(&format!("{field} [role=combobox]")).await?;
    eventually(d, "the sheet", async |d| d.exists(SEARCH).await).await?;
    d.click(SEARCH).await?;
    eventually(d, "the soft keyboard", async |d| {
        d.soft_keyboard_shown().await
    })
    .await?;
    let flush = format!(
        "(() => {{ const s = document.querySelector({SHEET:?}); \
         const box = s && s.querySelector('[data-slot=search]'); \
         const row = s && s.querySelector('[role=option]'); \
         if (!box || !row || document.activeElement !== box || innerHeight > {tall} - 100) return false; \
         const r = s.getBoundingClientRect(), b = box.getBoundingClientRect(), o = row.getBoundingClientRect(); \
         return Math.abs(r.bottom - innerHeight) < 1 && r.top >= 0 && b.top >= r.top \
           && o.top > 0 && o.bottom <= innerHeight; }})()"
    );
    let shown = eventually(d, "the sheet flush above the keyboard", async |d| {
        Ok(d.evaluate(&flush).await? == serde_json::Value::Bool(true))
    })
    .await;
    if let Err(error) = shown {
        let state = d
            .evaluate(&format!(
                "JSON.stringify({{ inner: innerHeight, visual: visualViewport.height, \
                 sheet: document.querySelector({SHEET:?})?.getBoundingClientRect(), \
                 box: document.querySelector({SEARCH:?})?.getBoundingClientRect() }})"
            ))
            .await?;
        anyhow::bail!("{error}: {state} (before the keyboard {tall})");
    }
    // Back hides the keyboard first, so the next scenario starts without it.
    d.press_back().await?;
    eventually(d, "Back to hide the keyboard", async |d| {
        Ok(!d.soft_keyboard_shown().await?)
    })
    .await
}

async fn select_sheet_above_the_keyboard<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    sheet_above_the_keyboard(d, "#searched").await
}

async fn cascader_sheet_above_the_keyboard<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    sheet_above_the_keyboard(d, "#cascade").await
}

e2e::scenario!(
    android_a_searchable_select_sheet_sits_above_the_keyboard,
    "/select-phone",
    select_sheet_above_the_keyboard,
    android_only("only Android reports and resizes for a soft keyboard")
);
e2e::scenario!(
    android_a_cascader_sheet_sits_above_the_keyboard,
    "/select-phone",
    cascader_sheet_above_the_keyboard,
    android_only("only Android reports and resizes for a soft keyboard")
);

/// Todo 3899: the held trigger's blur closes the list.
#[test]
fn a_held_trigger_closes_on_blur() {
    const TRIGGER: &str = "#searched [role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone").await;
        let page = &fixture.page;
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        let blur = format!(
            "(() => {{ const on = document.activeElement; \
             if (!on.matches({TRIGGER:?})) return false; on.blur(); return true; }})()"
        );
        wait::for_js_true(page, &blur, "the held trigger blurred")
            .await
            .unwrap();
        closed(page, TRIGGER).await.unwrap();
        fixture
            .console
            .assert_clean("a blurred held trigger")
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
        pointer::swipe(page, from, to, 8).await.unwrap();
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

/// Todo 4580: a `Select` trigger in the bottom of the frame is scrolled above the sheet.
#[test]
fn a_low_select_trigger_clears_the_sheet() {
    const TRIGGER: &str = "#low [role=combobox]";
    block_on(async {
        let fixture = phone("/select-phone/low-select").await;
        let page = &fixture.page;
        let js = format!(
            "(() => {{ const r = document.querySelector({TRIGGER:?}).getBoundingClientRect(); \
             return r.top > innerHeight - 270 && r.bottom < innerHeight; }})()"
        );
        wait::for_js_true(page, &js, "the trigger in the bottom 270px, in frame")
            .await
            .unwrap();
        tap(page, TRIGGER).await.unwrap();
        sheet_shown(page).await.unwrap();
        let js = format!(
            "document.querySelector({TRIGGER:?}).getBoundingClientRect().bottom \
             <= document.querySelector({SHEET:?}).getBoundingClientRect().top"
        );
        wait::for_js_true(page, &js, "the trigger above the sheet")
            .await
            .unwrap();
        fixture.console.assert_clean("a low select").unwrap();
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
