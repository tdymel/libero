//! `Slider`: the one fixture that exercises the pointer pass.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_text, linger};
use e2e::passes::target_size::MINIMUM;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

use crate::settle;

const VALUE_NOW: &str = "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";

const THUMB: &str = "[role=slider]";

const TRACK: &str = "[data-state~=size-md] > [data-state~=size-md] > div";

async fn value_now<D: Driver>(d: &mut D) -> Result<f64> {
    Ok(d.attr(THUMB, "aria-valuenow")
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or_default())
}

/// The 400px track's centre is 50; a quarter of it further is 75. A touch
/// that began off the thumb neither drags nor taps (1059).
async fn centre_drag<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(value_now(d).await? == 0.0, "the slider starts off 0");
    d.drag(TRACK, 100.0, 0.0).await?;
    if d.platform() == Platform::Android {
        linger(d, 8).await;
        let moved = value_now(d).await?;
        ensure!(moved == 0.0, "a swipe from the track moved it to {moved}");
        return Ok(());
    }
    eventually(d, "the drag to reach 75", async |d| {
        Ok((value_now(d).await? - 75.0).abs() < 2.0)
    })
    .await
}

async fn thumb_follows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let at_zero = d.rect(THUMB).await?.x;
    d.focus(THUMB).await?;
    d.press(keyboard::END).await?;
    eventually(d, "End to move the thumb 300px on", async |d| {
        Ok(value_now(d).await? == 100.0 && d.rect(THUMB).await?.x - at_zero > 300.0)
    })
    .await
}

e2e::scenario!(
    pressing_the_track_centre_and_dragging_right_follows_the_pointer,
    "/slider/drag",
    centre_drag
);

const BUBBLE: &str = "[role=tooltip]:not([hidden])";

/// Todo 1058: a touch held on the thumb opens no lingering long-press bubble;
/// its tap's bubble is gone right after the release.
async fn held_thumb<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.long_press(THUMB, 800).await?;
    let released = std::time::Instant::now();
    loop {
        if !d.exists(BUBBLE).await? {
            return Ok(());
        }
        ensure!(
            released.elapsed().as_millis() < 700,
            "the bubble still shows 700 ms after the release"
        );
    }
}

e2e::scenario!(
    a_touch_held_on_the_thumb_leaves_no_bubble_after_the_release,
    "/slider/drag",
    held_thumb,
    native: skip("996: Blitz has no touch input"),
    desktop: skip("1126: no touch input under Xvfb")
);
e2e::scenario!(
    the_thumb_moves_with_the_value,
    "/slider/drag",
    thumb_follows
);

const SCROLL_THUMB: &str = "[role=slider][aria-label=Volume]";
const RANGE_THUMB: &str = "[role=slider]:not([aria-label=Volume])";

/// Todo 1020: on Android a vertical swipe over a slider scrolls the page and
/// leaves the value; a mouse still grabs at once. Sideways drags and taps move it.
async fn a_swipe_scrolls<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (_, height) = d.viewport().await?;
    let touch = d.platform() == Platform::Android;
    for (thumb, ended, grabbed) in [
        (SCROLL_THUMB, "#slider-end", "0"),
        (RANGE_THUMB, "#range-end", "20-80"),
    ] {
        d.drag(thumb, 0.0, -250.0).await?;
        if touch {
            let at = d.rect(thumb).await?;
            ensure!(
                at.y + at.height / 2.0 < height / 2.0 - 100.0,
                "a swipe up over {thumb} did not scroll: its top at {}",
                at.y
            );
            linger(d, 8).await;
            ensure!(
                d.text(ended).await?.is_empty(),
                "a swipe over {thumb} moved it"
            );
        } else {
            eventually_text(d, ended, grabbed, "a vertical mouse drag").await?;
        }
    }

    d.drag(SCROLL_THUMB, 150.0, 0.0).await?;
    eventually(d, "a sideways drag to move it", async |d| {
        let value = d
            .attr(SCROLL_THUMB, "aria-valuenow")
            .await?
            .unwrap_or_default();
        Ok(value != "0" && d.text("#slider-end").await? == value)
    })
    .await?;

    // 100px of the 280px travel on from the thumb, about 36 further.
    let before: f64 = d.text("#slider-end").await?.parse()?;
    let thumb = d.rect(SCROLL_THUMB).await?;
    d.click_at(
        thumb.x + thumb.width / 2.0 + 100.0,
        thumb.y + thumb.height / 2.0,
    )
    .await?;
    eventually(
        d,
        "a tap on the track to jump there and commit",
        async |d| {
            let value = d.attr(SCROLL_THUMB, "aria-valuenow").await?;
            let committed = d.text("#slider-end").await?;
            Ok(value.as_deref() == Some(committed.as_str())
                && committed
                    .parse::<f64>()
                    .is_ok_and(|value| value >= before + 25.0))
        },
    )
    .await
}

e2e::scenario!(
    a_vertical_swipe_scrolls_the_page_and_a_sideways_drag_or_a_tap_moves_it,
    "/slider/scroll",
    a_swipe_scrolls
);

/// Todo 1179: a drag released past the track's end still tracks to the max and
/// ends there; a later hover moves nothing.
async fn a_release_outside_ends<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let max = d
        .attr(SCROLL_THUMB, "aria-valuemax")
        .await?
        .unwrap_or_default();
    d.drag(SCROLL_THUMB, 400.0, 60.0).await?;
    eventually_text(d, "#slider-end", &max, "a release past the end").await?;
    d.hover(RANGE_THUMB).await?;
    d.settle().await?;
    let value = d.attr(SCROLL_THUMB, "aria-valuenow").await?;
    ensure!(
        value.as_deref() == Some(max.as_str()),
        "a hover after the release moved it to {value:?}"
    );
    Ok(())
}

e2e::scenario!(
    a_drag_released_past_the_end_tracks_and_ends,
    "/slider/scroll",
    a_release_outside_ends
);

/// Todo 1200: a 6px drag ending on the thumb commits once and holds; the
/// swallowed click (1179) is only that one, a later tap on the track still jumps.
async fn a_short_drag_on_the_thumb<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.drag(SCROLL_THUMB, 6.0, 0.0).await?;
    eventually(d, "the short drag to commit", async |d| {
        let value = d.attr(SCROLL_THUMB, "aria-valuenow").await?;
        Ok(value.as_deref() == Some(d.text("#slider-end").await?.as_str()))
    })
    .await?;
    let committed = d.text("#slider-end").await?;
    d.settle().await?;
    let value = d.attr(SCROLL_THUMB, "aria-valuenow").await?;
    ensure!(
        value.as_deref() == Some(committed.as_str()),
        "the release's click moved it from {committed} to {value:?}"
    );
    let before: f64 = committed.parse()?;
    let thumb = d.rect(SCROLL_THUMB).await?;
    d.click_at(
        thumb.x + thumb.width / 2.0 + 100.0,
        thumb.y + thumb.height / 2.0,
    )
    .await?;
    eventually(d, "a later tap on the track to jump", async |d| {
        let committed = d.text("#slider-end").await?;
        Ok(committed
            .parse::<f64>()
            .is_ok_and(|value| value >= before + 25.0))
    })
    .await
}

e2e::scenario!(
    a_short_drag_ending_on_the_thumb_holds_and_the_next_tap_lands,
    "/slider/scroll",
    a_short_drag_on_the_thumb
);

/// Todo 483: the label focuses the thumb it names by id.
#[test]
fn a_click_on_the_label_focuses_the_thumb() {
    crate::select::label_click_focuses("/slider", THUMB);
}

/// The generic battery. No `targets()`: the thumb's 24px hit area is a `::before` (302);
/// the snapshot's `tooltip "40"` is described by the thumb (309), checked separately.
#[test]
fn it_meets_the_baseline() {
    Suite::new("slider", "/slider").focusable(THUMB).run();
}

#[test]
fn the_thumb_tracks_a_drag() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/slider", viewport).await.unwrap();

            // Drag right with intermediate moves: a bare press and release never reaches
            // a component that tracks movement.
            let from = pointer::centre_of(&fixture.page, THUMB).await.unwrap();
            let to = pointer::Point {
                x: from.x + 80.0,
                y: from.y,
            };

            // Waited for, not read: the value is written by a dioxus
            // re-render that the drag does not block on.
            wait::for_js_change(&fixture.page, VALUE_NOW, "the slider value to move", || {
                pointer::drag(&fixture.page, from, to, 10)
            })
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "at {}: dragging the thumb did not move it: {e}",
                    viewport.name()
                )
            });
            // The read-out is the text a theme colour can ruin, so it has to be the live value.
            wait::for_js_true(
                &fixture.page,
                &format!(
                    "document.querySelector('#volume-readout').textContent === {VALUE_NOW} + '%'"
                ),
                "the read-out to show the dragged value",
            )
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!("a drag at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Per frame, how far the open bubble's centre is off the thumb's. A pinned
/// bubble is `visibility: hidden` until its one measuring pass (1065).
const SAMPLE_BUBBLE: &str = "(() => { window.__offsets = []; const frame = () => { \
    const thumb = document.querySelector('[role=slider]').getBoundingClientRect(); \
    const open = document.querySelector('[role=tooltip]:not([hidden])'); \
    const bubble = open?.checkVisibility({ visibilityProperty: true }) && open.getBoundingClientRect(); \
    if (bubble) __offsets.push(Math.abs(thumb.x + thumb.width / 2 - bubble.x - bubble.width / 2)); \
    if (!window.__sampled) requestAnimationFrame(frame); }; requestAnimationFrame(frame); })()";

/// Todo 1058: the value bubble sits centred on the thumb in every frame of a
/// drag. The web measured fast enough before too; the lag showed over Android's IPC.
#[test]
fn the_value_bubble_rides_the_thumb_through_a_drag() {
    block_on(async {
        let fixture = Fixture::open("/slider/drag", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.evaluate(SAMPLE_BUBBLE).await.unwrap();
        let from = pointer::centre_of(page, THUMB).await.unwrap();
        let to = pointer::Point {
            x: from.x + 240.0,
            y: from.y,
        };
        wait::for_js_change(page, VALUE_NOW, "the thumb to move", || {
            pointer::drag(page, from, to, 30)
        })
        .await
        .unwrap();
        let (frames, worst): (usize, f64) = page
            .evaluate("(() => { window.__sampled = true; return [__offsets.length, Math.max(0, ...__offsets)]; })()")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            frames > 3,
            "the bubble showed in {frames} frames of the drag"
        );
        assert!(
            worst < 1.0,
            "the bubble trailed the thumb by up to {worst}px"
        );
        fixture.console.assert_clean("a drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Per frame of a drag on `thumb`, whether `check` failed for its shown bubble
/// (`b`, the bubble's rect; `t`, the thumb's; `el`, the bubble).
fn sample_bubble(thumb: &str, check: &str) -> String {
    format!(
        "(() => {{ window.__shown = 0; window.__failed = []; const frame = () => {{ \
        const el = document.querySelector('[role=tooltip]:not([hidden])'); \
        if (el?.checkVisibility({{ visibilityProperty: true }})) {{ \
            const b = el.getBoundingClientRect(); \
            const t = document.querySelector('{thumb}').getBoundingClientRect(); \
            __shown += 1; if (!({check})) __failed.push([b.top, b.bottom, t.top, t.bottom]); }} \
        if (!window.__sampled) requestAnimationFrame(frame); }}; requestAnimationFrame(frame); }})()"
    )
}

/// Drags `thumb` 120px with `sample_bubble` running; the frames shown and failed.
async fn drag_sampled(page: &chromiumoxide::Page, thumb: &str, check: &str) -> (usize, String) {
    page.evaluate(sample_bubble(thumb, check)).await.unwrap();
    let from = pointer::centre_of(page, thumb).await.unwrap();
    let to = pointer::Point {
        x: from.x + 120.0,
        y: from.y,
    };
    let value = format!("document.querySelector('{thumb}').getAttribute('aria-valuenow')");
    wait::for_js_change(page, &value, "the thumb to move", || {
        pointer::drag(page, from, to, 20)
    })
    .await
    .unwrap();
    page.evaluate(
        "(() => { window.__sampled = true; return [__shown, JSON.stringify(__failed)]; })()",
    )
    .await
    .unwrap()
    .into_value()
    .unwrap()
}

/// Todo 1065: the drag bubble is portaled, so an `overflow: hidden` box its
/// slider's own height does not clip it.
#[test]
fn an_overflow_box_does_not_clip_the_drag_bubble() {
    block_on(async {
        let fixture = Fixture::open("/slider/edges", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let check = "!document.querySelector('#clip-box').contains(el) \
            && b.bottom <= document.querySelector('#clip-box').getBoundingClientRect().top + 0.5";
        let (shown, failed) = drag_sampled(page, "[aria-label=Clipped]", check).await;
        assert!(shown > 3, "the bubble showed in {shown} frames of the drag");
        assert_eq!(failed, "[]", "frames with the bubble inside the box [b, t]");
        fixture.console.assert_clean("a drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1065: at the viewport's top edge the drag bubble flips below the thumb.
#[test]
fn the_drag_bubble_flips_below_at_the_top_edge() {
    block_on(async {
        let fixture = Fixture::open("/slider/edges", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let (shown, failed) = drag_sampled(page, "[aria-label=Edge]", "b.top >= t.bottom").await;
        assert!(shown > 3, "the bubble showed in {shown} frames of the drag");
        assert_eq!(
            failed, "[]",
            "frames with the bubble not below the thumb [b, t]"
        );
        fixture.console.assert_clean("a drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1059 on web touch: a swipe from the track neither drags nor taps, a
/// still tap there jumps, and a sideways drag from the thumb moves it.
#[test]
fn a_touch_drags_only_from_the_thumb() {
    block_on(async {
        let fixture = Fixture::open("/slider/scroll", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let value =
            format!("document.querySelector('{SCROLL_THUMB}').getAttribute('aria-valuenow')");
        let thumb = pointer::centre_of(page, SCROLL_THUMB).await.unwrap();
        let track = pointer::Point {
            x: thumb.x + 150.0,
            y: thumb.y,
        };
        let aside = pointer::Point {
            x: track.x + 60.0,
            y: track.y,
        };

        pointer::touch_drag(page, track, aside, 6).await.unwrap();
        settle::painted(page).await.unwrap();
        let (now, ended): (String, String) = page
            .evaluate(format!(
                "[{value}, document.querySelector('#slider-end').textContent]"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            (now.as_str(), ended.as_str()),
            ("0", ""),
            "a swipe from the track moved it"
        );

        pointer::touch_drag(page, track, track, 0).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector('#slider-end').textContent === {value} && Number({value}) > 40"
            ),
            "a still tap on the track to jump there and commit",
        )
        .await
        .unwrap();

        let thumb = pointer::centre_of(page, SCROLL_THUMB).await.unwrap();
        let back = pointer::Point {
            x: thumb.x - 80.0,
            y: thumb.y,
        };
        wait::for_js_change(page, &value, "a drag from the thumb to move it", || {
            pointer::touch_drag(page, thumb, back, 8)
        })
        .await
        .unwrap();
        fixture.console.assert_clean("touches").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The keyboard half of the same contract, which a drag cannot prove.
#[test]
fn the_thumb_moves_with_the_arrow_keys() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        keyboard::tab_to(&fixture.page, THUMB, 10).await.unwrap();

        wait::for_js_change(
            &fixture.page,
            VALUE_NOW,
            "ArrowRight to move the thumb",
            || keyboard::press(&fixture.page, keyboard::ARROW_RIGHT),
        )
        .await
        .unwrap();

        keyboard::press(&fixture.page, keyboard::HOME)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!("{VALUE_NOW} === '0'"),
            "Home to reach the minimum",
        )
        .await
        .unwrap_or_else(|e| panic!("Home should jump to the minimum: {e}"));

        keyboard::press(&fixture.page, keyboard::END).await.unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!("{VALUE_NOW} === '100'"),
            "End to reach the maximum",
        )
        .await
        .unwrap_or_else(|e| panic!("End should jump to the maximum: {e}"));

        fixture.close().await.unwrap();
    });
}

/// WCAG 2.5.8 on the 16px thumb's 24px hit area (302): hit-tests 11.5px out, 13px as the
/// failing control, then drags from 11px above, outside the drawn box.
#[test]
fn the_thumb_takes_the_pointer_over_24px() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/slider", viewport).await.unwrap();
            let centre = pointer::centre_of(&fixture.page, THUMB).await.unwrap();

            assert_eq!(
                corners_on_thumb(&fixture, centre, MINIMUM / 2.0 - 0.5).await,
                [true; 4],
                "at {}: a corner of the 24px hit area missed the thumb",
                viewport.name()
            );
            assert_eq!(
                corners_on_thumb(&fixture, centre, MINIMUM / 2.0 + 1.0).await,
                [false; 4],
                "at {}: the thumb took the pointer outside its 24px hit area",
                viewport.name()
            );

            let from = pointer::Point {
                x: centre.x,
                y: centre.y - 11.0,
            };
            let to = pointer::Point {
                x: from.x + 80.0,
                y: from.y,
            };
            wait::for_js_change(
                &fixture.page,
                VALUE_NOW,
                "a drag from the hit area to move the thumb",
                || pointer::drag(&fixture.page, from, to, 10),
            )
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "at {}: a drag from 11px above the thumb's centre did not move it: {e}",
                    viewport.name()
                )
            });
            fixture.close().await.unwrap();
        }
    });
}

/// Whether each corner of a square `offset` out from `centre` hit-tests to
/// the thumb.
async fn corners_on_thumb(fixture: &Fixture, centre: pointer::Point, offset: f64) -> Vec<bool> {
    fixture
        .page
        .evaluate(format!(
            "[[-1,-1],[1,-1],[-1,1],[1,1]].map(([dx, dy]) => {{ \
             const el = document.elementFromPoint({x} + dx * {offset}, {y} + dy * {offset}); \
             return !!el && !!el.closest('[role=slider]'); }})",
            x = centre.x,
            y = centre.y,
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// The thumb is centred on its value by margins, not a `transform` (Blitz's client rect
/// ignores that), on the half-thumb-inset travel.
#[test]
fn the_thumb_is_centred_on_its_value() {
    block_on(async {
        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let [dx, dy]: [f64; 2] = fixture
            .page
            .evaluate(
                "(() => { \
                 const thumb = document.querySelector('[role=slider]'); \
                 const t = thumb.getBoundingClientRect(); \
                 let track = thumb.parentElement; \
                 while (track.getBoundingClientRect().height >= t.height) track = track.parentElement; \
                 const k = track.getBoundingClientRect(); \
                 const [now, min, max] = ['now', 'min', 'max'] \
                   .map((n) => Number(thumb.getAttribute('aria-value' + n))); \
                 const at = (now - min) / (max - min); \
                 const x = k.left + t.width / 2 + at * (k.width - t.width); \
                 return [t.left + t.width / 2 - x, t.top + t.height / 2 - (k.top + k.height / 2)]; \
                 })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            dx.abs() < 0.5 && dy.abs() < 0.5,
            "the thumb is off its value by ({dx}, {dy})"
        );
        fixture.close().await.unwrap();
    });
}

/// The bubble is a `role="tooltip"`, and a tooltip only means something to
/// assistive technology through the element that points at it.
#[test]
fn the_thumb_is_described_by_its_value_bubble() {
    block_on(async {
        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let description = e2e::ax::description(&fixture.page, THUMB).await.unwrap();
        assert_eq!(description, "40", "the thumb's computed description");
        fixture.close().await.unwrap();
    });
}

/// The docs' switches through axe and the snapshot: required and discrete,
/// formatted, read-only and disabled.
#[test]
fn the_states_meet_the_baseline() {
    Suite::new("slider_states", "/slider/states")
        .focusable("#quality [role=slider]")
        .run();
}

/// Todo 532: ARIA 1.2 allows no `aria-required` on `role=slider`, and
/// Chromium drops it from the AX tree anyway; the label keeps its asterisk.
#[test]
fn a_required_thumb_carries_no_aria_required() {
    block_on(async {
        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_selector(page, "#quality [role=slider]")
            .await
            .unwrap();
        let (required, asterisk): (Option<String>, bool) = page
            .evaluate(
                "(() => { const t = document.querySelector('#quality [role=slider]'); \
                 const label = document.getElementById(t.getAttribute('aria-labelledby')); \
                 return [t.getAttribute('aria-required'), label.textContent.includes('*')]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            (required.as_deref(), asterisk),
            (None, true),
            "the thumb's aria-required, the label's asterisk"
        );
        fixture.close().await.unwrap();
    });
}

/// APG's slider keys beyond the arrows: Page keys and Shift+arrow move ten
/// steps.
#[test]
fn page_keys_and_shift_move_a_big_step() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, THUMB, 10).await.unwrap();
        for (key, shift, expected) in [
            (keyboard::PAGE_UP, false, "50"),
            (keyboard::PAGE_DOWN, false, "40"),
            (keyboard::ARROW_RIGHT, true, "50"),
            (keyboard::ARROW_LEFT, true, "40"),
        ] {
            match shift {
                true => keyboard::press_shift(page, key).await.unwrap(),
                false => keyboard::press(page, key).await.unwrap(),
            }
            wait::for_js_true(
                page,
                &format!("{VALUE_NOW} === '{expected}'"),
                &format!("{} (shift {shift}) to reach {expected}", key.key),
            )
            .await
            .unwrap();
        }
        fixture.close().await.unwrap();
    });
}

/// Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: the thumb must not
/// swallow a browser chord (todo 562).
#[test]
fn modifier_chords_are_left_to_the_browser() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, THUMB, 10).await.unwrap();
        keyboard::assert_chords_ignored(
            page,
            &[
                keyboard::ARROW_LEFT,
                keyboard::ARROW_RIGHT,
                keyboard::ARROW_UP,
                keyboard::ARROW_DOWN,
                keyboard::PAGE_UP,
                keyboard::PAGE_DOWN,
                keyboard::HOME,
                keyboard::END,
            ],
            VALUE_NOW,
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A discrete thumb reports its option's name, and the keys walk the options.
#[test]
fn a_discrete_thumb_reads_its_option() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let thumb = "#quality [role=slider]";
        let text = format!("document.querySelector('{thumb}').getAttribute('aria-valuetext')");
        wait::for_js_true(page, &format!("{text} === 'Medium'"), "the option's name")
            .await
            .unwrap();
        keyboard::tab_to(page, thumb, 10).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{text} === 'High'"),
            "ArrowRight to name High",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// `step: 0` is continuous, and the keys moved it by `0 * step`: nothing.
#[test]
fn a_continuous_step_still_moves_with_the_keys() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let thumb = "#gain [role=slider]";
        keyboard::tab_to(page, thumb, 10).await.unwrap();
        wait::for_js_change(
            page,
            &format!("document.querySelector('{thumb}').getAttribute('aria-valuenow')"),
            "ArrowRight to move a step-0 thumb",
            || keyboard::press(page, keyboard::ARROW_RIGHT),
        )
        .await
        .unwrap_or_else(|e| panic!("ArrowRight left a `step: 0` slider where it was: {e}"));
        fixture.close().await.unwrap();
    });
}

/// Tab reaches a read-only thumb and skips a disabled one; no key moves the
/// read-only one.
#[test]
fn tab_reaches_readonly_and_skips_disabled() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#fixed [role=slider]", 10)
            .await
            .unwrap();
        keyboard::press(page, keyboard::END).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let state: Vec<String> = page
            .evaluate(
                "[document.querySelector('#fixed [role=slider]').getAttribute('aria-valuenow'), \
                 document.activeElement.closest('#locked') ? 'locked' : 'elsewhere']",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            state,
            ["30", "elsewhere"],
            "[read-only value, focus after Tab]"
        );
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.13: the value bubble a keyboard focus opens goes on Escape, and
/// the focus and value stay.
#[test]
fn escape_hides_the_focus_bubble() {
    block_on(async {
        use e2e::passes::keyboard;

        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, THUMB, 10).await.unwrap();
        wait::for_visible(page, "[role=tooltip]").await.unwrap();
        keyboard::press(page, keyboard::ESCAPE).await.unwrap();
        wait::for_hidden(page, "[role=tooltip]")
            .await
            .unwrap_or_else(|e| panic!("Escape left the value bubble open: {e}"));
        let kept: bool = page
            .evaluate(format!(
                "document.activeElement === document.querySelector('{THUMB}') && {VALUE_NOW} === '40'"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(kept, "Escape moved the focus or the value");
        fixture.close().await.unwrap();
    });
}

/// Forced colours paint every background `Canvas`: the track, the filled bar
/// and the marks vanished, leaving a lone thumb (todo 506).
#[test]
fn track_bar_and_marks_show_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(
            SetEmulatedMediaParams::builder()
                .features(vec![MediaFeature::new("forced-colors", "active")])
                .build(),
        )
        .await
        .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        // `#quality` sits on Medium: bar, one filled mark on it, one mark past it.
        let bare: Vec<String> = page
            .evaluate(
                "(() => {
                    const bg = el => getComputedStyle(el).backgroundColor;
                    const outlined = el => {
                        const s = getComputedStyle(el);
                        return s.outlineStyle !== 'none' && parseFloat(s.outlineWidth) >= 1
                            && s.outlineColor !== 'rgba(0, 0, 0, 0)';
                    };
                    const thumb = document.querySelector('#quality [role=slider]');
                    let track = thumb.parentElement;
                    while (track.getBoundingClientRect().height >= thumb.getBoundingClientRect().height)
                        track = track.parentElement;
                    const [bar, ...rest] = track.children;
                    const marks = rest.filter(el => !el.textContent && el.getBoundingClientRect().width < 10);
                    const filled = marks.find(el => el.dataset.state === 'filled');
                    const open = marks.find(el => el.dataset.state !== 'filled');
                    return [
                        ['track', outlined(track) || bg(track) !== bg(document.body)],
                        ['bar', bg(bar) !== bg(track)],
                        ['filled mark', bg(filled) !== bg(bar)],
                        ['open mark', bg(open) !== bg(track)],
                    ].filter(([, shows]) => !shows).map(([name]) => name);
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(bare.is_empty(), "invisible in forced colours: {bare:?}");
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.11: the unfilled track shows the range's extent, and an open mark
/// on it is a ring around a hole, each at 3:1 (todo 512).
#[test]
fn the_track_and_an_open_mark_part_at_3_to_1() {
    crate::boundary::assert_boundaries(
        "/slider/states",
        "const thumb = document.querySelector('#quality [role=slider]');
         let track = thumb.parentElement;
         while (track.getBoundingClientRect().height >= thumb.getBoundingClientRect().height)
             track = track.parentElement;
         const open = [...track.children].find(el => !el.textContent
             && el.getBoundingClientRect().width < 10 && el.dataset.state !== 'filled');
         const bg = el => CSS(el, 'backgroundColor');
         return [
             ['track on the page', RATIO(bg(track), PAGE(track))],
             ['open mark ring on its hole', RATIO(CSS(open, 'borderTopColor'), bg(open))],
             ['open mark ring on the page', RATIO(CSS(open, 'borderTopColor'), PAGE(track))],
         ];",
    );
}

/// A track press focuses the thumb as the pointer's focus: the value bubble goes once the
/// pointer leaves.
#[test]
fn a_track_press_does_not_pin_the_value_bubble() {
    block_on(async {
        let fixture = Fixture::open("/slider", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let thumb = pointer::centre_of(page, THUMB).await.unwrap();
        let on_track = pointer::Point {
            x: thumb.x + 100.0,
            y: thumb.y,
        };
        wait::for_js_change(page, VALUE_NOW, "a track press to move the thumb", || {
            pointer::drag(page, on_track, on_track, 1)
        })
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector('{THUMB}')"),
            "the track press to focus the thumb",
        )
        .await
        .unwrap();
        pointer::move_to(page, pointer::Point { x: 2.0, y: 2.0 })
            .await
            .unwrap();
        wait::for_hidden(page, "[role=tooltip]")
            .await
            .unwrap_or_else(|e| panic!("the value bubble stayed after a track press: {e}"));
        fixture.close().await.unwrap();
    });
}

/// Todo 596: a disabled slider takes the pointer and shows `not-allowed` over
/// its thumb too, but opens no value bubble and a drag moves nothing.
#[test]
fn a_disabled_slider_shows_not_allowed_and_ignores_the_pointer() {
    block_on(async {
        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let thumb = "#locked [role=slider]";
        crate::action_icon::assert_disabled_look(page, thumb).await;

        let value = format!("document.querySelector('{thumb}').getAttribute('aria-valuenow')");
        let from = pointer::centre_of(page, thumb).await.unwrap();
        let to = pointer::Point {
            x: from.x - 80.0,
            y: from.y,
        };
        pointer::drag(page, from, to, 10).await.unwrap();
        pointer::move_to(page, from).await.unwrap();
        let (now, bubble): (String, bool) = page
            .evaluate(format!(
                "new Promise(r => setTimeout(() => r([{value}, \
                 [...document.querySelectorAll('[role=tooltip]')].some(t => t.checkVisibility())]), 600))"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            (now.as_str(), bubble),
            ("70", false),
            "[value after a drag, a bubble open]"
        );
        fixture.close().await.unwrap();
    });
}

/// Where each thumb sits along its track (0 left edge, 1 right edge), and the
/// track's rect `[left, top, width, height]`.
const ALONG: &str = "(() => { const thumbs = [...document.querySelectorAll('[role=slider]')]; \
     let track = thumbs[0]; while (track.getBoundingClientRect().width < 200) track = track.parentElement; \
     const t = track.getBoundingClientRect(); \
     return [thumbs.map(th => { const r = th.getBoundingClientRect(); \
       return (r.left + r.width / 2 - t.left) / t.width; }), [t.left, t.top, t.width, t.height]]; })()";

/// Todo 710: under RTL the minimum is at the right, as on a native range; ArrowLeft raises
/// the value.
#[test]
fn under_rtl_the_track_runs_right_to_left() {
    use e2e::passes::keyboard;
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let rtl = dir == "rtl";
            let fixture = crate::rtl_keys::open_in("/slider", dir).await;
            let page = &fixture.page;
            let (along, [left, top, width, height]): (Vec<f64>, [f64; 4]) =
                page.evaluate(ALONG).await.unwrap().into_value().unwrap();
            // The value is 40.
            let drawn = if rtl { 1.0 - along[0] } else { along[0] };
            assert!(
                (drawn - 0.4).abs() < 0.05,
                "{dir}: thumb drawn at {along:?}"
            );

            let near_left = pointer::Point {
                x: left + width * 0.05,
                y: top + height / 2.0,
            };
            let beside = pointer::Point {
                x: near_left.x + 1.0,
                y: near_left.y,
            };
            pointer::drag(page, near_left, beside, 2).await.unwrap();
            let pressed = if rtl { "> 85" } else { "< 15" };
            wait::for_js_true(
                page,
                &format!("Number({VALUE_NOW}) {pressed}"),
                &format!("{dir}: a press near the left edge"),
            )
            .await
            .unwrap();

            let before: String = page
                .evaluate(VALUE_NOW)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            let before: f64 = before.parse().unwrap();
            keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
            let moved = if rtl { ">" } else { "<" };
            wait::for_js_true(
                page,
                &format!("Number({VALUE_NOW}) {moved} {before}"),
                &format!("{dir}: ArrowLeft"),
            )
            .await
            .unwrap();
            fixture.console.assert_clean(dir).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Todo 710, `RangeSlider`: the minimum thumb sits on the right under RTL.
#[test]
fn under_rtl_a_range_puts_its_minimum_on_the_right() {
    block_on(async {
        for dir in ["ltr", "rtl"] {
            let fixture = crate::rtl_keys::open_in("/range-slider", dir).await;
            let (along, _): (Vec<f64>, [f64; 4]) = fixture
                .page
                .evaluate(ALONG)
                .await
                .unwrap()
                .into_value()
                .unwrap();
            // 20 and 80.
            let want = if dir == "rtl" { [0.8, 0.2] } else { [0.2, 0.8] };
            for (got, want) in along.iter().zip(want) {
                assert!((got - want).abs() < 0.05, "{dir}: thumbs at {along:?}");
            }
            fixture.close().await.unwrap();
        }
    });
}
