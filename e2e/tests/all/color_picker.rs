//! `ColorPicker`'s saturation pad: Left/Right move saturation, Up/Down brightness; a drag
//! moves both and focuses the thumb (406). Starts at `#1c7ed6`: 87% / 84%.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const THUMB: &str = "[role=slider][aria-label=Saturation]";

/// `[saturation, brightness]` in whole percent, from the thumb's own ARIA.
const READING: &str = "(() => { const t = document.querySelector('[role=slider][aria-label=Saturation]'); \
     const b = /brightness (\\d+)%/.exec(t.getAttribute('aria-valuetext')); \
     return `${t.getAttribute('aria-valuenow')},${b && b[1]}`; })()";

#[test]
fn it_meets_the_baseline() {
    Suite::new("color_picker", "/color-picker")
        .focusable(THUMB)
        .focusable("[role=slider][aria-label=Hue]")
        .run();
}

/// The swatch row and the alpha slider (todo 1773). No thumb targets: a 16px thumb
/// sits on a pad or track that takes the press anywhere, 2.5.8's equivalent.
#[test]
fn the_swatches_and_the_alpha_slider_meet_the_baseline() {
    Suite::new("color_picker-swatches", "/color-picker-swatches")
        .focusable("[data-slot=swatch]")
        .targets("[data-slot=swatch]")
        .no_contrast_coverage("three colour swatches and no text")
        .run();
    Suite::new("color_picker-alpha", "/color-picker-alpha")
        .focusable("[role=slider][aria-label=Alpha]")
        .run();
}

/// A move in the same task as the press, before the pad is measured, still lands (1266).
#[test]
fn an_early_move_on_the_pad_still_lands() {
    block_on(async {
        let fixture = Fixture::open("/color-picker", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("!!document.querySelector({THUMB:?})"),
            "the pad",
        )
        .await
        .unwrap();
        page.evaluate(format!(
            "(() => {{ const pad = document.querySelector({THUMB:?}).parentElement;
               const r = pad.getBoundingClientRect();
               const at = (type, x) => pad.dispatchEvent(new PointerEvent(type, {{
                   bubbles: true, pointerId: 1, isPrimary: true, pointerType: 'mouse',
                   button: 0, buttons: type === 'pointerup' ? 0 : 1,
                   clientX: r.left + r.width * x, clientY: r.top + r.height / 2 }}));
               at('pointerdown', 0.1);
               at('pointermove', 0.9); }})()"
        ))
        .await
        .unwrap();
        wait::for_js_true(
            page,
            &format!("{READING} === '90,50'"),
            "the early move to land",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The swatch equal to the value is pressed and carries the only check mark;
/// picking another moves both (todo 553).
#[test]
fn the_picked_swatch_is_pressed_and_checked() {
    const PRESSED: &str = "[...document.querySelectorAll('[data-slot=swatches] > button')]\
         .map(b => b.getAttribute('aria-pressed') + (b.querySelector('svg') ? '+check' : '')).join(',')";
    block_on(async {
        let fixture = Fixture::open("/color-picker-swatches", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            &format!("{PRESSED} === 'true+check,false,false'"),
            "the value's swatch to start pressed",
        )
        .await
        .unwrap();

        keyboard::tab_to(page, "[aria-label='#40c057']", 10)
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!("{PRESSED} === 'false,true+check,false'"),
            "Enter on the green swatch to press it",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("the swatches").unwrap();
        fixture.close().await.unwrap();
    });
}

/// `saturation,brightness` in whole percent, from the thumb's own ARIA.
async fn pad_reading<D: Driver>(d: &mut D) -> Result<String> {
    let now = d.attr(THUMB, "aria-valuenow").await?.unwrap_or_default();
    let text = d.attr(THUMB, "aria-valuetext").await?.unwrap_or_default();
    let brightness = text
        .split("brightness ")
        .nth(1)
        .and_then(|rest| rest.split('%').next())
        .unwrap_or_default()
        .to_string();
    Ok(format!("{now},{brightness}"))
}

async fn reads<D: Driver>(d: &mut D, expected: &str, during: &str) -> Result<()> {
    eventually(d, &format!("{expected} after {during}"), async |d| {
        Ok(pad_reading(d).await? == expected)
    })
    .await
}

/// The APG slider keys: Home/End take the saturation, the thumb's
/// `aria-valuenow`, to its ends; Page Up/Down step the brightness ten.
async fn the_page_keys_move_the_pad<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(THUMB).await?;
    reads(d, "87,84", "the starting colour").await?;
    d.press(keyboard::PAGE_DOWN).await?;
    reads(d, "87,74", "PageDown").await?;
    d.press(keyboard::PAGE_UP).await?;
    reads(d, "87,84", "PageUp").await?;
    d.press(keyboard::END).await?;
    reads(d, "100,84", "End").await?;
    d.press(keyboard::HOME).await?;
    reads(d, "0,84", "Home").await
}

e2e::scenario!(
    home_end_and_the_page_keys_move_the_pad,
    "/color-picker",
    the_page_keys_move_the_pad
);

/// Alt+ArrowLeft is Back: neither the pad nor the hue thumb swallows a
/// browser chord (todo 562).
#[test]
fn modifier_chords_are_left_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/color-picker", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let keys = [
            keyboard::ARROW_LEFT,
            keyboard::ARROW_RIGHT,
            keyboard::ARROW_UP,
            keyboard::ARROW_DOWN,
            keyboard::PAGE_UP,
            keyboard::PAGE_DOWN,
            keyboard::HOME,
            keyboard::END,
        ];
        keyboard::tab_to(page, THUMB, 10).await.unwrap();
        keyboard::assert_chords_ignored(page, &keys, READING)
            .await
            .unwrap();

        let hue = "[role=slider][aria-label=Hue]";
        keyboard::tab_to(page, hue, 10).await.unwrap();
        keyboard::assert_chords_ignored(
            page,
            &keys,
            &format!("document.querySelector({hue:?}).getAttribute('aria-valuenow')"),
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focus starts outside, so a thumb that never takes focus fails. The drag runs from a
/// quarter to three quarters into the pad on both axes.
#[test]
fn a_drag_moves_the_pad_and_leaves_the_thumb_focused() {
    block_on(async {
        for viewport in Viewport::ALL {
            let at = viewport.name();
            let fixture = Fixture::open("/color-picker", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, "#before", 5).await.unwrap();
            let (from, to) = pad_points(page, (0.25, 0.25), (0.75, 0.75)).await;
            pointer::drag(page, from, to, 10).await.unwrap();

            // 75% across, 25% up from the bottom; one percent of slack for
            // sub-pixel rounding of the pad's box.
            let near = "(() => { const [s, b] = READING.split(',').map(Number); \
                 return Math.abs(s - 75) <= 1 && Math.abs(b - 25) <= 1; })()"
                .replace("READING", READING);
            if let Err(e) = wait::for_js_true(page, &near, "the drag to land at 75%, 25%").await {
                panic!("at {at}: {e}; reading {}", reading(page).await);
            }

            wait::for_js_true(
                page,
                &format!("document.activeElement === document.querySelector({THUMB:?})"),
                "the thumb to hold focus after the drag",
            )
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            wait::for_js_change(page, READING, "ArrowUp after the drag", || {
                keyboard::press(page, keyboard::ARROW_UP)
            })
            .await
            .unwrap_or_else(|e| panic!("at {at}: {e}"));

            fixture
                .console
                .assert_clean(&format!("a pad drag at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// A press, no drag, jumps the thumb to the pressed spot and focuses it, on the pad
/// and on the hue track: the equivalent that spares the 16px thumbs 2.5.8 (todo 1811).
#[test]
fn a_press_on_the_pad_or_the_hue_track_jumps_the_thumb() {
    block_on(async {
        let fixture = Fixture::open("/color-picker", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#before", 5).await.unwrap();

        let (pressed, _) = pad_points(page, (0.25, 0.25), (0.0, 0.0)).await;
        pointer::click_at(page, pressed).await.unwrap();
        let near = "(() => { const [s, b] = READING.split(',').map(Number); \
             return Math.abs(s - 25) <= 1 && Math.abs(b - 75) <= 1; })()"
            .replace("READING", READING);
        if let Err(e) = wait::for_js_true(page, &near, "a press to land at 25%, 75%").await {
            panic!("{e}; reading {}", reading(page).await);
        }
        wait::for_js_true(
            page,
            &format!("document.activeElement === document.querySelector({THUMB:?})"),
            "the pad thumb to take focus on a press",
        )
        .await
        .unwrap();

        let track: [f64; 4] = page
            .evaluate(format!(
                "(() => {{ let el = document.querySelector({HUE:?}); \
                 while (el.getBoundingClientRect().width < 100) el = el.parentElement; \
                 const r = el.getBoundingClientRect(); return [r.x, r.y, r.width, r.height]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let (x, y) = (track[0] + track[2] * 0.25, track[1] + track[3] / 2.0);
        pointer::click_at(page, pointer::Point { x, y })
            .await
            .unwrap();
        if let Err(e) = wait::for_js_true(
            page,
            &format!(
                "Math.abs(Number(document.querySelector({HUE:?}).getAttribute('aria-valuenow')) - 90) <= 20 \
                 && document.activeElement === document.querySelector({HUE:?})"
            ),
            "a press a quarter along the hue track to land near 90 and focus its thumb",
        )
        .await
        {
            let state: String = page
                .evaluate(format!(
                    "`${{document.querySelector({HUE:?}).getAttribute('aria-valuenow')}} on ${{document.activeElement.outerHTML.slice(0, 120)}} track ${{JSON.stringify({track:?})}} hit ${{document.elementFromPoint({}, {})?.outerHTML.slice(0, 160)}}`",
                    x, y
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            panic!("{e}; hue {state}");
        }

        fixture
            .console
            .assert_clean("presses on the pad and the hue")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The hue thumb's face and the preview are painted from vars, not redrawn:
/// a hue step must still repaint both (todo 29).
#[test]
fn the_hue_thumb_and_the_preview_follow_the_value() {
    block_on(async {
        let fixture = Fixture::open("/color-picker-alpha", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let hue = "[role=slider][aria-label=Hue]";
        let face = format!("getComputedStyle(document.querySelector({hue:?})).backgroundColor");
        let preview_matches = "(() => { const n = parseInt(document.querySelector('#color').textContent.slice(1, 7), 16); \
             const rgb = `rgb(${n >> 16}, ${(n >> 8) & 255}, ${n & 255})`; \
             return getComputedStyle(document.querySelector('[data-slot=preview] > *')).backgroundImage.includes(rgb); })()";

        wait::for_js_true(
            page,
            preview_matches,
            "the preview to show the start colour",
        )
        .await
        .unwrap();
        keyboard::tab_to(page, hue, 10).await.unwrap();
        wait::for_js_change(
            page,
            &face,
            "Shift+ArrowRight to repaint the hue thumb",
            || keyboard::press_shift(page, keyboard::ARROW_RIGHT),
        )
        .await
        .unwrap();
        wait::for_js_true(page, preview_matches, "the preview to show the new colour")
            .await
            .unwrap();

        fixture.console.assert_clean("a hue step").unwrap();
        fixture.close().await.unwrap();
    });
}

/// WCAG 1.4.11: the saturation thumb's white border or its outer black ring,
/// composited over the pad under it, reads at 3:1 in either scheme (todo 554).
#[test]
fn the_saturation_thumb_parts_from_the_pad() {
    crate::boundary::assert_boundaries(
        "/color-picker",
        &format!(
            "const thumb = document.querySelector({THUMB:?});
             const pad = CSS(thumb, 'backgroundColor');
             const alpha = Number(CSS(thumb, 'boxShadow').match(/rgba\\(0, 0, 0, ([\\d.]+)\\)/)[1]);
             const ring = `rgb(${{pad.match(/[\\d.]+/g).slice(0, 3).map(c => c * (1 - alpha)).join(', ')}})`;
             return [
                 ['border or ring on the pad', Math.max(RATIO(CSS(thumb, 'borderTopColor'), pad), RATIO(ring, pad))],
                 ['border or ring on white', Math.max(RATIO(CSS(thumb, 'borderTopColor'), 'rgb(255, 255, 255)'),
                     RATIO('rgb(102, 102, 102)', 'rgb(255, 255, 255)') * (alpha >= 0.6))],
                 ['border or ring on black', Math.max(RATIO(CSS(thumb, 'borderTopColor'), 'rgb(0, 0, 0)'), 0)],
             ];"
        ),
    );
}

async fn reading(page: &chromiumoxide::Page) -> String {
    page.evaluate(READING).await.unwrap().into_value().unwrap()
}

/// Two points on the pad, each as a fraction of its width and height.
async fn pad_points(
    page: &chromiumoxide::Page,
    from: (f64, f64),
    to: (f64, f64),
) -> (pointer::Point, pointer::Point) {
    let rect: [f64; 4] = page
        .evaluate(format!(
            "(() => {{ const r = document.querySelector({THUMB:?}).parentElement.getBoundingClientRect(); \
             return [r.x, r.y, r.width, r.height]; }})()"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap();
    let point = |(fx, fy): (f64, f64)| pointer::Point {
        x: rect[0] + rect[2] * fx,
        y: rect[1] + rect[3] * fy,
    };
    (point(from), point(to))
}

const HUE: &str = "[role=slider][aria-label=Hue]";

/// `(saturation, brightness)` from the pad thumb's ARIA.
async fn pad<D: Driver>(d: &mut D) -> Result<(i32, i32)> {
    let now = d.attr(THUMB, "aria-valuenow").await?.unwrap_or_default();
    let text = d.attr(THUMB, "aria-valuetext").await?.unwrap_or_default();
    let brightness = text
        .split("brightness ")
        .nth(1)
        .and_then(|rest| rest.split('%').next())
        .unwrap_or_default();
    Ok((now.parse().unwrap_or(-1), brightness.parse().unwrap_or(-1)))
}

async fn pad_at<D: Driver>(d: &mut D, want: (i32, i32), after: &str) -> Result<()> {
    eventually(
        d,
        &format!("the pad at {want:?} after {after}"),
        async |d| Ok(pad(d).await? == want),
    )
    .await
}

async fn the_arrows_move_the_pad<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(THUMB).await?;
    pad_at(d, (87, 84), "focus").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    pad_at(d, (88, 84), "ArrowRight").await?;
    d.press(keyboard::ARROW_UP).await?;
    pad_at(d, (88, 85), "ArrowUp").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    pad_at(d, (87, 85), "ArrowLeft").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    pad_at(d, (87, 84), "ArrowDown").await?;
    d.press_shift(keyboard::ARROW_DOWN).await?;
    pad_at(d, (87, 74), "Shift+ArrowDown").await
}

/// Left and down from the thumb: less saturated, darker.
async fn a_drag_moves_both_axes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#before").await?;
    d.drag(THUMB, -40.0, 40.0).await?;
    eventually(d, "the drag to lower both axes", async |d| {
        let (s, b) = pad(d).await?;
        Ok(s < 87 && b < 84)
    })
    .await?;
    eventually_focused(d, THUMB, "the drag").await
}

async fn the_arrows_step_the_hue<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let before = d.attr(HUE, "aria-valuenow").await?;
    d.focus(HUE).await?;
    d.press_shift(keyboard::ARROW_RIGHT).await?;
    eventually(d, "Shift+ArrowRight to step the hue", async |d| {
        Ok(d.attr(HUE, "aria-valuenow").await? != before && d.text("#color").await? != "#1c7ed6")
    })
    .await
}

/// Todo 1545: 360 is hue 0, so End must stop the thumb at the right, not wrap it left.
async fn end_keeps_the_hue_at_the_right<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(HUE).await?;
    for (key, name) in [
        (keyboard::END, "End"),
        (keyboard::ARROW_RIGHT, "ArrowRight"),
    ] {
        d.press(key).await?;
        eventually(d, &format!("{name} to leave the hue at 359"), async |d| {
            Ok(d.attr(HUE, "aria-valuenow").await?.as_deref() == Some("359"))
        })
        .await?;
    }
    Ok(())
}

async fn a_swatch_click_picks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const GREEN: &str = "[aria-label='#40c057']";
    d.click(GREEN).await?;
    eventually(d, "a click to press the green swatch", async |d| {
        Ok(d.attr(GREEN, "aria-pressed").await?.as_deref() == Some("true"))
    })
    .await
}

async fn a_typed_colour_commits<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const INPUT: &str = "input[data-controlled]";
    d.click(INPUT).await?;
    d.press(keyboard::END).await?;
    for _ in 0..10 {
        d.press(keyboard::BACKSPACE).await?;
    }
    d.type_text("#228be6").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#readout", "#228be6ff", "Enter").await
}

e2e::scenario!(
    the_arrows_move_the_pad_on_both_axes,
    "/color-picker",
    the_arrows_move_the_pad
);
e2e::scenario!(
    a_drag_on_the_pad_moves_both_axes_and_focuses_the_thumb,
    "/color-picker",
    a_drag_moves_both_axes
);
e2e::scenario!(
    the_arrows_step_the_hue,
    "/color-picker",
    the_arrows_step_the_hue
);
e2e::scenario!(
    end_keeps_the_hue_thumb_at_the_right,
    "/color-picker",
    end_keeps_the_hue_at_the_right
);
e2e::scenario!(
    a_click_on_a_swatch_picks_its_colour,
    "/color-picker-swatches",
    a_swatch_click_picks
);
e2e::scenario!(
    a_typed_colour_commits_on_enter,
    "/color-field/alpha",
    a_typed_colour_commits
);
