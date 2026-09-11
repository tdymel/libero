//! `ColorPicker`'s saturation pad: a 2-D slider on one thumb. Left/Right move
//! the saturation, Up/Down the brightness; a drag on the pad moves both and
//! leaves the thumb focused (todo 406).
//!
//! `/color-picker` starts at `#1c7ed6`: saturation 87%, brightness 84%.

use e2e::browser::block_on;
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

#[test]
fn the_arrows_move_the_pad_on_both_axes() {
    block_on(async {
        let fixture = Fixture::open("/color-picker", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, THUMB, 10).await.unwrap();
        expect(page, "87,84", "the starting colour").await;

        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        expect(page, "88,84", "ArrowRight to raise the saturation only").await;
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        expect(page, "88,85", "ArrowUp to raise the brightness only").await;
        keyboard::press(page, keyboard::ARROW_LEFT).await.unwrap();
        expect(page, "87,85", "ArrowLeft to lower the saturation only").await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(page, "87,84", "ArrowDown to lower the brightness only").await;
        keyboard::press_shift(page, keyboard::ARROW_DOWN)
            .await
            .unwrap();
        expect(page, "87,74", "Shift+ArrowDown to step ten").await;

        fixture.console.assert_clean("the pad keys").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Focus starts on a button outside, so a thumb that never takes focus fails
/// here rather than passing on focus it already had. The drag runs from a
/// quarter into the pad to three quarters, so the reading names where it
/// ended on both axes.
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

async fn reading(page: &chromiumoxide::Page) -> String {
    page.evaluate(READING).await.unwrap().into_value().unwrap()
}

async fn expect(page: &chromiumoxide::Page, want: &str, what: &str) {
    if let Err(e) = wait::for_js_true(page, &format!("{READING} === {want:?}"), what).await {
        panic!("{e}; reading {}", reading(page).await);
    }
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
