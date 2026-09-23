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

/// Todo 1060: thumbs on one spot part the way the pointer first moves, so the
/// upper one can leave the lower one rightwards.
async fn collapsed_part<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(UPPER_THUMB).await?;
    d.press(keyboard::HOME).await?;
    reads(d, "Home to stop the upper thumb on the lower", |r| {
        r == "20-20"
    })
    .await?;
    d.drag("[role=slider]", 60.0, 0.0).await?;
    reads(d, "a rightward drag to take the upper thumb", |r| {
        r.starts_with("20-") && r != "20-20"
    })
    .await
}

const MAXIMUM: &str = "[role=slider][aria-label=Maximum]";

/// Whether the read-out's pair lies within `low` and `high`.
fn pair_within(readout: &str, low: (f64, f64), high: (f64, f64)) -> bool {
    let Some((a, b)) = readout.split_once('-') else {
        return false;
    };
    let within = |text: &str, (from, to): (f64, f64)| {
        text.parse::<f64>()
            .is_ok_and(|value| (from..=to).contains(&value))
    };
    within(a, low) && within(b, high)
}

/// Todo 1060's taps, from 20-80: the value tapped, whether the thumbs are first
/// put on one spot at 20, and where the pair must land.
type Tap = (f64, bool, (f64, f64), (f64, f64));

const TAPS: [Tap; 4] = [
    (65.0, false, (20.0, 20.0), (60.0, 70.0)),
    (92.0, false, (20.0, 20.0), (87.0, 97.0)),
    (35.0, true, (20.0, 20.0), (30.0, 40.0)),
    (8.0, true, (3.0, 13.0), (20.0, 20.0)),
];

/// Todo 1060: a still tap on the track moves the nearest thumb, never the
/// lower one by default: between the thumbs, beyond the upper, beside a pair.
async fn track_taps<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (low, high) = (d.rect(LOWER_THUMB).await?, d.rect(UPPER_THUMB).await?);
    let y = low.y + low.height / 2.0;
    let (x20, x80) = (low.x + low.width / 2.0, high.x + high.width / 2.0);
    for (value, collapse, lower, upper) in TAPS {
        if collapse {
            d.focus(MAXIMUM).await?;
            d.press(keyboard::HOME).await?;
            reads(d, "Home to put the thumbs on one spot", |r| r == "20-20").await?;
        }
        d.click_at(x20 + (value - 20.0) / 60.0 * (x80 - x20), y)
            .await?;
        let what = format!("a tap at {value} to move the nearest thumb there");
        reads(d, &what, |r| pair_within(r, lower, upper)).await?;
    }
    Ok(())
}

e2e::scenario!(
    a_track_tap_moves_the_nearest_thumb,
    "/range-slider",
    track_taps
);
e2e::scenario!(
    thumbs_on_one_spot_part_the_way_the_drag_goes,
    "/range-slider",
    collapsed_part
);
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

/// Todo 1060 on web touch: a touch on the upper thumb drags the upper thumb.
#[test]
fn a_touch_drags_the_upper_thumb() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Mobile)
            .await
            .unwrap();
        let (x, y): (f64, f64) = fixture
            .page
            .evaluate(format!(
                "(() => {{ const r = {UPPER}.getBoundingClientRect(); \
                 return [r.x + r.width / 2, r.y + r.height / 2]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let start = pointer::Point { x, y };
        let to = pointer::Point { x: x - 60.0, y };
        wait::for_js_change(
            &fixture.page,
            &format!("{UPPER}.getAttribute('aria-valuenow')"),
            "the upper thumb to move",
            || pointer::touch_drag(&fixture.page, start, to, 8),
        )
        .await
        .unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!("{READOUT}.startsWith('20-') && {READOUT} !== '20-80'"),
            "the lower thumb to stay",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("a touch drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// [`track_taps`] with web touch taps.
#[test]
fn a_touch_tap_on_the_track_moves_the_nearest_thumb() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        let low = pointer::centre_of(page, LOWER_THUMB).await.unwrap();
        let high = pointer::centre_of(page, UPPER_THUMB).await.unwrap();
        for (value, collapse, lower, upper) in TAPS {
            if collapse {
                page.evaluate(format!("document.querySelector('{MAXIMUM}').focus()"))
                    .await
                    .unwrap();
                keyboard::press(page, keyboard::HOME).await.unwrap();
                wait::for_js_true(page, &format!("{READOUT} === '20-20'"), "one spot")
                    .await
                    .unwrap();
            }
            let at = pointer::Point {
                x: low.x + (value - 20.0) / 60.0 * (high.x - low.x),
                y: low.y,
            };
            pointer::touch_drag(page, at, at, 0).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(([a, b]) => a >= {} && a <= {} && b >= {} && b <= {})({READOUT}.split('-').map(Number))",
                    lower.0, lower.1, upper.0, upper.1
                ),
                &format!("a touch tap at {value} to move the nearest thumb there"),
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("touch taps").unwrap();
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

/// Todo 1134: the two thumbs sit in one group named by the label, each a slider of its own.
#[test]
fn the_thumbs_form_a_labelled_group() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Desktop)
            .await
            .unwrap();
        let tree = e2e::ax::snapshot(&fixture.page, "[role=group]")
            .await
            .unwrap();
        assert!(tree.contains("group \"Price\""), "{tree}");
        assert!(tree.contains("slider \"Price Minimum\""), "{tree}");
        assert!(tree.contains("slider \"Price Maximum\""), "{tree}");
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
