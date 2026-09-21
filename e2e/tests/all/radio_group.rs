//! `RadioGroup`: the `RadioSet` archetype, not `RovingTabindex`: APG's radio group has no
//! Home or End, and its arrows select as they move.

use anyhow::Result;
use e2e::archetypes::RadioSet;
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually_focused, eventually_text};
use e2e::passes::target_size::MINIMUM;
use e2e::suite::Step;
use e2e::{Fixture, Suite, Viewport, passes::keyboard};

pub const RADIOS: &str = "[role=radiogroup] input[type=radio]";
/// The press target WCAG 2.5.8 measures: the whole row. Structural, as a row has no role or
/// attribute; suspect this first if it disagrees with the inputs.
pub const ROWS: &str = "[role=radiogroup] > div";
/// The tab stop. Not `RADIOS`: `querySelector` would name the first radio,
/// which is not where Tab enters a group whose second option is checked.
const CHECKED: &str = "[role=radiogroup] input[type=radio]:checked";
/// The "third" state's wait: not `CHECKED`, true at rest. The circle, since the input is
/// `opacity: 0` (757).
const THIRD_CHECKED: &str =
    "[role=radiogroup] input[type=radio][data-radio-index=\"2\"]:checked + *";

#[test]
fn it_meets_the_baseline() {
    Suite::new("radio_group", "/radio-group")
        .focusable(CHECKED)
        // Rows are under 24px, so 2.5.8 holds by spacing; `planted.rs` proves it can refuse
        // (377).
        .targets_spaced(ROWS)
        // The third option checked, so a snapshot that still says the second
        // is checked after an arrow press is caught.
        .state(
            "third",
            &[Step::TabTo(CHECKED), Step::Press(keyboard::ARROW_DOWN)],
            THIRD_CHECKED,
        )
        .run();
}

#[test]
fn it_honours_the_radio_group_contract() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/radio-group", viewport).await.unwrap();

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
                .assert_clean(&format!("the radio group contract at {}", viewport.name()))
                .unwrap();
            fixture.close().await.unwrap();
        }
    });
}

/// The docs page's switches on at once: cards in a row, every caption, an
/// error, `required` and a disabled option.
#[test]
fn a_card_field_meets_the_baseline() {
    Suite::new("radio_group_field", "/radio-group/field")
        .focusable(CHECKED)
        .targets(ROWS)
        .run();
}

/// Nothing checked and the first option disabled: Tab enters at the first
/// option that can be picked, and Space checks it.
#[test]
fn an_unanswered_group_enters_at_the_first_enabled_option() {
    block_on(async {
        let fixture = Fixture::open("/radio-group/empty", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let focused =
            format!("[...document.querySelectorAll({RADIOS:?})].indexOf(document.activeElement)");

        keyboard::tab_to(page, "button", 5).await.unwrap();
        keyboard::press(page, keyboard::TAB).await.unwrap();
        let entered: i64 = page
            .evaluate(focused.clone())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(entered, 1, "Tab entered the group at radio {entered}");

        keyboard::press(page, keyboard::SPACE).await.unwrap();
        e2e::wait::for_js_true(
            page,
            &format!("document.querySelectorAll({RADIOS:?})[1].checked"),
            "Space to check the focused radio",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// A click on the drawn circle checks the radio and focuses it, as a click on
/// a native radio does.
#[test]
fn a_circle_click_checks_and_focuses_the_radio() {
    block_on(async {
        let fixture = Fixture::open("/radio-group", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        e2e::passes::pointer::click(page, "input[data-radio-index=\"2\"] + span")
            .await
            .unwrap();
        e2e::wait::for_js_true(
            page,
            &format!(
                "(() => {{ const r = document.querySelectorAll({RADIOS:?})[2]; \
                 return r.checked && document.activeElement === r; }})()"
            ),
            "the clicked radio to be checked and focused",
        )
        .await
        .unwrap();
        fixture.close().await.unwrap();
    });
}

/// The circle takes a click, so it shows `pointer` as `Checkbox`'s box does;
/// a disabled one shows `not-allowed`.
#[test]
fn the_circle_shows_it_takes_a_click() {
    block_on(async {
        let fixture = Fixture::open("/radio-group/field", Viewport::Desktop)
            .await
            .unwrap();
        let cursors: Vec<String> = fixture
            .page
            .evaluate(format!(
                "[...document.querySelectorAll({RADIOS:?})]\
                 .map(r => getComputedStyle(r.nextElementSibling).cursor)"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(cursors, ["pointer", "pointer", "not-allowed"]);
        fixture.close().await.unwrap();

        let fixture = Fixture::open("/radio-group", Viewport::Desktop)
            .await
            .unwrap();
        let cursor: String = fixture
            .page
            .evaluate(format!(
                "getComputedStyle(document.querySelector({RADIOS:?}).nextElementSibling).cursor"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(cursor, "pointer", "a plain radio's circle");
        fixture.close().await.unwrap();
    });
}

/// Read-only: the group says so, and every radio stays a valid `radio`.
#[test]
fn a_readonly_group_meets_the_baseline() {
    Suite::new("radio_group_readonly", "/radio-group/readonly")
        .focusable(CHECKED)
        .targets_spaced(ROWS)
        .run();
}

/// Todo 746: a click in a read-only group leaves focus on the checked option, its one tab
/// stop. Blitz fires no `focusin` for a click's move (734).
async fn a_readonly_click_keeps_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for row in [
        "[role=radiogroup] > div:first-child",
        "[role=radiogroup] > div:last-child",
    ] {
        d.click(row).await?;
        eventually_focused(d, &radio(2), &format!("a click on {row}")).await?;
    }
    Ok(())
}

e2e::scenario!(
    a_readonly_click_keeps_focus_on_the_checked_option,
    "/radio-group/readonly",
    a_readonly_click_keeps_focus
);

/// The rows really are under 24px, so `targets_spaced` is needed; if they ever grow full
/// size this fails and says to declare `targets` (376, 377, 302).
#[test]
fn its_rows_are_undersized_so_the_spacing_exception_is_load_bearing() {
    block_on(async {
        for viewport in Viewport::ALL {
            let fixture = Fixture::open("/radio-group", viewport).await.unwrap();
            let heights: Vec<f64> = fixture
                .page
                .evaluate(format!(
                    "[...document.querySelectorAll({ROWS:?})]\
                     .map(el => el.getBoundingClientRect().height)"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(
                heights.len(),
                3,
                "expected a row per option, found {heights:?}"
            );
            for height in &heights {
                assert!(
                    *height < MINIMUM,
                    "at {}: a row is {height}px tall, which meets WCAG 2.5.8's {MINIMUM}px \
                     outright. The unit no longer needs the spacing exception, so \
                     `it_meets_the_baseline` should declare `targets(ROWS)` rather than \
                     `targets_spaced(ROWS)`",
                    viewport.name()
                );
            }
            fixture.close().await.unwrap();
        }
    });
}

/// Forced colours paint every background `Canvas`, so the checked dot vanished
/// and every radio looked unchecked (todo 506).
#[test]
fn the_checked_dot_shows_in_forced_colours() {
    use chromiumoxide::cdp::browser_protocol::emulation::{MediaFeature, SetEmulatedMediaParams};
    block_on(async {
        let fixture = Fixture::open("/radio-group", Viewport::Desktop)
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
        e2e::wait::for_js_true(
            page,
            "matchMedia('(forced-colors: active)').matches",
            "forced colours to apply",
        )
        .await
        .unwrap();
        let [dot, page_bg]: [String; 2] = page
            .evaluate(
                "(() => {
                    const dot = document.querySelector('input[type=radio]:checked ~ span > span');
                    return [getComputedStyle(dot).backgroundColor,
                            getComputedStyle(document.body).backgroundColor];
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(dot, page_bg, "the checked dot is painted the page's colour");
        fixture.close().await.unwrap();
    });
}

fn radio(n: usize) -> String {
    format!("[role=radiogroup] > div:nth-of-type({n}) input[type=radio]")
}

async fn tab_enters_at_the_checked<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#before").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, &radio(2), "Tab").await
}

async fn the_arrows_move_and_select<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(&radio(2)).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, "#picked", "Some(Team)", "ArrowDown").await?;
    eventually_focused(d, &radio(3), "ArrowDown").await?;
    // APG: the arrows wrap.
    d.press(keyboard::ARROW_DOWN).await?;
    eventually_text(d, "#picked", "Some(Free)", "ArrowDown from the last").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_text(d, "#picked", "Some(Team)", "ArrowUp from the first").await
}

async fn a_label_click_selects<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("[role=radiogroup] > div:first-of-type label")
        .await?;
    eventually_text(d, "#picked", "Some(Free)", "a click on Free's label").await
}

e2e::scenario!(
    tab_enters_a_radio_group_at_the_checked_radio,
    "/radio-group/echo",
    tab_enters_at_the_checked
);
e2e::scenario!(
    the_arrows_move_and_select_in_a_radio_group,
    "/radio-group/echo",
    the_arrows_move_and_select
);
e2e::scenario!(
    a_click_on_an_option_label_selects_it,
    "/radio-group/echo",
    a_label_click_selects
);
