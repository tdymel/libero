//! `FloatingWindow`'s resize and drag geometry: the grip, the separator and the drags,
//! each checked against the rect the browser drew.

use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused};
use e2e::passes::keyboard;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport};

use super::floating_window::{
    DIALOG, HANDLE, MAX, MENU, MIN, MOVE_REPORT, RESIZE_REPORT, SEPARATOR, STEP, TAB_BUDGET,
    keyboard_resize, near, open, opened, rect, reported, wait_for_move, wait_for_size,
};

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

/// Todo 2353: the grip's arrows move its corner, so a bottom-end window shrinks
/// from the right and the bottom, not from its anchored edges.
#[test]
fn a_keyboard_resize_moves_the_grips_corner_of_an_anchored_window() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-anchored", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        open(page).await.unwrap();
        keyboard::tab_to(page, SEPARATOR, TAB_BUDGET).await.unwrap();

        let before = rect(page, DIALOG).await.unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        let narrower = wait_for_size(page, (before.2 - STEP, before.3), "a step left")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        let shorter = wait_for_size(page, (narrower.2, before.3 - STEP), "a step up")
            .await
            .unwrap();
        assert!(
            (shorter.0 - before.0).abs() <= 1.0 && (shorter.1 - before.1).abs() <= 1.0,
            "the top-left corner moved: {before:?} -> {shorter:?}"
        );
        fixture.console.assert_clean("an anchored resize").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2631: under RTL the grip is the bottom-left corner, so an anchored window's
/// arrows keep its top-right corner and the right edge stays through a widen.
#[test]
fn under_rtl_a_keyboard_resize_moves_the_grips_corner_of_an_anchored_window() {
    block_on(async {
        let fixture = Fixture::open("/floating-window-anchored", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate_bounded("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
        open(page).await.unwrap();
        keyboard::tab_to(page, SEPARATOR, TAB_BUDGET).await.unwrap();

        let before = rect(page, DIALOG).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        let narrower = wait_for_size(page, (before.2 - STEP, before.3), "a step right")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        let shorter = wait_for_size(page, (narrower.2, before.3 - STEP), "a step up")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        let wider = wait_for_size(page, (before.2, shorter.3), "a step left")
            .await
            .unwrap();
        for (what, drawn) in [
            ("narrower", narrower),
            ("shorter", shorter),
            ("wider", wider),
        ] {
            assert!(
                (drawn.0 + drawn.2 - (before.0 + before.2)).abs() <= 1.0
                    && (drawn.1 - before.1).abs() <= 1.0,
                "the top-right corner moved ({what}): {before:?} -> {drawn:?}"
            );
        }
        fixture
            .console
            .assert_clean("an RTL anchored resize")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 2641: a pinned bottom-end window grows about its anchored corner, so the grip's
/// arrows act as a pointer drag does: toward the anchor shrinks, away widens.
async fn pinned_resize_by_key(rtl: bool) {
    let fixture = Fixture::open("/floating-window-pinned", Viewport::Desktop)
        .await
        .unwrap();
    let page = &fixture.page;
    if rtl {
        page.evaluate_bounded("document.documentElement.dir = 'rtl'")
            .await
            .unwrap();
    }
    open(page).await.unwrap();
    keyboard::tab_to(page, SEPARATOR, TAB_BUDGET).await.unwrap();

    // The anchored corner is the bottom-right, or the bottom-left under RTL.
    let (shrink, widen) = match rtl {
        true => (keyboard::ARROW_LEFT, keyboard::ARROW_RIGHT),
        false => (keyboard::ARROW_RIGHT, keyboard::ARROW_LEFT),
    };
    let before = rect(page, DIALOG).await.unwrap();
    keyboard::press(page, shrink).await.unwrap();
    let narrower = wait_for_size(
        page,
        (before.2 - STEP, before.3),
        "a step toward the anchor",
    )
    .await
    .unwrap();
    keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
    let shorter = wait_for_size(page, (narrower.2, before.3 - STEP), "a step down")
        .await
        .unwrap();
    keyboard::press(page, widen).await.unwrap();
    let wider = wait_for_size(page, (before.2, shorter.3), "a step away from the anchor")
        .await
        .unwrap();
    for (what, drawn) in [
        ("narrower", narrower),
        ("shorter", shorter),
        ("wider", wider),
    ] {
        let corner = |r: (f64, f64, f64, f64)| (if rtl { r.0 } else { r.0 + r.2 }, r.1 + r.3);
        let (anchored, was) = (corner(drawn), corner(before));
        assert!(
            (anchored.0 - was.0).abs() <= 1.0 && (anchored.1 - was.1).abs() <= 1.0,
            "the anchored corner moved ({what}): {before:?} -> {drawn:?}"
        );
    }
    fixture.console.assert_clean("a pinned resize").unwrap();
    fixture.close().await.unwrap();
}

#[test]
fn a_keyboard_resize_of_a_pinned_window_moves_the_grips_corner() {
    block_on(pinned_resize_by_key(false));
}

#[test]
fn under_rtl_a_keyboard_resize_of_a_pinned_window_moves_the_grips_corner() {
    block_on(pinned_resize_by_key(true));
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
        page.evaluate_bounded("document.documentElement.dir = 'rtl'")
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

async fn a_drag_past_the_edge<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let first = opened(d).await?;
    let (vw, vh) = d.viewport().await?;
    // Right, then up-left, then down: each edge the title bar could leave by.
    for (dx, dy, edge) in [
        (900.0, 0.0, "right"),
        (-900.0, -900.0, "top left"),
        (0.0, 900.0, "bottom"),
    ] {
        let before = d.rect(DIALOG).await?;
        d.drag(HANDLE, dx, dy).await?;
        // Moved (not a lost drag) and clamped: window and title bar inside.
        let inside = eventually(d, &format!("a drag past the {edge} edge"), async |d| {
            let r = d.rect(DIALOG).await?;
            let bar = d.rect(HANDLE).await?;
            let moved = !near(r.x, before.x) || !near(r.y, before.y);
            let fits = first.height > vh || r.y + r.height <= vh + 1.0;
            Ok(moved
                && fits
                && r.x >= -1.0
                && r.y >= -1.0
                && r.x + r.width <= vw + 1.0
                && bar.y + bar.height <= vh + 1.0)
        })
        .await;
        if let Err(e) = inside {
            let (r, bar) = (d.rect(DIALOG).await?, d.rect(HANDLE).await?);
            bail!("{e}: the window is at {r:?}, its title bar at {bar:?}, in {vw}x{vh}");
        }
    }
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

/// Todo 3311: a window at the right edge that grows moves left in the render the
/// report reads, so `onresize` reports where it is drawn.
async fn a_resize_at_the_right_edge<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    opened(d).await?;
    let (vw, _) = d.viewport().await?;
    d.drag(HANDLE, 900.0, 0.0).await?;
    eventually(d, "a drag to the right edge", async |d| {
        let r = d.rect(DIALOG).await?;
        Ok(near(r.x + r.width, vw))
    })
    .await?;
    let before = d.rect(DIALOG).await?;
    d.focus(SEPARATOR).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    let after = reported(d, RESIZE_REPORT, "the step at the edge").await?;
    assert!(
        after.width > before.width && after.x + after.width <= vw + 1.0,
        "the step left {before:?} at {after:?} in {vw}"
    );
    Ok(())
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

/// `use_drag` cancels the pointerdown and its focus, so the hook focuses the pressed
/// handle itself (439c).
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

/// Todo 2282: the grip of a window still at its centred placement follows the
/// pointer; growing about the anchor moved it at half the pointer's speed.
async fn a_grip_drag<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let window = opened(d).await?;
    let grip = d.rect(SEPARATOR).await?;
    let (vw, vh) = d.viewport().await?;
    if window.x + window.width + 30.0 > vw || window.y + window.height + 20.0 > vh {
        return Ok(());
    }
    d.drag(SEPARATOR, 30.0, 20.0).await?;
    eventually(d, "the grip to stay under the pointer", async |d| {
        let r = d.rect(SEPARATOR).await?;
        Ok(near(r.x, grip.x + 30.0) && near(r.y, grip.y + 20.0))
    })
    .await?;
    let moved = d.rect(DIALOG).await?;
    if !near(moved.x, window.x) || !near(moved.y, window.y) {
        bail!("the window's top-left moved: {window:?} -> {moved:?}");
    }
    Ok(())
}

e2e::scenario!(
    a_grip_drag_keeps_the_grip_under_the_pointer,
    "/floating-window",
    a_grip_drag
);
e2e::scenario!(
    a_resize_wins_over_the_callers_sx_size,
    "/floating-window-sized",
    a_resize_beats_the_callers_size,
    ios: skip("3416: the phone's 402px viewport leaves the window no room to grow by a key")
);

e2e::scenario!(
    a_drag_past_the_edge_keeps_it_in_the_viewport,
    "/floating-window",
    a_drag_past_the_edge,
    ios: skip("3416: dragged to the top, the title bar sits under the status bar, which takes the next drag")
);

e2e::scenario!(
    the_separator_resizes_it_and_clamps_to_the_callers_bounds,
    "/floating-window",
    the_separator_clamps
);
e2e::scenario!(
    a_resize_at_the_right_edge_reports_where_it_is_drawn,
    "/floating-window",
    a_resize_at_the_right_edge
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
    desktop: skip("958: element identity on the WebView"),
    ios: skip("958: element identity on the WebView")
);
