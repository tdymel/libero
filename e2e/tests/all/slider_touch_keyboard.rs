//! `Slider`'s touch and keyboard contracts: a touch on the track or thumb, and the
//! arrow, Page, modifier, Tab and Escape keys.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::pointer;
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

use super::slider::{BUBBLE, SCROLL_THUMB, THUMB, VALUE_NOW};
use crate::settle;

/// Todo 1058: a touch held on the thumb opens no lingering long-press bubble;
/// its tap's bubble is gone right after the release.
async fn held_thumb<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // Held, a Tooltip's touch linger never runs out: a bubble on it stays up.
    let held = d.hold_timers(&[TOUCH_LINGER_MS]).await?;
    d.long_press(THUMB, 800).await?;
    let released = std::time::Instant::now();
    eventually(d, "the bubble to go with the release", async |d| {
        Ok(!d.exists(BUBBLE).await?)
    })
    .await?;
    // The first read can come before the tap's bubble renders: settle, then read again.
    d.settle().await?;
    ensure!(!d.exists(BUBBLE).await?, "a bubble rose after the release");
    if held {
        let lingering = d.armed(TOUCH_LINGER_MS).await?;
        ensure!(
            lingering == 0,
            "{lingering} touch lingers armed after the release"
        );
    }
    ensure!(
        held || released.elapsed() < std::time::Duration::from_millis(u64::from(TOUCH_LINGER_MS)),
        "the bubble lingered after the release"
    );
    Ok(())
}

/// A Tooltip's stay after a touch release.
const TOUCH_LINGER_MS: u32 = 1500;

e2e::scenario!(
    a_touch_held_on_the_thumb_leaves_no_bubble_after_the_release,
    "/slider/drag",
    held_thumb,
    desktop: skip("1126: no touch input under Xvfb")
);

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
            .evaluate_bounded(format!(
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
        // Whatever End would do has run once its handler's renders have.
        e2e::clock::settle(page).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement !== document.querySelector('#fixed [role=slider]')",
            "Tab to leave the read-only thumb",
        )
        .await
        .unwrap();
        let state: Vec<String> = page
            .evaluate_bounded(
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
            .evaluate_bounded(format!(
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
