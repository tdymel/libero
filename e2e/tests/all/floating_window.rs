//! `FloatingWindow` (379): the non-modal overlay archetype, plus keyboard move/resize whose
//! `onmove`/`onresize` reports must match the drawn rect. Geometry is desktop only.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, Rect, eventually, eventually_focused};
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::focus, passes::keyboard, passes::pointer, wait};

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
const MIN: (f64, f64) = (240.0, 120.0);
const MAX: (f64, f64) = (480.0, 360.0);

/// The theme's `move_step` and `resize_step`.
const STEP: f64 = 10.0;

/// Enough to cross the trigger and the two readouts, or the window's own
/// handle, close button, body button and separator.
const TAB_BUDGET: usize = 8;

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
        .targets_spaced(SEPARATOR)
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
        }
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

/// The separator resizes by a step, and Home and End hand the window numbers
/// far outside the caller's bounds so that what comes back is the clamp.
#[test]
fn the_separator_resizes_the_window_and_clamps_to_the_callers_bounds() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        keyboard_resize(&fixture.page).await.unwrap();
        fixture.console.assert_clean("a keyboard resize").unwrap();
        fixture.close().await.unwrap();
    });
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

/// Todo 796: under RTL the grip is the bottom-left corner, ArrowLeft widens a
/// moved window, and its right edge stays where it was.
#[test]
fn under_rtl_the_grip_resizes_from_the_bottom_left() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        open(page).await.unwrap();
        let (window, grip) = (
            rect(page, DIALOG).await.unwrap(),
            rect(page, SEPARATOR).await.unwrap(),
        );
        assert!(
            (grip.0 - window.0).abs() <= 2.0,
            "the grip {grip:?} is not at the window's left {window:?}"
        );

        keyboard::tab_to(page, HANDLE, TAB_BUDGET).await.unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        let moved = wait_for_move(page, window, (-STEP, 0.0)).await.unwrap();

        keyboard::tab_to(page, SEPARATOR, TAB_BUDGET).await.unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        let wider = wait_for_size(page, (moved.2 + STEP, moved.3), "a step left under RTL")
            .await
            .unwrap();
        assert!(
            (wider.0 + wider.2 - (moved.0 + moved.2)).abs() <= 1.0,
            "the right edge moved: {moved:?} -> {wider:?}"
        );
        fixture.console.assert_clean("an RTL resize").unwrap();
        fixture.close().await.unwrap();
    });
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
            let text: Option<String> = page.evaluate(js.to_string()).await?.into_value()?;
            Ok(text.as_deref() == Some(wanted))
        },
    )
    .await;
    if settled.is_err() {
        let text: Option<String> = page.evaluate(read.clone()).await?.into_value()?;
        bail!(
            "after {what} the window is drawn at {expected:?} (value, text, min, max), but the \
             separator says {:?}",
            text.unwrap_or_default()
        );
    }
    Ok(())
}

/// `use_drag` cancels the pointerdown and its focus, so the hook focuses the pressed
/// handle itself (439c). Focus starts on the window root.
#[test]
fn a_drag_leaves_its_handle_focused() {
    block_on(async {
        let fixture = Fixture::open("/floating-window", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open(page).await.unwrap();

        for (handle, what) in [(SEPARATOR, "the resize grip"), (HANDLE, "the title bar")] {
            let from = pointer::centre_of(page, handle).await.unwrap();
            let to = pointer::Point {
                x: from.x + 20.0,
                y: from.y + 20.0,
            };
            pointer::drag(page, from, to, 5).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.activeElement === document.querySelector({})",
                    serde_json::to_string(handle).unwrap()
                ),
                &format!("{what} to hold focus after a drag"),
            )
            .await
            .unwrap();
        }

        fixture.console.assert_clean("a pointer drag").unwrap();
        fixture.close().await.unwrap();
    });
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
        .evaluate("!!document.querySelector('[data-slot=steps]')")
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

/// Open the title-bar menu and click the item named `label`.
async fn choose(page: &Page, label: &str) -> Result<()> {
    pointer::click(page, MENU).await?;
    wait::for_visible(page, "[role=menu]").await?;
    let at: Option<(f64, f64)> = page
        .evaluate(format!(
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
async fn open(page: &Page) -> Result<()> {
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
async fn rect(page: &Page, selector: &str) -> Result<(f64, f64, f64, f64)> {
    let found: Option<(f64, f64, f64, f64)> = page
        .evaluate(format!(
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
async fn wait_for_move(
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
async fn wait_for_size(
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
            let text: Option<String> = page.evaluate(js.to_string()).await?.into_value()?;
            Ok(text.as_deref() == Some(wanted))
        },
    )
    .await;
    if settled.is_err() {
        let text: Option<String> = page.evaluate(read.clone()).await?.into_value()?;
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
            .evaluate("[...document.querySelectorAll('[role=dialog]')].map(d => +getComputedStyle(d.parentElement).zIndex)")
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
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let active: String = page
            .evaluate("document.activeElement?.id ?? ''")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(active, "close-first", "a close from the page moved focus");
        fixture.console.assert_clean("two windows").unwrap();
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
        page.execute(SetDeviceMetricsOverrideParams::new(320, 640, 1.0, true))
            .await
            .unwrap();
        activate(page, "#open-first").await;
        wait::for_visible(page, DIALOG).await.unwrap();
        let overflows: Vec<serde_json::Value> = page
            .evaluate(
                "[document.documentElement, document.querySelector('[role=dialog]'), document.querySelector('[role=dialog] h2')] \
                 .filter(e => e.scrollWidth > e.clientWidth).map(e => [e.tagName, e.scrollWidth, e.clientWidth])",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(overflows.is_empty(), "320px: {overflows:?}");
        fixture.console.assert_clean("a long title").unwrap();
        fixture.close().await.unwrap();
    });
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

        page.evaluate("document.querySelector('#close-first').focus()")
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
    page.evaluate(format!(
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

fn near(a: f64, b: f64) -> bool {
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

async fn opened<D: Driver>(d: &mut D) -> Result<Rect> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Enter to open it", async |d| d.exists(DIALOG).await).await?;
    d.rect(DIALOG).await
}

/// Waits for `report` to read the drawn rect, and returns that rect.
async fn reported<D: Driver>(d: &mut D, report: &str, what: &str) -> Result<Rect> {
    eventually(d, &format!("{what} reported in {report}"), async |d| {
        let drawn = rect_line(d.rect(DIALOG).await?);
        Ok(d.text(report).await? == drawn)
    })
    .await?;
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

async fn a_drag_past_the_edge<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opened(d).await?;
    let (width, _) = d.viewport().await?;
    d.drag(HANDLE, 900.0, 0.0).await?;
    eventually(d, "the window to stay inside the viewport", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(r.x + r.width <= width + 1.0)
    })
    .await
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

async fn the_separator_clamps<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let before = opened(d).await?;
    d.focus(SEPARATOR).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "a step wider", async |d| {
        Ok(near(d.rect(DIALOG).await?.width, before.width + STEP))
    })
    .await?;
    reported(d, RESIZE_REPORT, "the step").await?;
    // A phone's viewport is narrower than MAX: the window caps at it.
    let (vw, vh) = d.viewport().await?;
    let max = (MAX.0.min(vw.floor()), MAX.1.min(vh.floor()));
    for (key, name, (width, height)) in [(keyboard::HOME, "Home", MIN), (keyboard::END, "End", max)]
    {
        d.press(key).await?;
        eventually(d, &format!("{name} to reach {width}x{height}"), async |d| {
            let r = d.rect(DIALOG).await?;
            Ok(near(r.width, width) && near(r.height, height))
        })
        .await?;
        reported(d, RESIZE_REPORT, name).await?;
        assert_eq!(
            d.attr(SEPARATOR, "aria-valuenow").await?,
            Some(width.to_string())
        );
        assert_eq!(
            d.attr(SEPARATOR, "aria-valuetext").await?,
            Some(format!("{width} by {height} pixels"))
        );
    }
    assert_eq!(
        d.attr(SEPARATOR, "aria-valuemin").await?,
        Some(MIN.0.to_string())
    );
    assert_eq!(
        d.attr(SEPARATOR, "aria-valuemax").await?,
        Some(max.0.to_string())
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

async fn a_title_bar_drag<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let before = opened(d).await?;
    // A phone's viewport leaves the window less room than the drag: it stops at the edge.
    let (vw, vh) = d.viewport().await?;
    let x = (before.x + 30.0).min(vw - before.width).max(0.0);
    let y = (before.y + 20.0).min(vh - before.height).max(0.0);
    d.drag(HANDLE, 30.0, 20.0).await?;
    eventually(d, "the drag to move it by (30, 20)", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.x, x) && near(r.y, y))
    })
    .await?;
    reported(d, MOVE_REPORT, "the drag").await?;
    Ok(())
}

async fn drags_leave_the_handle_focused<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opened(d).await?;
    for handle in [SEPARATOR, HANDLE] {
        d.drag(handle, 20.0, 20.0).await?;
        eventually_focused(d, handle, &format!("dragging {handle}")).await?;
    }
    Ok(())
}

/// Todo 922: the caller's `sx` `width`/`height` is the initial size only; the
/// keyboard and the pointer resize past it, and Reset brings it back.
async fn a_resize_beats_the_callers_size<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let first = opened(d).await?;
    assert!(
        near(first.width, 400.0) && near(first.height, 200.0),
        "the caller's sx size is not the initial size: {first:?}"
    );
    d.focus(SEPARATOR).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually(d, "the keyboard to add a step each way", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.width, 400.0 + STEP) && near(r.height, 200.0 + STEP))
    })
    .await?;
    let keyed = reported(d, RESIZE_REPORT, "the keyboard resize").await?;
    // A phone's viewport caps the width below the drag's.
    let (vw, _) = d.viewport().await?;
    d.drag(SEPARATOR, 30.0, 20.0).await?;
    eventually(d, "the pointer to add (30, 20)", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.width, (keyed.width + 30.0).min(vw)) && near(r.height, keyed.height + 20.0))
    })
    .await?;
    reported(d, RESIZE_REPORT, "the pointer resize").await?;
    // The menu's roving keys need element identity on the WebView (958).
    if matches!(d.platform(), Platform::Android | Platform::Desktop) {
        return Ok(());
    }
    d.focus(MENU).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the title-bar menu", async |d| {
        d.exists("[role=menu]").await
    })
    .await?;
    d.press(keyboard::END).await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Reset to restore the caller's size", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.width, 400.0) && near(r.height, 200.0))
    })
    .await
}

e2e::scenario!(
    a_resize_wins_over_the_callers_sx_size,
    "/floating-window-sized",
    a_resize_beats_the_callers_size
);
e2e::scenario!(
    enter_opens_it_with_focus_inside_and_escape_hands_it_back,
    "/floating-window",
    enter_and_escape,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    f6_moves_focus_between_the_window_and_the_page,
    "/floating-window",
    f6_round_trips,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_drag_past_the_edge_keeps_it_in_the_viewport,
    "/floating-window",
    a_drag_past_the_edge
);
e2e::scenario!(
    an_arrow_on_the_title_bar_moves_it_by_a_step,
    "/floating-window",
    arrows_on_the_title_bar
);
e2e::scenario!(
    the_separator_resizes_it_and_clamps_to_the_callers_bounds,
    "/floating-window",
    the_separator_clamps
);
e2e::scenario!(
    the_title_bar_menu_resizes_and_resets,
    "/floating-window",
    the_menu_resizes_and_resets,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    a_title_bar_drag_moves_it_with_the_pointer,
    "/floating-window",
    a_title_bar_drag
);
e2e::scenario!(
    drags_leave_their_handle_focused,
    "/floating-window",
    drags_leave_the_handle_focused,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
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
                .evaluate(
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
        for viewport in Viewport::ALL {
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
        }
    });
}
