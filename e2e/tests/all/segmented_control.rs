//! `SegmentedControl`: the `RadioSet` archetype.
//!
//! Each segment is a native radio beside its `<label>`, so it is a radio group
//! to assistive technology and to the keyboard - not a tab strip, whatever it
//! looks like.

use e2e::archetypes::RadioSet;
use e2e::browser::block_on;
use e2e::suite::Step;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{keyboard, pointer},
};

pub const RADIOS: &str = "[role=radiogroup] input[type=radio]";
const CHECKED: &str = "[role=radiogroup] input[type=radio]:checked";
/// What the "right" state waits on: the last segment's label, checked. Not
/// `CHECKED`, since the radio is visually hidden and never counts as visible,
/// and not any checked label, since the middle one is checked at rest.
const RIGHT_CHECKED: &str = "[role=radiogroup] label[for$=\"-segment-2\"][data-state~=checked]";

#[test]
fn it_meets_the_baseline() {
    Suite::new("segmented_control", "/segmented-control")
        .focusable(CHECKED)
        .targets("[role=radiogroup] label")
        .state(
            "right",
            &[Step::TabTo(CHECKED), Step::Press(keyboard::ARROW_RIGHT)],
            RIGHT_CHECKED,
        )
        .run();
}

#[test]
fn it_honours_the_radio_group_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/segmented-control", viewport).await.unwrap();

            RadioSet {
                radios: RADIOS,
                checked: 1,
                tab_budget: 5,
            }
            .assert_contract(&fixture.page)
            .await
            .unwrap_or_else(|e| panic!("at {}: {e}", viewport.name()));

            fixture
                .console
                .assert_clean(&format!(
                    "the segmented control contract at {}",
                    viewport.name()
                ))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

async fn settle(fixture: &Fixture) {
    fixture
        .page
        .evaluate("new Promise(r => setTimeout(() => r(1), 100))")
        .await
        .unwrap();
}

/// The focused radio's index in the group, `-1` when focus is elsewhere.
async fn focused(fixture: &Fixture) -> i64 {
    fixture
        .page
        .evaluate(format!(
            "[...document.querySelectorAll('{RADIOS}')].indexOf(document.activeElement)"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn checked(fixture: &Fixture) -> i64 {
    fixture
        .page
        .evaluate(format!(
            "[...document.querySelectorAll('{RADIOS}')].findIndex(el => el.checked)"
        ))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

/// Tabs from the top of the page until focus reaches a radio.
async fn tab_in(fixture: &Fixture) -> i64 {
    fixture
        .page
        .evaluate("document.activeElement && document.activeElement.blur()")
        .await
        .unwrap();
    for _ in 0..4 {
        keyboard::press(&fixture.page, keyboard::TAB).await.unwrap();
        settle(fixture).await;
        let at = focused(fixture).await;
        if at >= 0 {
            return at;
        }
    }
    -1
}

/// The field's wiring lands on the group: name, captions, error, required.
#[test]
fn the_group_carries_the_field_wiring() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/field", Viewport::Desktop)
            .await
            .unwrap();

        let tree = e2e::ax::snapshot(&fixture.page, "[role=radiogroup]")
            .await
            .unwrap();
        assert!(
            tree.starts_with("radiogroup \"Alignment\" [invalid] [required]"),
            "{tree}"
        );
        let described: String = fixture
            .page
            .evaluate(
                "document.querySelector('[role=radiogroup]').getAttribute('aria-describedby')\
                 .split(' ').map(id => document.getElementById(id).textContent).join(' | ')",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            described,
            "Where each line starts. | Applies to the whole document. | Pick an alignment."
        );

        fixture
            .console
            .assert_clean("segmented control field")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Read-only: focusable, but no click, arrow, Space or Enter picks a segment.
#[test]
fn a_read_only_strip_keeps_its_pick() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/readonly", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        assert_eq!(
            tab_in(&fixture).await,
            1,
            "Tab enters at the checked segment"
        );
        for key in [keyboard::ARROW_RIGHT, keyboard::SPACE, keyboard::ENTER] {
            keyboard::press(page, key).await.unwrap();
        }
        pointer::click(page, "label[for$='-segment-0']")
            .await
            .unwrap();
        settle(&fixture).await;
        assert_eq!(checked(&fixture).await, 1);

        fixture
            .console
            .assert_clean("read-only segmented control")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The arrows step over a disabled segment, both ways.
#[test]
fn the_arrows_skip_a_disabled_segment() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/disabled-middle", Viewport::Desktop)
            .await
            .unwrap();

        assert_eq!(
            tab_in(&fixture).await,
            0,
            "Tab enters at the checked segment"
        );
        keyboard::press(&fixture.page, keyboard::ARROW_RIGHT)
            .await
            .unwrap();
        settle(&fixture).await;
        assert_eq!((focused(&fixture).await, checked(&fixture).await), (2, 2));
        keyboard::press(&fixture.page, keyboard::ARROW_LEFT)
            .await
            .unwrap();
        settle(&fixture).await;
        assert_eq!((focused(&fixture).await, checked(&fixture).await), (0, 0));

        fixture
            .console
            .assert_clean("disabled middle segment")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A pick that was disabled later must not take the whole strip out of the
/// tab order: Chromium lets Tab reach only the checked radio of a group.
#[test]
fn tab_reaches_a_strip_whose_pick_is_disabled() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/disabled-pick", Viewport::Desktop)
            .await
            .unwrap();

        assert_eq!(tab_in(&fixture).await, 0, "Tab never reached the strip");
        keyboard::press(&fixture.page, keyboard::ARROW_RIGHT)
            .await
            .unwrap();
        settle(&fixture).await;
        assert_eq!((focused(&fixture).await, checked(&fixture).await), (2, 2));

        fixture.console.assert_clean("disabled pick").unwrap();
        fixture.close().await.unwrap();
    });
}
