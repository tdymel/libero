//! `FloatingWindow`: the overlay archetype, plus the geometry it owns.
//!
//! Three tables said the `Overlay` archetype covered this component while
//! nothing ran against it: `e2e/src/archetypes/mod.rs`,
//! `e2e/src/archetypes/overlay.rs` and `plans/13-e2e-pass/overview.md`
//! (todo 379). It is the half of review 8's B3 that todo 353 dropped when it
//! was filed `Lightbox`-only, and it matters because `fabcf2bc` split thirteen
//! component functions with no browser pass and named `Carousel`, `Lightbox`
//! and `FloatingWindow` as the three that moved the most live state.
//!
//! **A window is non-modal**, so the archetype runs with `traps_focus: false`:
//! it opens, focus moves inside, Escape closes it and focus returns to the
//! trigger, but the page behind it stays tabbable on purpose.
//!
//! Beyond the archetype, what a static review cannot clear is the geometry:
//!
//! * the title bar is the keyboard move handle, and an arrow on it moves the
//!   window by the theme's step;
//! * the corner separator resizes, and Home and End ask for `0x0` and for
//!   `u16::MAX` so the caller's own `min-`/`max-` bounds are what answers;
//! * both report through `onmove`/`onresize`, and those reports are **owed to
//!   an effect**. Reading the rect in the same task as the write reported the
//!   previous one (`codebase/components/floating-window`), so every geometry
//!   check here asserts the report against the rect the browser actually drew.
//!   That is the assertion the effect exists for.
//!
//! * the separator is the only thing that tells a screen-reader user how big
//!   the window is, so after every resize its `aria-valuenow`/`valuetext` are
//!   asserted against the drawn rect and its `aria-valuemin`/`valuemax`
//!   against the caller's bounds in pixels (todo 388). Without them ARIA's
//!   implicit 0..100 clamped every size to `100`, and the value once lagged
//!   the window by two resizes.
//!
//! Desktop only for the geometry. The fixture's window may be 480x360, which
//! does not fit a 390px viewport, so measuring it there would measure the
//! viewport clamp rather than the component. The archetype and the `Suite`
//! baseline run at both.
//!
//! ## Not covered, and why
//!
//! * **The viewport clamp on a move.** `clamped()` is CSS, and reaching it by
//!   keyboard takes forty-odd presses; each one starts an async rect read, so
//!   repeating them faster than the reads land measures the race rather than
//!   the clamp. The resize's Home and End clamp against the caller's bounds in
//!   one press each, and that is the clamp this unit measures.
//! * **Pointer drag geometry.** `use_drag` is covered by its own hook tests,
//!   and the keyboard path is the one that fails silently for a keyboard user.
//!   Only the focus a drag leaves behind is asserted (todo 439c).
//! * **Stacking beyond open order.** `/floating-window-pair` checks that the
//!   newer window stacks on top; raise-on-click is `WindowHost`'s unit test.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::archetypes::Overlay;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::focus, passes::keyboard, passes::pointer, wait};

pub const TRIGGER: &str = "#open-window";
pub const DIALOG: &str = "[role=dialog]";
/// The title bar, which is also the keyboard move handle.
pub const HANDLE: &str = "[role=dialog] [data-window-handle]";
/// The corner grip, which is the resize handle.
pub const SEPARATOR: &str = "[role=dialog] [role=separator]";
/// The close button in the title bar.
pub const CLOSE: &str = "[role=dialog] [data-window-title-bar] button";
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
        // The window's text is portaled beside the fixture. Without this, axe
        // reporting no contrast violation in the open state and axe never
        // having looked at the window read the same (todo 327).
        .contrast_covers(DIALOG)
        // Measured 20x20 at both viewports (an `ActionIcon` at `size: "sm"`,
        // as `Notifications`' close button is). So it is under 2.5.8's 24x24
        // outright and conforms through the **spacing exception** instead: the
        // 24px circle on its centre reaches no other target, because the
        // title bar's move handle is `flex: 1` and its centre is the width of
        // the title away. `targets` was declared here first and went red
        // naming the 20x20, which is how the number above was measured.
        .targets_spaced(CLOSE)
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
    // The handle is focusable only because the window is not pinned, and it is
    // named for the job. `assert_focus_ring` tabs to it itself, from the
    // window's own root, so reaching it asserts both - and it leaves focus
    // there for the arrows below.
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

/// The separator is the only thing that tells a screen-reader user how big
/// the window is: its value is the drawn width, its text the drawn size, and
/// its range the caller's own bounds in pixels (todo 388).
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

/// `use_drag` cancels the pointerdown and with it the browser's focus, so the
/// hook focuses the pressed handle itself (todo 439c). Focus starts on the
/// window root after opening, so neither handle holds it beforehand.
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

/// What the component told the caller has to be what the browser drew.
///
/// This is the assertion the `owed` effect exists for. A report written in the
/// handler rather than after the render carries the rect from *before* the
/// change, which on a 10px step is a 10px lie that nothing else on the page
/// shows.
///
/// The report arrives after the render, so it is waited for rather than read.
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

/// A pixel of slack, so a fractional layout is not a failure. Every step this
/// unit measures is ten pixels or a clamp to a bound, so nothing it asserts
/// can hide inside it.
fn close_to(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0
}

/// Two windows: the newer one stacks on top, Escape closes only the window
/// holding focus and hands focus to its own trigger, and a close from the page
/// leaves focus where the user put it rather than pulling it to the trigger.
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
