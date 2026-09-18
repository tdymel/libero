//! `RangeSlider`: the drag picks a thumb, and only that one moves.

use anyhow::Result;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const UPPER: &str = "document.querySelectorAll('[role=slider]')[1]";
const READOUT: &str = "document.querySelector('#price-readout').textContent";
const LOWER_THUMB: &str = "[role=slider][aria-valuenow=\"20\"]";
const UPPER_THUMB: &str = "[role=slider][aria-valuenow=\"80\"]";

async fn reads<D: Driver>(d: &mut D, what: &str, check: impl Fn(&str) -> bool) -> Result<()> {
    eventually(d, what, async |d| {
        Ok(check(&d.text("#price-readout").await?))
    })
    .await
}

async fn the_keys_move_the_focused<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(LOWER_THUMB).await?;
    d.press(keyboard::HOME).await?;
    reads(d, "Home to move the lower thumb to 0", |r| r == "0-80").await?;
    d.focus(UPPER_THUMB).await?;
    d.press(keyboard::END).await?;
    reads(d, "End to move the upper thumb to 100", |r| r == "0-100").await
}

async fn a_drag_moves_the_grabbed<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.drag(UPPER_THUMB, -60.0, 0.0).await?;
    reads(d, "the upper thumb dragged, the lower kept", |r| {
        r.starts_with("20-") && r != "20-80"
    })
    .await
}

e2e::scenario!(
    the_keys_move_the_focused_thumb,
    "/range-slider",
    the_keys_move_the_focused
);
e2e::scenario!(
    a_drag_moves_the_grabbed_thumb_only,
    "/range-slider",
    a_drag_moves_the_grabbed
);

/// Todo 483: the label, which names both thumbs, focuses the first.
#[test]
fn a_click_on_the_label_focuses_the_lower_thumb() {
    crate::select::label_click_focuses("/range-slider", "[role=slider]");
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("range_slider", "/range-slider")
        .focusable("[role=slider]")
        .run();
}

/// The scenario above drags at desktop width; this adds the phone.
#[test]
fn a_drag_moves_the_grabbed_thumb_only_on_a_phone() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Mobile)
            .await
            .unwrap();
        // Read directly: `centre_of` takes the first match, the lower thumb.
        let upper: Vec<f64> = fixture
            .page
            .evaluate(format!(
                "(() => {{ const r = {UPPER}.getBoundingClientRect(); \
                 return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let start = pointer::Point {
            x: upper[0],
            y: upper[1],
        };
        let to = pointer::Point {
            x: start.x - 60.0,
            y: start.y,
        };

        wait::for_js_change(
            &fixture.page,
            &format!("{UPPER}.getAttribute('aria-valuenow')"),
            "the upper thumb to move",
            || pointer::drag(&fixture.page, start, to, 10),
        )
        .await
        .unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!(
                "{READOUT}.startsWith('20-') && {READOUT} !== '20-80' \
                 && {READOUT}.endsWith('-' + {UPPER}.getAttribute('aria-valuenow'))"
            ),
            "the read-out to show the lower value kept and the upper one dragged",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a drag on a phone").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Each thumb reaches its own closed value bubble through `aria-describedby`.
/// The baseline's lone `tooltip` is the first thumb's, open from the focus-ring pass.
#[test]
fn each_thumb_is_described_by_its_value_bubble() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Desktop)
            .await
            .unwrap();
        let open: u32 = fixture
            .page
            .evaluate("document.querySelectorAll('[role=tooltip]:not([hidden])').length")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(open, 0, "no bubble is open before focus or hover");
        for value in ["20", "80"] {
            let selector = format!("[role=slider][aria-valuenow='{value}']");
            let description = e2e::ax::description(&fixture.page, &selector)
                .await
                .unwrap();
            assert_eq!(
                description, value,
                "the {value} thumb's computed description"
            );
        }
        fixture.close().await.unwrap();
    });
}

/// Alt+ArrowLeft is Back: neither thumb swallows a browser chord (todo 562).
#[test]
fn modifier_chords_are_left_to_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let keys = [
            keyboard::ARROW_LEFT,
            keyboard::ARROW_RIGHT,
            keyboard::PAGE_UP,
            keyboard::PAGE_DOWN,
            keyboard::HOME,
            keyboard::END,
        ];
        keyboard::tab_to(page, "[role=slider]", 10).await.unwrap();
        keyboard::assert_chords_ignored(page, &keys, READOUT)
            .await
            .unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        keyboard::assert_chords_ignored(page, &keys, READOUT)
            .await
            .unwrap();
        fixture.close().await.unwrap();
    });
}
