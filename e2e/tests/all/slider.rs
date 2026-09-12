//! `Slider`, minimally.
//!
//! This fixture exists for one reason: it is the only one that exercises the
//! pointer pass. A template with an unproven pass is not a finished template.

use e2e::browser::block_on;
use e2e::passes::pointer;
use e2e::passes::target_size::MINIMUM;
use e2e::{Fixture, Suite, Viewport, wait};

const VALUE_NOW: &str = "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";

const THUMB: &str = "[role=slider]";

/// Todo 483: the label focuses the thumb it names by id.
#[test]
fn a_click_on_the_label_focuses_the_thumb() {
    crate::select::label_click_focuses("/slider", THUMB);
}

/// The generic battery.
///
/// * **No `targets()`.** The thumb is drawn 16x16, and its 24x24 hit area is a
///   `::before` (todo 302), which a bounding box does not include. That
///   check would report 16 for a thumb that meets 2.5.8, so
///   `the_thumb_takes_the_pointer_over_24px` checks the hit area instead.
///
/// The baseline's `tooltip "40"` is the value bubble, and it is not ownerless:
/// the thumb names it in `aria-describedby` (todo 309). The snapshot prints no
/// descriptions, so `the_thumb_is_described_by_its_value_bubble` checks that.
#[test]
fn it_meets_the_baseline() {
    Suite::new("slider", "/slider").focusable(THUMB).run();
}

#[test]
fn the_thumb_tracks_a_drag() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/slider", viewport).await.unwrap();

            // Drag right along the track. The intermediate moves are the point:
            // a press followed straight by a release never reaches a component
            // that tracks movement.
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

/// WCAG 2.5.8 on the thumb's hit area, not its box: the thumb is drawn 16px
/// and an invisible square 24px wide takes the pointer (todo 302).
///
/// Hit-tests the square's corners, 11.5px out from the thumb's centre on both
/// axes, and 13px out as the control that the check can fail. Then drags from
/// 11px above the centre, which is outside the thumb's drawn box and outside
/// the slider's root, so only the hit area can start that drag.
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

/// The thumb is centred on its value by margins, not a `transform` (Blitz's
/// client rect ignores that): its centre on the track's centre line, at the
/// value's point of the half-thumb-inset travel.
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

/// A press on the track focuses the thumb from code, and that focus is the
/// pointer's: the value bubble goes once the pointer leaves, as after a press
/// on the thumb itself.
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
