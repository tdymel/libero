//! `ColorField`'s dropdown between two buttons (todo 449). `/color-field/alpha`
//! holds `#1c7ed6` with the alpha slider and three swatches;
//! `/color-field/swatches` holds only the swatches; `/color-field/keep-text`
//! has `fix_on_blur: false` and no dropdown.

use e2e::browser::block_on;
use e2e::passes::keyboard;
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "input[data-controlled]";
const EYE_DROPPER: &str = "[aria-label='Pick a color from the screen']";

/// The dropdown is portaled after the page: Tab past either end goes back
/// through the text input, not to the end of the document.
#[test]
fn tab_past_the_dropdown_moves_on_from_the_field() {
    block_on(async {
        // Stops in the dropdown after the one Arrow Down focuses.
        for (route, later_stops) in [("/color-field/alpha", 5), ("/color-field/swatches", 2)] {
            let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, INPUT, 5).await.unwrap();
            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            expect(
                page,
                "!!document.activeElement.closest('[role=dialog]')",
                "Arrow Down to enter the dropdown",
            )
            .await;
            keyboard::press_shift(page, keyboard::TAB).await.unwrap();
            expect(
                page,
                &format!("document.activeElement === document.querySelector({INPUT:?})"),
                &format!("{route}: Shift+Tab from the first stop to land on the text input"),
            )
            .await;

            keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
            for _ in 0..later_stops {
                keyboard::press(page, keyboard::TAB).await.unwrap();
            }
            expect(
                page,
                "document.activeElement.getAttribute('aria-label') === '#228be6'",
                &format!("{route}: Tab to reach the last swatch"),
            )
            .await;
            keyboard::press(page, keyboard::TAB).await.unwrap();
            expect(
                page,
                &format!("document.activeElement === document.querySelector({EYE_DROPPER:?})"),
                &format!("{route}: Tab from the last swatch to move on from the text input"),
            )
            .await;
            keyboard::press(page, keyboard::TAB).await.unwrap();
            expect(
                page,
                "document.activeElement.id === 'after' && !document.querySelector('[role=dialog]')",
                &format!("{route}: Tab on to the button after the field, the dropdown closed"),
            )
            .await;

            fixture.console.assert_clean(route).unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// Every thumb is named without the caller naming it, and hue and alpha are
/// heard in their units.
#[test]
fn the_picker_thumbs_are_named_with_their_units() {
    block_on(async {
        let fixture = Fixture::open("/color-field/alpha", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        expect(
            page,
            "!!document.activeElement.closest('[role=dialog]')",
            "Arrow Down to enter the dropdown",
        )
        .await;
        let tree = e2e::ax::snapshot(page, "[role=dialog]").await.unwrap();
        for thumb in [
            "slider \"Saturation\"",
            "slider \"Hue\"",
            "slider \"Alpha\"",
        ] {
            assert!(tree.contains(thumb), "no {thumb} in:\n{tree}");
        }
        expect(
            page,
            "document.querySelector('[role=slider][aria-label=Hue]').getAttribute('aria-valuetext') === '208 degrees' \
             && document.querySelector('[role=slider][aria-label=Alpha]').getAttribute('aria-valuetext') === '100%'",
            "hue in degrees and alpha in percent",
        )
        .await;

        fixture.console.assert_clean("the picker thumbs").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 552: Enter on text that is no color marks the field invalid, shows the
/// error and says it, while focus stays; the next keystroke clears it.
#[test]
fn unparsable_text_shows_and_says_an_error() {
    block_on(async {
        let fixture = Fixture::open("/color-field/alpha", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        page.evaluate(format!("document.querySelector({INPUT:?}).select()"))
            .await
            .unwrap();
        keyboard::type_text(page, "nope").await.unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        let error = "Not a valid color";
        expect(
            page,
            &format!(
                "(() => {{ const input = document.querySelector({INPUT:?}); \
                 const ids = (input.getAttribute('aria-describedby') || '').split(' '); \
                 return input.getAttribute('aria-invalid') === 'true' \
                   && ids.some(id => document.getElementById(id)?.textContent.trim() === {error:?}) \
                   && document.activeElement === input \
                   && [...document.querySelectorAll('[role=status]')].some(s => s.textContent === {error:?}); }})()"
            ),
            "Enter on unparsable text to show and announce the error",
        )
        .await;
        keyboard::type_text(page, "x").await.unwrap();
        expect(
            page,
            &format!("document.querySelector({INPUT:?}).getAttribute('aria-invalid') !== 'true'"),
            "the next keystroke to clear the error",
        )
        .await;
        fixture.console.assert_clean("an unparsable color").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 699: with `fix_on_blur: false`, Tab away from text that is no color
/// keeps the text and the value, and marks the field invalid with the error.
/// Focus has left, so nothing is announced: the input describes it on return.
#[test]
fn blur_on_unparsable_text_keeps_it_with_an_error() {
    block_on(async {
        let fixture = Fixture::open("/color-field/keep-text", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, INPUT, 5).await.unwrap();
        page.evaluate(format!("document.querySelector({INPUT:?}).select()"))
            .await
            .unwrap();
        keyboard::type_text(page, "nope").await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let error = "Not a valid color";
        expect(
            page,
            &format!(
                "(() => {{ const input = document.querySelector({INPUT:?}); \
                 const ids = (input.getAttribute('aria-describedby') || '').split(' '); \
                 return document.activeElement.id === 'after' \
                   && input.value === 'nope' \
                   && input.getAttribute('aria-invalid') === 'true' \
                   && ids.some(id => document.getElementById(id)?.textContent.trim() === {error:?}) \
                   && document.getElementById('readout').textContent === '#40c057'; }})()"
            ),
            "Tab from unparsable text to keep it and show the error",
        )
        .await;
        fixture
            .console
            .assert_clean("a blurred unparsable color")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn expect(page: &chromiumoxide::Page, check: &str, what: &str) {
    if let Err(error) = wait::for_js_true(page, check, what).await {
        let focus: String = page
            .evaluate("document.activeElement.outerHTML.slice(0, 160)")
            .await
            .and_then(|value| Ok(value.into_value()?))
            .unwrap_or_default();
        panic!("{what}: {error}; focus={focus}");
    }
}
