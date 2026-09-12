//! `RangeSlider`: the drag picks a thumb, and only that one moves.

use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, wait};

const UPPER: &str = "document.querySelectorAll('[role=slider]')[1]";
const READOUT: &str = "document.querySelector('#price-readout').textContent";

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

#[test]
fn a_drag_moves_the_grabbed_thumb_only() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/range-slider", viewport).await.unwrap();
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
            .unwrap_or_else(|e| panic!("at {}: the drag did not move it: {e}", viewport.name()));
            wait::for_js_true(
                &fixture.page,
                &format!(
                    "{READOUT}.startsWith('20-') && {READOUT} !== '20-80' \
                     && {READOUT}.endsWith('-' + {UPPER}.getAttribute('aria-valuenow'))"
                ),
                "the read-out to show the lower value kept and the upper one dragged",
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

#[test]
fn the_keys_move_the_focused_thumb() {
    block_on(async {
        let fixture = Fixture::open("/range-slider", Viewport::Desktop)
            .await
            .unwrap();
        keyboard::tab_to(&fixture.page, "[role=slider]", 10)
            .await
            .unwrap();
        keyboard::press(&fixture.page, keyboard::HOME)
            .await
            .unwrap();
        wait::for_js_true(
            &fixture.page,
            &format!("{READOUT} === '0-80'"),
            "Home to move the lower thumb to the minimum",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}
