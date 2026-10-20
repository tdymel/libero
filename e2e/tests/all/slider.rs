//! `Slider`: the one fixture that exercises the pointer pass.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_text, swipe_scrolls_on_android};
use e2e::passes::target_size::MINIMUM;
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Suite, Viewport, wait};

pub(super) const VALUE_NOW: &str =
    "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";

pub(super) const THUMB: &str = "[role=slider]";

const TRACK: &str = "[data-state~=size-md] > [data-state~=size-md] > div";

/// A missing or unparsable `aria-valuenow` fails, not reads as 0 (todo 1762).
async fn value_now<D: Driver>(d: &mut D) -> Result<f64> {
    let now = d
        .attr(THUMB, "aria-valuenow")
        .await?
        .ok_or_else(|| anyhow::anyhow!("the thumb has no aria-valuenow"))?;
    Ok(now.parse()?)
}

/// The 400px track's centre is 50; a quarter of it further is 75. A touch
/// that began off the thumb neither drags nor taps (1059).
async fn centre_drag<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    ensure!(value_now(d).await? == 0.0, "the slider starts off 0");
    d.drag(TRACK, 100.0, 0.0).await?;
    // A touch on the track only taps (1059), iOS's too (3603).
    if matches!(d.platform(), Platform::Android | Platform::Ios) {
        d.settle().await?;
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

async fn value_reaches<D: Driver>(d: &mut D, what: &str, want: f64) -> Result<()> {
    eventually(d, what, async |d| Ok(value_now(d).await? == want)).await
}

/// The docs promise arrows by one step, PageUp, PageDown and Shift+Arrow by ten.
async fn arrows_and_pages<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(THUMB).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    value_reaches(d, "ArrowRight to step to 1", 1.0).await?;
    d.press(keyboard::ARROW_UP).await?;
    value_reaches(d, "ArrowUp to step to 2", 2.0).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    value_reaches(d, "ArrowLeft to step back to 1", 1.0).await?;
    d.press(keyboard::PAGE_UP).await?;
    value_reaches(d, "PageUp to jump to 11", 11.0).await?;
    d.press(keyboard::PAGE_DOWN).await?;
    value_reaches(d, "PageDown to jump back to 1", 1.0).await?;
    d.press_shift(keyboard::ARROW_RIGHT).await?;
    value_reaches(d, "Shift+ArrowRight to jump to 11", 11.0).await?;
    d.press_shift(keyboard::ARROW_LEFT).await?;
    value_reaches(d, "Shift+ArrowLeft to jump back to 1", 1.0).await
}

e2e::scenario!(
    the_arrows_page_keys_and_shift_arrows_step_the_value,
    "/slider/drag",
    arrows_and_pages
);

const SEGMENT: &str = "#segments [data-slot=segment]";

/// Todo 2136: segments as wide as their spans, 2px apart; the one holding 50
/// filled a quarter and named in the value; End fills the last. A mark on a boundary draws no dot.
async fn segments_fill<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let nth = |n: usize| format!("{SEGMENT}:nth-child({n})");
    let fill = |n: usize| format!("{SEGMENT}:nth-child({n}) > [data-slot=segment-fill]");
    eventually(
        d,
        "three segments, the second a quarter filled",
        async |d| {
            let (first, second, third) = (
                d.rect(&nth(1)).await?,
                d.rect(&nth(2)).await?,
                d.rect(&nth(3)).await?,
            );
            let filled = d.rect(&fill(2)).await?;
            Ok((second.x - (first.x + first.width) - 2.0).abs() < 1.0
                && (third.width / second.width - 0.5).abs() < 0.1
                && (filled.width / second.width - 0.25).abs() < 0.05)
        },
    )
    .await?;
    // Todo 2168: the mark at 40 keeps its caption but no dot, which would cover the gap.
    let (second, dot, caption) = (
        d.rect(&nth(2)).await?,
        d.rect("#segments [data-slot=mark]").await?,
        d.rect("#segments [data-slot=mark-label]").await?,
    );
    ensure!(
        dot.x > second.x + second.width,
        "the first dot sits at {}, not at 100",
        dot.x
    );
    ensure!(
        (caption.x + caption.width / 2.0 - second.x).abs() < 4.0,
        "the 40 caption is at {}, the gap at {}",
        caption.x,
        second.x
    );
    let text = d.attr(THUMB, "aria-valuetext").await?.unwrap_or_default();
    ensure!(text.ends_with("Middle"), "the value reads {text:?}");
    d.focus(THUMB).await?;
    d.press(keyboard::END).await?;
    eventually(d, "End to fill the last segment", async |d| {
        let (third, filled) = (d.rect(&nth(3)).await?, d.rect(&fill(3)).await?);
        Ok(third.width > 0.0 && (filled.width - third.width).abs() < 1.0)
    })
    .await
}

e2e::scenario!(
    a_segmented_track_fills_up_to_the_value,
    "/slider/segments",
    segments_fill
);

e2e::scenario!(
    pressing_the_track_centre_and_dragging_right_follows_the_pointer,
    "/slider/drag",
    centre_drag
);

pub(super) const BUBBLE: &str = "[role=tooltip]:not([hidden])";

e2e::scenario!(
    the_thumb_moves_with_the_value,
    "/slider/drag",
    thumb_follows
);

pub(super) const SCROLL_THUMB: &str = "[role=slider][aria-label=Volume]";
const RANGE_THUMB: &str = "[role=slider]:not([aria-label=Volume])";

/// Todo 1020: on Android a vertical swipe over a slider scrolls the page and
/// leaves the value; a mouse still grabs at once. Sideways drags and taps move it.
async fn a_swipe_scrolls_on_android<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for (thumb, ended, grabbed) in [
        (SCROLL_THUMB, "#slider-end", "0"),
        (RANGE_THUMB, "#range-end", "20-80"),
    ] {
        swipe_scrolls_on_android(d, thumb, ended, grabbed).await?;
        if matches!(d.platform(), Platform::Android | Platform::Ios) {
            d.settle().await?;
            ensure!(
                d.text(ended).await?.is_empty(),
                "a swipe over {thumb} moved it"
            );
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
    a_vertical_swipe_scrolls_the_page_on_android_only_and_a_sideways_drag_or_a_tap_moves_it,
    "/slider/scroll",
    a_swipe_scrolls_on_android
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
    a_release_outside_ends,
    ios: skip("a touch screen has no hover")
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
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
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
        page.evaluate_bounded(SAMPLE_BUBBLE).await.unwrap();
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
            .evaluate_bounded("(() => { window.__sampled = true; return [__offsets.length, Math.max(0, ...__offsets)]; })()")
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
    page.evaluate_bounded(sample_bubble(thumb, check))
        .await
        .unwrap();
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
    page.evaluate_bounded(
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

/// WCAG 2.5.8 on the 16px thumb's 24px hit area (302): hit-tests 11.5px out, 13px as the
/// failing control, then drags from 11px above, outside the drawn box.
#[test]
fn the_thumb_takes_the_pointer_over_24px() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}

/// Whether each corner of a square `offset` out from `centre` hit-tests to
/// the thumb.
async fn corners_on_thumb(fixture: &Fixture, centre: pointer::Point, offset: f64) -> Vec<bool> {
    fixture
        .page
        .evaluate_bounded(format!(
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
            .evaluate_bounded(
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
            .evaluate_bounded(
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
        page.execute_bounded(
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
            .evaluate_bounded(
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
        // Up first, or its going proves nothing.
        wait::for_visible(page, BUBBLE).await.unwrap_or_else(|e| {
            panic!("no value bubble under the pointer after a track press: {e}")
        });
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
        e2e::clock::settle(page).await.unwrap();
        let (now, bubble): (String, bool) = page
            .evaluate_bounded(format!(
                "[{value}, [...document.querySelectorAll('[role=tooltip]')].some(t => t.checkVisibility())]"
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

/// Todo 3191: focus on a disabled slider's thumb lifts the fade its ring would share.
#[test]
fn focus_on_a_disabled_thumb_lifts_the_fade() {
    const OPACITY: &str = "(() => { let o = 1; \
         for (let e = document.querySelector('#locked [role=slider]'); e; e = e.parentElement) \
         o *= Number(getComputedStyle(e).opacity); return o; })()";
    block_on(async {
        let fixture = Fixture::open("/slider/states", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let rest: f64 = page
            .evaluate_bounded(OPACITY)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(rest, 0.5, "the disabled slider's resting fade");
        page.evaluate_bounded("document.querySelector('#locked [role=slider]').focus()")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("{OPACITY} === 0.75"),
            "the focused disabled thumb to fade to 0.75",
        )
        .await
        .unwrap();
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
            let (along, [left, top, width, height]): (Vec<f64>, [f64; 4]) = page
                .evaluate_bounded(ALONG)
                .await
                .unwrap()
                .into_value()
                .unwrap();
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
                .evaluate_bounded(VALUE_NOW)
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
                .evaluate_bounded(ALONG)
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

/// For `#id`: the filled bars, whether they all sit on the minimum's side of
/// the others, and whether the line track's bar is gone.
fn bars_js(id: &str) -> String {
    format!(
        "(() => {{ const root = document.querySelector('#{id}'); \
         const bars = [...root.querySelectorAll('[data-slot=bars] > span')]; \
         const x = bar => bar.getBoundingClientRect().x; \
         const filled = bars.filter(bar => bar.dataset.state === 'filled').map(x); \
         const empty = bars.filter(bar => bar.dataset.state !== 'filled').map(x); \
         const rtl = getComputedStyle(root).direction === 'rtl'; \
         const before = filled.every(a => empty.every(b => rtl ? a > b : a < b)); \
         return [filled.length, before, !root.querySelector('[data-slot=bar]')]; }})()"
    )
}

/// Todo 2004: a bars track fills as many bars as the value covers, from the
/// minimum's side in either direction, and a key moves the fill.
#[test]
fn a_bars_track_fills_from_the_minimum_in_either_direction() {
    block_on(async {
        let fixture = Fixture::open("/slider/bars", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            for id in ["bars", "bars-rtl"] {
                wait::for_visible(page, &format!("#{id} [data-slot=bars] > span")).await?;
                let (filled, before, no_line): (usize, bool, bool) =
                    page.evaluate_bounded(bars_js(id)).await?.into_value()?;
                ensure!(filled == 2, "#{id}: {filled} of 8 bars filled at 25");
                ensure!(before, "#{id}: the filled bars are not on the minimum's side");
                ensure!(no_line, "#{id}: the line track's bar still draws");
            }
            page.evaluate_bounded("document.querySelector('#bars-rtl [role=slider]').focus()")
                .await?;
            keyboard::press(page, keyboard::END).await?;
            wait::for_js_true(
                page,
                "document.querySelectorAll('#bars-rtl [data-slot=bars] > [data-state=filled]').length === 8",
                "End to fill every bar",
            )
            .await?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("the bars track");
        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// A visible bubble that says `text`, as JS.
fn preview_js(text: &str) -> String {
    format!(
        "[...document.querySelectorAll('[role=tooltip]')].some((e) =>
            e.textContent === '{text}' && getComputedStyle(e).visibility === 'visible')"
    )
}

/// Where a mouse over `#id`'s track points at `fraction` of the range, as the drag maps it.
async fn track_point(
    page: &chromiumoxide::Page,
    id: &str,
    fraction: f64,
) -> Result<pointer::Point> {
    Ok(page
        .evaluate_bounded(format!(
            "(() => {{ const t = document.querySelector('#{id} [data-slot=track]').getBoundingClientRect();
             const w = document.querySelector('#{id} [role=slider]').getBoundingClientRect().width;
             return {{ x: t.x + w / 2 + {fraction} * (t.width - w), y: t.y + t.height / 2 }}; }})()"
        ))
        .await?
        .into_value()?)
}

/// Todo 3412: `group_digits` writes the hover bubble's thousands in the formats'.
#[test]
fn group_digits_groups_the_bubble() {
    block_on(async {
        let fixture = Fixture::open("/slider/grouped", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            for id in ["single", "range"] {
                pointer::move_to(page, track_point(page, id, 0.75).await?).await?;
                wait::for_js_true(
                    page,
                    &preview_js("7,500"),
                    &format!("#{id}: the grouped 7,500"),
                )
                .await?;
                pointer::move_to(page, pointer::Point { x: 2.0, y: 2.0 }).await?;
            }
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("the grouped bubble");
        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 3799: a track that resizes under a hovering mouse is measured again for the next move.
#[test]
fn the_hover_preview_follows_a_resized_track() {
    block_on(async {
        let fixture = Fixture::open("/slider/preview-resize", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            pointer::move_to(page, track_point(page, "single", 0.75).await?).await?;
            wait::for_js_true(page, &preview_js("75"), "the preview of 75").await?;
            page.evaluate_bounded("document.querySelector('#box').style.width = '200px'")
                .await?;
            e2e::clock::next_frame(page).await?;
            pointer::move_to(page, track_point(page, "single", 0.5).await?).await?;
            wait::for_js_true(
                page,
                &preview_js("50"),
                "the preview of 50 on the narrowed track",
            )
            .await?;
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("the resized preview");
        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}

/// Todo 2212: `preview_on_hover` shows the value under a hovering mouse on a Slider and a
/// RangeSlider; Escape hides it until the pointer leaves (WCAG 1.4.13).
#[test]
fn preview_on_hover_shows_the_value_and_escape_hides_it() {
    block_on(async {
        let fixture = Fixture::open("/slider/preview", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let outcome = async {
            for id in ["single", "range"] {
                pointer::move_to(page, track_point(page, id, 0.75).await?).await?;
                wait::for_js_true(
                    page,
                    &preview_js("75"),
                    &format!("#{id}: the preview of 75"),
                )
                .await?;
                keyboard::press(page, keyboard::ESCAPE).await?;
                wait::for_js_true(
                    page,
                    &format!("!({})", preview_js("75")),
                    &format!("#{id}: Escape to hide it"),
                )
                .await?;
                // Still over the track: a move shows nothing till the pointer leaves.
                pointer::move_to(page, track_point(page, id, 0.8).await?).await?;
                e2e::clock::settle(page).await?;
                e2e::clock::next_frame(page).await?;
                let back: bool = page
                    .evaluate_bounded(preview_js("80"))
                    .await?
                    .into_value()?;
                ensure!(
                    !back,
                    "#{id}: the preview came back before the pointer left"
                );
                pointer::move_to(page, pointer::Point { x: 2.0, y: 2.0 }).await?;
                pointer::move_to(page, track_point(page, id, 0.8).await?).await?;
                wait::for_js_true(
                    page,
                    &preview_js("80"),
                    &format!("#{id}: the preview after leaving"),
                )
                .await?;
                pointer::move_to(page, pointer::Point { x: 2.0, y: 2.0 }).await?;
            }
            Ok::<_, anyhow::Error>(())
        }
        .await;
        let console = fixture.console.assert_clean("the hover preview");
        fixture.close().await.unwrap();
        outcome.unwrap();
        console.unwrap();
    });
}
