//! `FloatingWindow` (379): the non-modal overlay archetype, plus keyboard move/resize whose
//! `onmove`/`onresize` reports must match the drawn rect. Geometry is desktop only.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, Rect, eventually, eventually_focused};
use e2e::suite::Step;
use e2e::wait::PageBounded;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{focus, keyboard, pointer, target_size},
    wait,
};

pub const TRIGGER: &str = "#open-window";
pub const DIALOG: &str = "[role=dialog]";
/// The title bar, which is also the keyboard move handle.
pub const HANDLE: &str = "[role=dialog] [data-slot=handle]";
/// The corner grip, which is the resize handle.
pub const SEPARATOR: &str = "[role=dialog] [role=separator]";
/// The close button in the title bar.
pub const CLOSE: &str = "[role=dialog] [data-slot=title-bar] > [data-slot=close]";
/// The title bar's menu button: Move, Resize, Reset (todo 570).
pub const MENU: &str = "[role=dialog] [data-slot=menu]";
pub const MOVE_REPORT: &str = "#move-report";
pub const RESIZE_REPORT: &str = "#resize-report";

/// What the fixture asks for in the window's own `sx`. Home and End hand the
/// window `0x0` and `u16::MAX`, and these are what has to come back.
pub(super) const MIN: (f64, f64) = (240.0, 120.0);
pub(super) const MAX: (f64, f64) = (480.0, 360.0);

/// The theme's `move_step` and `resize_step`.
pub(super) const STEP: f64 = 10.0;

/// Enough to cross the trigger and the two readouts, or the window's own
/// handle, close button, body button and separator.
pub(super) const TAB_BUDGET: usize = 8;

/// Todo 3151: the handle's `1.5em` follows the page's font, so a 13px page draws it 19.5px
/// high; its hit area stays 24px.
#[test]
fn the_move_handle_keeps_24px_under_a_smaller_page_font() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded("document.documentElement.style.fontSize = '13px'")
            .await
            .unwrap();
        pointer::click(page, TRIGGER).await.unwrap();
        wait::for_visible(page, HANDLE).await.unwrap();
        target_size::assert_minimum(page, HANDLE).await.unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("floating_window", "/floating-window")
        .focusable(TRIGGER)
        // The window's text is portaled beside the fixture; without this, axe never
        // looking at it reads as clean (327).
        .contrast_covers(DIALOG)
        // Drawn 20x20 (an `ActionIcon` at `size: "sm"`), it takes presses in
        // an invisible 24x24 box (todos 505, 566), so it meets 2.5.8 outright.
        .targets(CLOSE)
        .targets(MENU)
        .targets(HANDLE)
        .targets_spaced(SEPARATOR)
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .run();
}

/// The title-bar menu, then Move's step buttons, 2.5.7's alternative to a drag (todo 1773).
/// Apart from the rest: the open menu covers the corner grip the spacing pass measures.
#[test]
fn the_menu_and_the_step_buttons_meet_the_baseline() {
    Suite::new("floating_window-steps", "/floating-window")
        .contrast_covers(DIALOG)
        .targets("[role=menu] [role=menuitem]")
        .targets("[data-slot=steps] button")
        .state(
            "open",
            &[Step::TabTo(TRIGGER), Step::Press(keyboard::ENTER)],
            DIALOG,
        )
        .state(
            "menu",
            &[Step::TabTo(MENU), Step::Press(keyboard::ENTER)],
            "[role=menu]",
        )
        // Todo 2637: the steps show before their first button takes focus; a snapshot
        // between the two caught the ring elsewhere.
        .state(
            "steps",
            &[Step::Press(keyboard::ENTER)],
            "[data-slot=steps] :focus",
        )
        .run();
}

#[test]
fn it_honours_the_overlay_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/floating-window", viewport).await.unwrap();

            Overlay {
                trigger: TRIGGER,
                panel: DIALOG,
                // Non-modal: the page behind it stays reachable, which is the
                // difference between this and `Modal` or `Lightbox`.
                traps_focus: false,
                tab_budget: TAB_BUDGET,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("the window contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// An arrow on the title bar moves the window by the theme's step, and the
/// rect it reports is the rect the browser drew.
#[test]
fn the_title_bar_moves_the_window_and_reports_where_it_landed() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        keyboard_move(&fixture.page).await.unwrap();
        fixture.console.assert_clean("a keyboard move").unwrap();
        fixture.close().await.unwrap();
    });
}

pub async fn keyboard_move(page: &Page) -> Result<()> {
    open(page).await?;
    // An unpinned window's handle is focusable; `assert_focus_ring` tabs to it from the
    // window root and leaves focus there for the arrows below.
    let ring = focus::assert_focus_ring(page, HANDLE, TAB_BUDGET).await?;
    focus::assert_ring_contrast(&ring)?;

    let before = rect(page, DIALOG).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    let after = wait_for_move(page, before, (STEP, 0.0)).await?;
    assert_report(page, MOVE_REPORT, after, "the move right").await?;

    keyboard::press(page, keyboard::ARROW_UP).await?;
    let after = wait_for_move(page, after, (0.0, -STEP)).await?;
    assert_report(page, MOVE_REPORT, after, "the move up").await?;
    Ok(())
}

pub async fn keyboard_resize(page: &Page) -> Result<()> {
    open(page).await?;
    let ring = focus::assert_focus_ring(page, SEPARATOR, TAB_BUDGET).await?;
    focus::assert_ring_contrast(&ring)?;

    let before = rect(page, DIALOG).await?;
    keyboard::press(page, keyboard::ARROW_RIGHT).await?;
    let stepped = wait_for_size(page, (before.2 + STEP, before.3), "a step right").await?;
    assert_report(page, RESIZE_REPORT, stepped, "the step right").await?;
    assert_separator(page, stepped, "the step right").await?;

    // `0x0`, which the caller's minimum answers.
    keyboard::press(page, keyboard::HOME).await?;
    let smallest = wait_for_size(page, MIN, "Home").await?;
    assert_report(page, RESIZE_REPORT, smallest, "Home").await?;
    assert_separator(page, smallest, "Home").await?;

    // `u16::MAX`, which the caller's maximum answers. The viewport is 1280x800
    // here, so it is the caller's bounds that clamp and not the screen.
    keyboard::press(page, keyboard::END).await?;
    let largest = wait_for_size(page, MAX, "End").await?;
    assert_report(page, RESIZE_REPORT, largest, "End").await?;
    assert_separator(page, largest, "End").await
}

/// The separator tells a screen reader the window's size: value the drawn width, text the
/// drawn size, range the caller's bounds in pixels (388).
async fn assert_separator(page: &Page, drawn: (f64, f64, f64, f64), what: &str) -> Result<()> {
    let expected = format!(
        "{} {} by {} pixels {} {}",
        drawn.2.round(),
        drawn.2.round(),
        drawn.3.round(),
        MIN.0,
        MAX.0
    );
    let read = format!(
        "(() => {{ const el = document.querySelector({}); if (!el) return null; \
         return ['aria-valuenow', 'aria-valuetext', 'aria-valuemin', 'aria-valuemax']\
         .map(name => el.getAttribute(name)).join(' '); }})()",
        serde_json::to_string(SEPARATOR)?
    );
    let (js, wanted) = (read.as_str(), expected.as_str());
    let settled = wait::until(
        &format!("the separator to announce {expected:?} after {what}"),
        || async move {
            let text: Option<String> = page.evaluate_bounded(js.to_string()).await?.into_value()?;
            Ok(text.as_deref() == Some(wanted))
        },
    )
    .await;
    if settled.is_err() {
        let text: Option<String> = page.evaluate_bounded(read.clone()).await?.into_value()?;
        bail!(
            "after {what} the window is drawn at {expected:?} (value, text, min, max), but the \
             separator says {:?}",
            text.unwrap_or_default()
        );
    }
    Ok(())
}

/// Todo 570: the title-bar menu moves, resizes and resets the window with
/// clicks alone (2.5.7), each reported like a keyboard move.
#[test]
fn the_title_bar_menu_moves_resizes_and_resets_without_a_drag() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        menu_adjust(&fixture.page).await.unwrap();
        fixture.console.assert_clean("the title-bar menu").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn menu_adjust(page: &Page) -> Result<()> {
    open(page).await?;
    let opened = rect(page, DIALOG).await?;

    choose(page, "Move").await?;
    expect_focus(
        page,
        "[aria-label='Move up']",
        "Move to focus the first step button",
    )
    .await?;
    // The buttons' row makes the window taller, and a placed window recentres.
    let showing = rect(page, DIALOG).await?;
    pointer::click(page, "[data-slot=steps] [aria-label='Move right']").await?;
    let moved = wait_for_move(page, showing, (STEP, 0.0)).await?;
    assert_report(page, MOVE_REPORT, moved, "the Move right button").await?;
    // Escape leaves the buttons, not the window.
    keyboard::press(page, keyboard::ESCAPE).await?;
    expect_focus(page, MENU, "Escape to hand focus back to the menu button").await?;
    if page
        .evaluate_bounded("!!document.querySelector('[data-slot=steps]')")
        .await?
        .into_value::<bool>()?
        || rect(page, DIALOG).await.is_err()
    {
        bail!("Escape on the step buttons should hide them and keep the window open");
    }

    choose(page, "Resize").await?;
    expect_focus(
        page,
        "[aria-label='Shorter']",
        "Resize to focus the first step button",
    )
    .await?;
    pointer::click(page, "[data-slot=steps] [aria-label='Wider']").await?;
    let wider = wait_for_size(page, (moved.2 + STEP, moved.3), "the Wider button").await?;
    assert_report(page, RESIZE_REPORT, wider, "the Wider button").await?;
    pointer::click(page, "[data-slot=steps] button:last-child").await?;
    expect_focus(page, MENU, "Done to hand focus back to the menu button").await?;

    choose(page, "Reset position and size").await?;
    let reset = format!(
        "(r => Math.abs(r.x - {}) <= 1 && Math.abs(r.y - {}) <= 1 && Math.abs(r.width - {}) <= 1)\
         (document.querySelector('[role=dialog]').getBoundingClientRect())",
        opened.0, opened.1, opened.2
    );
    if wait::for_js_true(page, &reset, "Reset to put the window back")
        .await
        .is_err()
    {
        bail!(
            "after Reset the window is at {:?}, not where it opened, {opened:?}",
            rect(page, DIALOG).await?
        );
    }
    let back = rect(page, DIALOG).await?;
    assert_report(page, MOVE_REPORT, back, "Reset").await?;
    assert_report(page, RESIZE_REPORT, back, "Reset").await
}

/// Todo 3828: a browser resize after the window opened lowers its size cap, which the
/// separator's range announces.
#[test]
fn a_browser_resize_lowers_the_size_cap() {
    const NARROW: i64 = 700;
    block_on(async {
        let fixture = Fixture::open("/floating-window-sized", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open(page).await.unwrap();
        let widest = separator_max(page).await.unwrap();
        assert!(
            widest > NARROW as f64,
            "the cap {widest} was below {NARROW}"
        );

        page.execute_bounded(SetDeviceMetricsOverrideParams::new(
            NARROW,
            Viewport::Desktop.size().1,
            1.0,
            false,
        ))
        .await
        .unwrap();
        let narrowed = wait::until("the size cap to follow the viewport", || async move {
            Ok(separator_max(page).await? <= NARROW as f64)
        })
        .await;
        assert!(
            narrowed.is_ok(),
            "the cap stayed {} after the viewport shrank to {NARROW}",
            separator_max(page).await.unwrap()
        );
        fixture.close().await.unwrap();
    });
}

async fn separator_max(page: &Page) -> Result<f64> {
    let max: Option<f64> = page
        .evaluate_bounded(format!(
            "parseFloat(document.querySelector({})?.getAttribute('aria-valuemax'))",
            serde_json::to_string(SEPARATOR)?
        ))
        .await?
        .into_value()?;
    max.ok_or_else(|| anyhow::anyhow!("the separator has no aria-valuemax"))
}

/// Open the title-bar menu and click the item named `label`.
async fn choose(page: &Page, label: &str) -> Result<()> {
    pointer::click(page, MENU).await?;
    wait::for_visible(page, "[role=menu]").await?;
    let at: Option<(f64, f64)> = page
        .evaluate_bounded(format!(
            "(() => {{ const el = [...document.querySelectorAll('[role=menuitem]')].find(e => e.textContent.trim() === {label:?}); \
             if (!el) return null; const r = el.getBoundingClientRect(); return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
        ))
        .await?
        .into_value()?;
    let Some((x, y)) = at else {
        bail!("no menu item {label:?} in the open menu");
    };
    let at = pointer::Point { x, y };
    pointer::drag(page, at, at, 1).await
}

async fn expect_focus(page: &Page, selector: &str, what: &str) -> Result<()> {
    let check = format!(
        "document.activeElement === document.querySelector({})",
        serde_json::to_string(selector)?
    );
    if wait::for_js_true(page, &check, what).await.is_err() {
        let actual = focus::active_element(page).await?;
        bail!("{what}: focus is on {actual:?}");
    }
    Ok(())
}

/// Open the window from the keyboard and wait for focus to land in it.
pub(super) async fn open(page: &Page) -> Result<()> {
    keyboard::tab_to(page, TRIGGER, TAB_BUDGET).await?;
    keyboard::press(page, keyboard::ENTER).await?;
    wait::for_visible(page, DIALOG).await?;
    wait::for_js_true(
        page,
        &format!(
            "document.querySelector({})?.contains(document.activeElement)",
            serde_json::to_string(DIALOG)?
        ),
        "focus to land inside the window",
    )
    .await
}

/// `(x, y, width, height)` of `selector`, as the browser drew it.
pub(super) async fn rect(page: &Page, selector: &str) -> Result<(f64, f64, f64, f64)> {
    let found: Option<(f64, f64, f64, f64)> = page
        .evaluate_bounded(format!(
            "(() => {{ const el = document.querySelector({}); if (!el) return null; \
             const r = el.getBoundingClientRect(); return [r.x, r.y, r.width, r.height]; }})()",
            serde_json::to_string(selector)?
        ))
        .await?
        .into_value()?;
    found.ok_or_else(|| anyhow::anyhow!("{selector} is not on the page"))
}

/// Wait until the window has moved by exactly `delta` and nothing else about
/// it has changed, then answer its new rect.
pub(super) async fn wait_for_move(
    page: &Page,
    before: (f64, f64, f64, f64),
    delta: (f64, f64),
) -> Result<(f64, f64, f64, f64)> {
    let wanted = (before.0 + delta.0, before.1 + delta.1);
    let described = format!("the window to move by {delta:?} to {wanted:?}");
    let settled = wait::until(&described, || async move {
        let now = rect(page, DIALOG).await?;
        Ok(close_to(now.0, wanted.0) && close_to(now.1, wanted.1))
    })
    .await;
    let now = rect(page, DIALOG).await?;
    if settled.is_err() {
        bail!("{described}, but it is at ({}, {})", now.0, now.1);
    }
    if !close_to(now.2, before.2) || !close_to(now.3, before.3) {
        bail!(
            "moving the window resized it: {}x{} became {}x{}",
            before.2,
            before.3,
            now.2,
            now.3
        );
    }
    Ok(now)
}

/// Wait until the window measures `wanted`, then answer its new rect.
pub(super) async fn wait_for_size(
    page: &Page,
    wanted: (f64, f64),
    what: &str,
) -> Result<(f64, f64, f64, f64)> {
    let described = format!("{what} to size the window to {}x{}", wanted.0, wanted.1);
    let settled = wait::until(&described, || async move {
        let now = rect(page, DIALOG).await?;
        Ok(close_to(now.2, wanted.0) && close_to(now.3, wanted.1))
    })
    .await;
    let now = rect(page, DIALOG).await?;
    if settled.is_err() {
        bail!("{described}, but it measures {}x{}", now.2, now.3);
    }
    Ok(now)
}

/// The report must match the drawn rect, as the `owed` effect exists to ensure; a report
/// from the handler carries the old rect. Waited for, since it arrives after the render.
async fn assert_report(
    page: &Page,
    selector: &str,
    drawn: (f64, f64, f64, f64),
    what: &str,
) -> Result<()> {
    let expected = format!(
        "{} {} {} {}",
        drawn.0.round(),
        drawn.1.round(),
        drawn.2.round(),
        drawn.3.round()
    );
    let read = format!(
        "document.querySelector({})?.textContent",
        serde_json::to_string(selector)?
    );
    let (js, wanted) = (read.as_str(), expected.as_str());
    let settled = wait::until(
        &format!("{what} to be reported as {expected:?}"),
        || async move {
            let text: Option<String> = page.evaluate_bounded(js.to_string()).await?.into_value()?;
            Ok(text.as_deref() == Some(wanted))
        },
    )
    .await;
    if settled.is_err() {
        let text: Option<String> = page.evaluate_bounded(read.clone()).await?.into_value()?;
        bail!(
            "after {what} the window is drawn at {expected:?}, but it reported {:?}",
            text.unwrap_or_default()
        );
    }
    Ok(())
}

/// A pixel of slack for fractional layout; every step here is 10px or a clamp.
fn close_to(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

/// Two windows: the newer stacks on top, Escape closes only the focused one and returns
/// focus to its trigger; a close from the page leaves focus where it is.
#[test]
fn each_window_closes_alone_and_a_close_from_the_page_keeps_focus() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-pair", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        activate(page, "#open-first").await;
        wait_for(page, "document.querySelectorAll('[role=dialog]').length === 1 && document.querySelector('[role=dialog]').contains(document.activeElement)", "the first window to take focus").await;
        activate(page, "#open-second").await;
        wait_for(
            page,
            "document.querySelectorAll('[role=dialog]').length === 2",
            "the second window",
        )
        .await;
        let z: Vec<i32> = page
            .evaluate_bounded("[...document.querySelectorAll('[role=dialog]')].map(d => +getComputedStyle(d.parentElement).zIndex)")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(z[1] > z[0], "the window opened last stacks on top: {z:?}");

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait_for(page, "document.querySelectorAll('[role=dialog]').length === 1 && document.activeElement?.id === 'open-second'", "Escape to close only the second window and focus its trigger").await;

        activate(page, "#close-first").await;
        wait_for(
            page,
            "document.querySelectorAll('[role=dialog]').length === 0",
            "the first window to close",
        )
        .await;
        crate::settle::painted(page).await.unwrap();
        let active: String = page
            .evaluate_bounded("document.activeElement?.id ?? ''")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(active, "close-first", "a close from the page moved focus");
        fixture.console.assert_clean("two windows").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4405: Escape on a focused tooltip's trigger closes the tooltip only; the next one the window.
#[test]
fn one_escape_closes_a_tooltip_in_a_window_and_not_the_window() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-tooltip", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        activate(page, TRIGGER).await;
        wait::for_visible(page, DIALOG).await.unwrap();
        page.evaluate_bounded("document.querySelector('#save').focus()")
            .await
            .unwrap();
        wait::for_visible(page, "#save-tip").await.unwrap();

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, "#save-tip").await.unwrap();
        crate::settle::painted(page).await.unwrap();
        assert!(
            wait::exists(page, DIALOG).await.unwrap(),
            "the Escape that closed the tooltip closed the window too"
        );

        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait_for(
            page,
            "!document.querySelector('[role=dialog]') && document.activeElement?.id === 'open-window'",
            "the second Escape to close the window and focus its trigger",
        )
        .await;
        fixture
            .console
            .assert_clean("a tooltip in a window")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 4404: the owner unmounting with its window open and focused hands focus back to the opener.
#[test]
fn an_unmounted_owner_hands_focus_back() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-owner", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        activate(page, "#show-owner").await;
        wait::for_visible(page, "#remove-owner").await.unwrap();
        activate(page, "#remove-owner").await;
        wait_for(
            page,
            "!document.querySelector('[role=dialog]') && document.activeElement?.id === 'show-owner'",
            "focus to return to the opener once the owner is gone",
        )
        .await;
        fixture.console.assert_clean("an unmounted owner").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A title with no break opportunity wraps in the title bar at 320px instead
/// of being clipped by the window (1.4.10).
#[test]
fn a_long_title_wraps_at_320px() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    block_on(async {
        let fixture = Fixture::open("/floating-window-pair", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute_bounded(SetDeviceMetricsOverrideParams::new(320, 640, 1.0, true))
            .await
            .unwrap();
        activate(page, "#open-first").await;
        wait::for_visible(page, DIALOG).await.unwrap();
        assert_fits_at_320(
            page,
            DIALOG,
            "[document.documentElement, document.querySelector('[role=dialog]'), document.querySelector('[role=dialog] h2')]",
        )
        .await;
        fixture.console.assert_clean("a long title").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1.4.10: nothing in `boxes` (a JS array) scrolls sideways and `dialog` sits inside the
/// viewport, waited for its placement; a fixed box wider than the viewport widens no scroll.
pub async fn assert_fits_at_320(page: &Page, dialog: &str, boxes: &str) {
    let probe = format!(
        "(() => {{ const d = document.querySelector({}).getBoundingClientRect(); \
         const over = {boxes}.filter(e => e.scrollWidth > e.clientWidth) \
           .map(e => [e.tagName, e.scrollWidth, e.clientWidth]); \
         if (d.left < -0.5 || d.right > innerWidth + 0.5) \
           over.push(['clipped', d.left, d.right, innerWidth]); \
         return over; }})()",
        serde_json::to_string(dialog).unwrap()
    );
    let js = probe.as_str();
    let fits = wait::until("everything inside the 320px viewport", || async move {
        let over: Vec<serde_json::Value> =
            page.evaluate_bounded(js.to_string()).await?.into_value()?;
        Ok(over.is_empty())
    })
    .await;
    if fits.is_err() {
        let over: Vec<serde_json::Value> = page
            .evaluate_bounded(probe.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        panic!("320px: {over:?}");
    }
}

/// Alt+ArrowLeft is Back: neither handle may swallow a browser chord (todo 562).
#[test]
fn modifier_chords_are_left_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open(page).await.unwrap();
        let probe = "(r => [r.x, r.y, r.width, r.height])(document.querySelector('[role=dialog]').getBoundingClientRect())";
        let arrows = [
            keyboard::ARROW_LEFT,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_UP,
            keyboard::ARROW_DOWN,
        ];
        keyboard::tab_to(page, SEPARATOR, TAB_BUDGET).await.unwrap();
        let mut keys = arrows.to_vec();
        keys.extend([keyboard::HOME, keyboard::END]);
        keyboard::assert_chords_ignored(page, &keys, probe)
            .await
            .unwrap();
        keyboard::tab_to(page, HANDLE, TAB_BUDGET).await.unwrap();
        keyboard::assert_chords_ignored(page, &arrows, probe)
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}

const F6: keyboard::Key = keyboard::Key {
    key: "F6",
    code: "F6",
    vk: 117,
    text: None,
};

/// Todo 572: F6 moves focus from the window back to the page, where it came
/// from, and from the page into the topmost window.
#[test]
fn f6_moves_focus_between_the_page_and_the_top_window() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-pair", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        activate(page, "#open-first").await;
        wait_for(
            page,
            "document.querySelector('[role=dialog]')?.contains(document.activeElement)",
            "the first window to take focus",
        )
        .await;
        activate(page, "#open-second").await;
        wait_for(page, "document.querySelectorAll('[role=dialog]').length === 2 && document.querySelectorAll('[role=dialog]')[1].contains(document.activeElement)", "the second window to take focus").await;

        keyboard::press(page, F6).await.unwrap();
        wait_for(
            page,
            "document.activeElement?.id === 'open-second'",
            "F6 to return focus to the second window's opener",
        )
        .await;

        page.evaluate_bounded("document.querySelector('#close-first').focus()")
            .await
            .unwrap();
        keyboard::press(page, F6).await.unwrap();
        wait_for(
            page,
            "document.querySelectorAll('[role=dialog]')[1].contains(document.activeElement)",
            "F6 to move focus into the top window",
        )
        .await;

        keyboard::press(page, F6).await.unwrap();
        wait_for(
            page,
            "document.activeElement?.id === 'close-first'",
            "F6 to return focus where it left the page",
        )
        .await;
        fixture.console.assert_clean("F6").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focus `selector` and press Enter on it.
async fn activate(page: &Page, selector: &str) {
    page.evaluate_bounded(format!(
        "document.querySelector({}).focus()",
        serde_json::to_string(selector).unwrap()
    ))
    .await
    .unwrap();
    keyboard::press(page, keyboard::ENTER).await.unwrap();
}

async fn wait_for(page: &Page, js: &str, what: &str) {
    wait::for_js_true(page, js, what).await.unwrap();
}

// Shared web/native scenarios (822). Geometry is read off the drawn rect.

pub(super) fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

fn rect_line(r: Rect) -> String {
    format!(
        "{} {} {} {}",
        r.x.round(),
        r.y.round(),
        r.width.round(),
        r.height.round()
    )
}

pub(super) async fn opened<D: Driver>(d: &mut D) -> Result<Rect> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Enter to open it", async |d| d.exists(DIALOG).await).await?;
    d.rect(DIALOG).await
}

/// Waits for `report` to read the drawn rect, and returns that rect.
pub(super) async fn reported<D: Driver>(d: &mut D, report: &str, what: &str) -> Result<Rect> {
    let seen = eventually(d, &format!("{what} reported in {report}"), async |d| {
        let drawn = rect_line(d.rect(DIALOG).await?);
        Ok(d.text(report).await? == drawn)
    })
    .await;
    if let Err(e) = seen {
        let (text, drawn) = (d.text(report).await?, rect_line(d.rect(DIALOG).await?));
        bail!("{e}: it reads {text:?}, the window is drawn at {drawn:?}");
    }
    d.rect(DIALOG).await
}

async fn enter_and_escape<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opened(d).await?;
    eventually_focused(d, &format!("{DIALOG}, {DIALOG} *"), "Enter").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to close it", async |d| {
        Ok(!d.exists(DIALOG).await?)
    })
    .await?;
    eventually_focused(d, TRIGGER, "Escape").await
}

async fn f6_round_trips<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opened(d).await?;
    for inside in [DIALOG, "#window-done", HANDLE] {
        d.focus(inside).await?;
        d.press(F6).await?;
        eventually_focused(d, TRIGGER, &format!("F6 from {inside}")).await?;
        d.press(F6).await?;
        eventually_focused(d, DIALOG, "F6 from the page").await?;
    }
    Ok(())
}

async fn arrows_on_the_title_bar<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let before = opened(d).await?;
    d.focus(HANDLE).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually(d, "a step right and down", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.x, before.x + STEP) && near(r.y, before.y + STEP))
    })
    .await?;
    let after = reported(d, MOVE_REPORT, "the move").await?;
    assert!(
        near(after.width, before.width) && near(after.height, before.height),
        "the move resized it: {before:?} to {after:?}"
    );
    Ok(())
}

async fn the_menu_resizes_and_resets<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let first = opened(d).await?;
    d.focus(MENU).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the title-bar menu", async |d| {
        d.exists("[role=menu]").await
    })
    .await?;
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, "[aria-label=Shorter]", "Resize").await?;
    let showing = d.rect(DIALOG).await?;
    d.focus("[aria-label=Wider]").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Wider to add a step", async |d| {
        Ok(near(d.rect(DIALOG).await?.width, showing.width + STEP))
    })
    .await?;
    reported(d, RESIZE_REPORT, "the Wider button").await?;
    d.press(keyboard::ESCAPE).await?;
    eventually(d, "Escape to hide the step buttons", async |d| {
        Ok(!d.exists("[data-slot=steps]").await?)
    })
    .await?;
    assert!(
        d.exists(DIALOG).await?,
        "Escape on the step buttons closed the window"
    );
    eventually_focused(d, MENU, "Escape on the step buttons").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the title-bar menu", async |d| {
        d.exists("[role=menu]").await
    })
    .await?;
    d.press(keyboard::END).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Reset to restore the opened size", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.width, first.width) && near(r.height, first.height))
    })
    .await
}

e2e::scenario!(
    enter_opens_it_with_focus_inside_and_escape_hands_it_back,
    "/floating-window",
    enter_and_escape
);
e2e::scenario!(
    f6_moves_focus_between_the_window_and_the_page,
    "/floating-window",
    f6_round_trips,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);
e2e::scenario!(
    an_arrow_on_the_title_bar_moves_it_by_a_step,
    "/floating-window",
    arrows_on_the_title_bar
);
e2e::scenario!(
    the_title_bar_menu_resizes_and_resets,
    "/floating-window",
    the_menu_resizes_and_resets,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);

/// `menu_parts` reaches the labels of the portaled title-bar menu.
#[test]
fn menu_parts_style_the_portaled_menu() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-sized", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::click(page, TRIGGER).await?;
            wait::for_visible(page, MENU).await?;
            pointer::click(page, MENU).await?;
            wait::for_visible(page, "[role=menu] [data-slot=label]").await?;
            let style: String = page
                .evaluate_bounded(
                    "getComputedStyle(document.querySelector('[role=menu] [data-slot=label]')).fontStyle",
                )
                .await?
                .into_value()?;
            anyhow::ensure!(style == "italic", "menu_parts missed the label: {style}");
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("floating window menu_parts");

        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 1307: tall content scrolls inside the window; its title bar and last button stay reachable.
#[test]
fn a_tall_window_stays_reachable() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let fixture = Fixture::open("/floating-window-tall", viewport)
                .await
                .unwrap();
            let page = &fixture.page;
            pointer::click(page, TRIGGER).await.unwrap();
            wait::for_visible(page, DIALOG).await.unwrap();
            let name = viewport.name();
            crate::modal::assert_reachable(
                page,
                "document.querySelector('#window-done')",
                &format!("the Done button at {name}"),
            )
            .await;
            crate::modal::assert_reachable(
                page,
                "document.querySelector('[role=dialog] [data-slot=title-bar] > [data-slot=close]')",
                &format!("the close button at {name}"),
            )
            .await;
            fixture
                .console
                .assert_clean(&format!("/floating-window-tall at {name}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// `FloatingWindow` whose title bar is deaf to the arrows (a capturing listener): nothing a
/// snapshot or axe sees changes.
#[test]
fn a_floating_window_with_a_deaf_title_bar_never_moves() {
    block_on(async {
        e2e::selftest::must_fail("/floating-window", Some(
                "document.addEventListener('keydown', e => { \
                 if (e.target.closest('[data-slot=handle]')) e.stopImmediatePropagation(); }, true)",
            ), "the window to move by (10.0, 0.0)", async |page| keyboard_move(page).await)
        .await;
    });
}

/// `FloatingWindow` reporting the pre-move rect, the `owed` effect's defect. It skips the
/// value it last wrote: a flag is already cleared by the next microtask and hung the page.
#[test]
fn a_floating_window_reporting_a_stale_rect_fails_the_move_report() {
    block_on(async {
        e2e::selftest::must_fail("/floating-window", Some(
                "(() => { let written = null; \
                 new MutationObserver(() => { const el = document.querySelector('#move-report'); \
                 if (!el) return; const now = el.textContent; if (now === written) return; \
                 const parts = now.trim().split(' '); if (parts.length !== 4) return; \
                 written = [Number(parts[0]) - 10, parts[1], parts[2], parts[3]].join(' '); \
                 el.textContent = written; }) \
                 .observe(document.body, { subtree: true, childList: true, characterData: true }); })()",
            ), "but it reported", async |page| keyboard_move(page).await)
        .await;
    });
}

/// `FloatingWindow` whose corner handle is deaf to Home and End only, so the arrows still
/// resize but the caller's bounds never answer.
#[test]
fn a_floating_window_that_ignores_home_on_its_separator_never_reaches_the_minimum() {
    block_on(async {
        e2e::selftest::must_fail("/floating-window", Some(
                "document.addEventListener('keydown', e => { \
                 if ((e.key === 'Home' || e.key === 'End') && e.target.closest('[role=separator]')) \
                 e.stopImmediatePropagation(); }, true)",
            ), "Home to size the window to 240x120", async |page| keyboard_resize(page).await)
        .await;
    });
}

/// `FloatingWindow`'s 20x20 close button with its 24x24 press box stripped (505, 566) and
/// crammed against the move handle, inside the 24px circle (377).
#[test]
fn a_floating_windows_crammed_title_bar_fails_the_spacing_exception() {
    block_on(async {
        e2e::selftest::must_fail(
            "/floating-window",
            Some(&crate::planted::stylesheet(
                "[data-slot=title-bar] { gap: 0 !important; } \
                 [data-slot=title-bar] button { width: 16px !important; height: 16px !important; } \
                 [data-slot=title-bar] button::before { content: none !important; } \
                 [data-slot=handle] { flex: 0 0 8px !important; min-width: 0 !important; }",
            )),
            "spacing exception fails",
            async |page| {
                keyboard::tab_to(page, TRIGGER, 8).await?;
                keyboard::press(page, keyboard::ENTER).await?;
                wait::for_visible(page, DIALOG).await?;
                target_size::assert_minimum_or_spacing(page, CLOSE).await
            },
        )
        .await;
    });
}

/// `FloatingWindow` with `#ddd` body text: proves the baseline's axe run reaches the
/// portaled window. Non-modal, so no scroll lock and no `UNLOCK` lift.
#[test]
fn a_floating_window_with_faint_text_fails_the_contrast_pass_in_the_open_state() {
    block_on(async {
        e2e::selftest::must_fail(
            "/floating-window",
            Some(&crate::planted::stylesheet(
                "[role=dialog] [data-slot=body] * { color: #dddddd !important; }",
            )),
            "color-contrast",
            async |page| {
                keyboard::tab_to(page, TRIGGER, 8).await?;
                keyboard::press(page, keyboard::ENTER).await?;
                wait::for_visible(page, DIALOG).await?;
                e2e::passes::contrast::assert_clean(page, "[data-fixture-ready]").await
            },
        )
        .await;
    });
}
