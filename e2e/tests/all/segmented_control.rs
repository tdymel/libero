//! `SegmentedControl`: the `RadioSet` archetype. Each segment is a native radio, so it is a
//! radio group, not a tab strip, whatever it looks like.

use anyhow::Result;
use e2e::archetypes::RadioSet;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_focused, eventually_text};
use e2e::suite::Step;
use e2e::{
    Fixture, Suite, Viewport,
    passes::{keyboard, pointer},
};

pub const RADIOS: &str = "[role=radiogroup] input[type=radio]";
const CHECKED: &str = "[role=radiogroup] input[type=radio]:checked";

/// The checked segment's label, not `:checked`: natively that did not follow
/// ArrowRight.
async fn selected<D: Driver>(d: &mut D, expected: &str, after: &str) -> Result<()> {
    let label = "[role=radiogroup] label[data-state~=checked]";
    eventually_text(d, label, expected, after).await
}

async fn a_label_click_selects<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    selected(d, "Center", "mounting").await?;
    d.click("input[type=radio] + label").await?;
    selected(d, "Left", "a click on Left's label").await
}

async fn the_arrows_move_focus_and_wrap<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("input[aria-label=Center]").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    selected(d, "Right", "ArrowRight").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    d.press(keyboard::ARROW_LEFT).await?;
    selected(d, "Left", "two ArrowLefts").await?;
    eventually_focused(d, "input[aria-label=Left]", "two ArrowLefts").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    selected(d, "Right", "ArrowLeft from Left").await?;
    eventually_focused(d, "input[aria-label=Right]", "ArrowLeft from Left").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    selected(d, "Left", "ArrowDown from Right").await
}

async fn the_arrows_move_focus_past_an_odd_id<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("input[aria-label=Center]").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    selected(d, "Right", "ArrowRight").await?;
    eventually_focused(d, "input[aria-label=Right]", "ArrowRight").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    selected(d, "Left", "a second ArrowRight").await?;
    eventually_focused(d, "input[aria-label=Left]", "a second ArrowRight").await
}

async fn enter_picks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("input[aria-label=Left]").await?;
    d.press(keyboard::ENTER).await?;
    selected(d, "Left", "Enter on Left").await
}

async fn enter_submits_in_a_form<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("input[aria-label=Left]").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#submits", "Submits 1", "Enter on Left").await?;
    selected(d, "Center", "Enter in a form").await
}

async fn one_tab_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "input[aria-label=Center]", "Tab").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "the second Tab").await?;
    d.press_shift(keyboard::TAB).await?;
    eventually_focused(d, "input[aria-label=Center]", "Shift+Tab").await
}

e2e::scenario!(
    a_click_on_a_segment_label_selects_it,
    "/segmented-control",
    a_label_click_selects
);
e2e::scenario!(
    the_arrows_move_focus_with_the_selection_and_wrap,
    "/segmented-control",
    the_arrows_move_focus_and_wrap,
    web: skip("1826: it_honours_the_radio_group_contract runs these keys at both viewports"),
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    the_arrows_move_focus_when_the_id_is_no_css_identifier,
    "/segmented-control/odd-id",
    the_arrows_move_focus_past_an_odd_id,
    android: skip("958: element identity on the WebView"),
    desktop: skip("958: element identity on the WebView")
);
e2e::scenario!(
    enter_picks_the_focused_segment,
    "/segmented-control",
    enter_picks
);
e2e::scenario!(
    enter_in_a_form_submits_and_picks_nothing,
    "/segmented-control/form",
    enter_submits_in_a_form
);
e2e::scenario!(
    tab_enters_on_the_checked_segment_and_leaves_with_the_next_tab,
    "/segmented-control",
    one_tab_stop
);
/// The "right" state's wait: the last segment's label, checked. Not `CHECKED` (the radio is
/// hidden) nor any checked label (the middle one is, at rest).
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

/// The field with its error, read-only, the disabled segments and long labels (todo 1773).
#[test]
fn the_other_variants_meet_the_baseline() {
    // The disabled pick's tab stop is the first enabled segment.
    for (name, route, stop) in [
        (
            "segmented_control-field",
            "/segmented-control/field",
            CHECKED,
        ),
        (
            "segmented_control-readonly",
            "/segmented-control/readonly",
            CHECKED,
        ),
        (
            "segmented_control-disabled-middle",
            "/segmented-control/disabled-middle",
            CHECKED,
        ),
        (
            "segmented_control-disabled-pick",
            "/segmented-control/disabled-pick",
            "input[aria-label=Left]",
        ),
        ("segmented_control-long", "/segmented-control/long", CHECKED),
    ] {
        Suite::new(name, route)
            .focusable(stop)
            .targets("[role=radiogroup] label")
            .run();
    }
}

#[test]
fn it_honours_the_radio_group_contract() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
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
        })
        .await;
    });
}

/// Todo 491: the picked segment showed only a faint tint.
#[test]
fn the_picked_segment_shows_the_on_state_ring() {
    use crate::button::{
        assert_gray_in_forced_colours, assert_on_in_forced_colours, assert_on_marker,
    };
    const PICKED: &str = "[role=radiogroup] label[for$=\"-segment-1\"]";
    const OTHER: &str = "[role=radiogroup] label[for$=\"-segment-0\"]";
    block_on(async {
        let fixture = Fixture::open("/segmented-control", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        assert_on_marker(page, PICKED, OTHER).await;
        e2e::browser::force_colours(page).await.unwrap();
        assert_on_in_forced_colours(page, PICKED, OTHER).await;
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/segmented-control/disabled-middle", Viewport::Desktop)
            .await
            .unwrap();
        e2e::browser::force_colours(&fixture.page).await.unwrap();
        assert_gray_in_forced_colours(&fixture.page, PICKED).await;
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/segmented-control/disabled-pick", Viewport::Desktop)
            .await
            .unwrap();
        e2e::browser::force_colours(&fixture.page).await.unwrap();
        crate::button::assert_text_in_forced_colours(&fixture.page, PICKED, "HighlightText").await;
        fixture.close().await.unwrap();
    });
}

/// Waits until the focused and the checked radio are at `focused` and `checked` (todo 1820);
/// `None` reads no focus.
async fn until_at(fixture: &Fixture, focused: Option<i64>, checked: i64, what: &str) {
    let radios = format!("[...document.querySelectorAll('{RADIOS}')]");
    let focus = focused.map_or("true".to_string(), |at| {
        format!("{radios}.indexOf(document.activeElement) === {at}")
    });
    e2e::wait::for_js_true(
        &fixture.page,
        &format!("{focus} && {radios}.findIndex(el => el.checked) === {checked}"),
        what,
    )
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

/// Todo 660: a raw `<form>` counts too, read off the DOM; Enter submits it and keeps the
/// pick. The `Form` case is the scenario's (todo 1826).
#[test]
fn enter_submits_a_raw_form() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/raw-form", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        assert_eq!(
            tab_in(&fixture).await,
            1,
            "Tab enters at the checked segment"
        );
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        until_at(&fixture, None, 2, "the arrow to pick").await;
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        e2e::wait::for_js_true(
            page,
            "document.querySelector('#submits').dataset.submits === '1'",
            "Enter to submit",
        )
        .await
        .unwrap();
        assert_eq!(checked(&fixture).await, 2, "Enter moved the pick");

        fixture.console.assert_clean("Enter in a form").unwrap();
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
        e2e::clock::settle(page).await.unwrap();
        assert_eq!(checked(&fixture).await, 1);
        // Todo 746: focus stays on the checked segment, the one tab stop.
        e2e::wait::for_js_true(
            page,
            "document.activeElement?.id.endsWith('-segment-1')",
            "focus on the checked segment after a read-only click",
        )
        .await
        .unwrap();

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
        until_at(&fixture, Some(2), 2, "ArrowRight over the disabled segment").await;
        keyboard::press(&fixture.page, keyboard::ARROW_LEFT)
            .await
            .unwrap();
        until_at(&fixture, Some(0), 0, "ArrowLeft back over it").await;

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
        until_at(
            &fixture,
            Some(2),
            2,
            "ArrowRight to the next enabled segment",
        )
        .await;

        fixture.console.assert_clean("disabled pick").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Seven segments ran 359px past a phone's page, which then scrolled sideways
/// (WCAG 1.4.10, todo 525).
#[test]
fn a_long_strip_wraps_inside_a_phone() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/long", Viewport::Mobile)
            .await
            .unwrap();
        let [page_overflow, strip_overflow]: [f64; 2] = fixture
            .page
            .evaluate(
                "(() => { const strip = document.querySelector('[role=radiogroup]'); \
                 const edge = strip.parentElement.getBoundingClientRect().right; \
                 const right = Math.max(...[...strip.querySelectorAll('label')] \
                 .map((l) => l.getBoundingClientRect().right)); \
                 const root = document.documentElement; \
                 return [root.scrollWidth - root.clientWidth, Math.max(0, right - edge)]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(page_overflow, 0.0, "the page scrolls sideways");
        assert!(
            strip_overflow < 0.5,
            "a segment runs {strip_overflow}px past its box"
        );
        fixture.close().await.unwrap();
    });
}

/// Todo 2465: one enabled segment has nowhere to move, so the arrows and Home/End stay the page's.
#[test]
fn a_lone_segment_leaves_the_arrows_to_the_page() {
    block_on(async {
        let fixture = Fixture::open("/segmented-control/lone", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, "#lone-segment-1", 10).await.unwrap();
        crate::accordion::assert_keys_left_to_page(
            page,
            &[
                keyboard::ARROW_DOWN,
                keyboard::ARROW_UP,
                keyboard::ARROW_RIGHT,
                keyboard::ARROW_LEFT,
            ],
        )
        .await;
        fixture.close().await.unwrap();
    });
}
