//! The date and time pickers' flows: a time picked before any day (todo 3111). `today` is
//! the 9th and `min` the 10th at 09:00, so any time on `today` lands on `min`. Disabled pickers
//! (todo 3881) on `/date/*`.

use anyhow::{Result, ensure};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::{keyboard, pointer};
use e2e::wait::PageBounded;
use e2e::{Fixture, Viewport, wait};

const INPUT: &str = "input[data-controlled]";
const SECOND_TAB: &str = "[role=dialog] [role=tablist] [role=tab]:nth-child(2)";
const HOURS: &str = "[role=dialog] [data-column='Hours']";

/// Opens the field, goes to its second tab (the time, or the start's time), and turns the hours.
async fn pick_a_time_first(route: &str) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    let page = &fixture.page;

    pointer::click(page, INPUT).await.unwrap();
    wait::for_js_true(page, &selector_exists(SECOND_TAB), "the dropdown's tabs")
        .await
        .unwrap();
    pointer::click(page, SECOND_TAB).await.unwrap();
    wait::for_js_true(page, &selector_exists(HOURS), "the clock's hours")
        .await
        .unwrap();
    page.evaluate_bounded(format!("document.querySelector({HOURS:?}).focus()"))
        .await
        .unwrap();
    keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
    fixture
}

fn selector_exists(selector: &str) -> String {
    format!("!!document.querySelector({selector:?})")
}

async fn readout_is(page: &chromiumoxide::Page, text: &str, what: &str) {
    let check = format!("document.querySelector('#readout').textContent.trim() === {text:?}");
    wait::for_js_true(page, &check, what).await.unwrap();
}

/// A time turned on the clock before any day was picked lands on `today`, below `min`: it is
/// pulled up to `min`, not emitted below it.
#[test]
fn a_time_picked_before_any_day_is_pulled_into_min() {
    block_on(async {
        let fixture = pick_a_time_first("/chrono-field/time-first").await;

        readout_is(
            &fixture.page,
            "2026-10-10 09:00:00",
            "the time pulled up to min",
        )
        .await;

        fixture.console.assert_clean("a time before a day").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The same for the start of a range of moments. A field holds a half range in its dropdown
/// (todo 4186), so its tabs show the start.
#[test]
fn a_start_time_picked_before_any_day_is_pulled_into_min() {
    block_on(async {
        let fixture = pick_a_time_first("/chrono-field/time-first-range").await;
        let page = &fixture.page;
        let tab = |n: u8| {
            format!(
                "document.querySelector('[role=dialog] [role=tablist] [role=tab]:nth-child({n})').textContent"
            )
        };
        let pulled = format!(
            "{}.includes('10 Oct') && {}.includes('9:00 AM')",
            tab(1),
            tab(2)
        );
        wait::for_js_true(page, &pulled, "the start time pulled up to min")
            .await
            .unwrap();
        readout_is(page, "", "nothing emitted for a half range").await;

        fixture
            .console
            .assert_clean("a start time before a day")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3881: the control each picker's test focuses: a day, a month or year cell, a column,
/// the face, a step tab.
const CONTROLS: [&str; 10] = [
    "#date [data-slot='day'][tabindex='0']",
    "#range [data-slot='day'][tabindex='0']",
    "#month [data-slot='cell'][tabindex='0']",
    "#year [data-slot='cell'][tabindex='0']",
    "#time [data-column='Hours']",
    "#analog [data-slot='face']",
    "#duration [data-column='Hours']",
    "#datetime [data-slot='day'][tabindex='0']",
    "#datetime [role='tab'][aria-selected='true']",
    "#moments [role='tab'][aria-selected='true']",
];

/// Todo 3881: a focused picker control that turns disabled keeps its focus, says
/// `aria-disabled`, leaves the tab order and swallows keys and presses.
#[test]
fn a_picker_that_becomes_disabled_keeps_focus_and_takes_no_pick() {
    stays_put("/date/disabled");
}

/// The same when a `Fieldset` around the pickers turns disabled.
#[test]
fn a_picker_whose_fieldset_becomes_disabled_keeps_focus_and_takes_no_pick() {
    stays_put("/date/disabled-fieldset");
}

fn stays_put(route: &str) {
    block_on(async {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#moments").await.unwrap();
        // `click()` by script: the flip moves no focus.
        let flip = "document.querySelector('#flip').click()";
        let probe = "document.querySelector('[data-probe]')";

        for control in CONTROLS {
            page.evaluate_bounded(format!(
                "(() => {{ const c = document.querySelector({control:?}); c.dataset.probe = ''; c.focus(); }})()"
            ))
            .await
            .unwrap();
            page.evaluate_bounded(flip).await.unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "(() => {{ const c = {probe}; return document.activeElement === c \
                     && c.getAttribute('aria-disabled') === 'true' && c.tabIndex === -1; }})()"
                ),
                &format!("focus kept on the soft-disabled {control}"),
            )
            .await
            .unwrap();

            for key in [
                keyboard::ENTER,
                keyboard::SPACE,
                keyboard::ARROW_UP,
                keyboard::ARROW_RIGHT,
            ] {
                keyboard::press(page, key).await.unwrap();
            }
            pointer::click(page, "[data-probe]").await.unwrap();
            let kept: bool = page
                .evaluate_bounded(format!(
                    "document.activeElement === {probe} \
                     && document.querySelector('#changes').dataset.changes === '0'"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert!(kept, "the disabled {control} lost focus or took a pick");

            page.evaluate_bounded(flip).await.unwrap();
            wait::for_js_true(
                page,
                // A tab always says `aria-disabled`, `false` when enabled.
                &format!("{probe}.getAttribute('aria-disabled') !== 'true'"),
                &format!("{control} enabled again"),
            )
            .await
            .unwrap();
            page.evaluate_bounded(format!("delete {probe}.dataset.probe"))
                .await
                .unwrap();
        }

        // Enabled, the same keys pick: the counter is live.
        page.evaluate_bounded("document.querySelector(\"#time [data-column='Hours']\").focus()")
            .await
            .unwrap();
        keyboard::press(page, keyboard::ARROW_UP).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#changes').dataset.changes !== '0'",
            "an enabled pick",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("disabled pickers").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3886: a column and the face drag on touch (`touch-action: none`) only while enabled, so
/// a touch on a disabled one scrolls the page.
#[test]
fn a_disabled_column_and_face_let_a_touch_scroll() {
    block_on(async {
        let fixture = Fixture::open("/date/disabled", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#moments").await.unwrap();
        let touch = |control: &str| {
            format!(
                "(() => {{ const c = document.querySelector({control:?}); \
                 return c.getAttribute('aria-disabled') === 'true' ? 'auto' : 'none'; }})() \
                 === getComputedStyle(document.querySelector({control:?})).touchAction"
            )
        };
        for round in 0..2 {
            for control in [
                "#time [data-column='Hours']",
                "#duration [data-column='Hours']",
                "#analog [data-slot='face']",
            ] {
                wait::for_js_true(page, &touch(control), &format!("{control} touch-action"))
                    .await
                    .unwrap();
            }
            if round == 0 {
                let was = "document.querySelector(\"#time [data-column='Hours']\").getAttribute('aria-disabled') === 'true'";
                let before: bool = page
                    .evaluate_bounded(was)
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                page.evaluate_bounded("document.querySelector('#flip').click()")
                    .await
                    .unwrap();
                wait::for_js_true(page, &format!("({was}) !== {before}"), "the flip")
                    .await
                    .unwrap();
            }
        }
        fixture.close().await.unwrap();
    });
}

/// Todo 3881: a disabled `Fieldset` posts no picker value; only the field outside it is sent.
#[test]
fn a_disabled_fieldset_sends_no_picker_value() {
    block_on(async {
        let fixture = Fixture::open("/date/fieldset-submit", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_visible(page, "#submit").await.unwrap();
        page.evaluate_bounded(
            "document.querySelector('#sent').addEventListener('submit', (event) => { \
             event.preventDefault(); \
             window.sent = [...new FormData(event.target, event.submitter).keys()]; })",
        )
        .await
        .unwrap();
        pointer::click(page, "#submit").await.unwrap();
        wait::for_js_true(page, "Array.isArray(window.sent)", "the submit")
            .await
            .unwrap();
        let sent: Vec<String> = page
            .evaluate_bounded("window.sent")
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            sent,
            ["outside"],
            "a picker of the disabled fieldset was sent"
        );
        fixture.console.assert_clean("fieldset submit").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3883: a soft-disabled control of each picker: a day, a cell, a column, the face, a
/// step tab.
const PRESSED: [&str; 9] = [
    "#date [data-slot='day'][data-date='2026-10-21']",
    "#range [data-slot='day'][data-date='2026-10-21']",
    "#month [data-slot='cell'][data-date='2026-10-01']",
    "#year [data-slot='cell'][data-date='2026-01-01']",
    "#time [data-column='Hours']",
    "#analog [data-slot='face']",
    "#duration [data-column='Hours']",
    "#datetime [role='tab'][aria-selected='true']",
    "#moments [role='tab'][aria-selected='true']",
];

/// A press on a disabled picker's control focuses it, as on a disabled `Button` (3838), and
/// picks nothing.
async fn press_disabled_pickers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#flip").await?;
    eventually(d, "the pickers disabled", async |d| {
        Ok(d.attr(PRESSED[4], "aria-disabled").await?.as_deref() == Some("true"))
    })
    .await?;
    for control in PRESSED {
        d.click(control).await?;
        eventually_focused(d, control, &format!("a press on {control}")).await?;
    }
    d.flush().await?;
    let changes = d.attr("#changes", "data-changes").await?;
    ensure!(
        changes.as_deref() == Some("0"),
        "a disabled picker took {changes:?} picks"
    );
    Ok(())
}

e2e::scenario!(
    a_press_on_a_disabled_picker_focuses_it_and_picks_nothing,
    "/date/disabled",
    press_disabled_pickers,
    macos: skip("upstream: WebKit does not focus a clicked button"),
    ios: skip("upstream: WebKit does not focus a clicked button")
);

/// `/date/extras`: a picker's value as its readout's `data-value` says it.
async fn value_is<D: Driver>(d: &mut D, id: &str, value: &str, what: &str) -> Result<()> {
    let readout = format!("#{id}-value");
    eventually(d, what, async |d| {
        Ok(d.attr(&readout, "data-value").await?.as_deref() == Some(value))
    })
    .await
}

const NEXT_MONTH: &str = "#presets [data-slot='preset']:nth-child(2)";

/// Todo 3988: a preset picks its day, pages to its month and is pressed. Todo 4003: every week
/// is headed by its ISO number, October 1 2026 being in week 40.
async fn presets_and_week_numbers<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "week 40 heads October", async |d| {
        Ok(d.text("#presets [role='rowheader']").await?.trim() == "40")
    })
    .await?;
    d.click(NEXT_MONTH).await?;
    value_is(d, "presets", "2026-11-01", "the preset's day").await?;
    eventually(d, "the preset pressed, November shown", async |d| {
        Ok(
            d.attr(NEXT_MONTH, "aria-pressed").await?.as_deref() == Some("true")
                && d.exists("#presets [data-date='2026-11-30']:not([data-outside])")
                    .await?,
        )
    })
    .await
}

e2e::scenario!(
    a_preset_picks_its_day_and_weeks_show_their_numbers,
    "/date/extras",
    presets_and_week_numbers
);

/// Todo 3988: in a field's dropdown a preset commits its day and closes, like a picked day.
async fn field_preset<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("#field input[data-controlled]").await?;
    let preset = "[role='dialog'] [data-slot='preset']";
    eventually(d, "the dropdown's presets", async |d| {
        d.exists(preset).await
    })
    .await?;
    d.click(preset).await?;
    value_is(d, "field", "2026-10-01", "the preset's day").await?;
    eventually(d, "the dropdown closed", async |d| {
        Ok(!d.exists("[role='dialog']").await?)
    })
    .await
}

e2e::scenario!(
    a_field_preset_commits_and_closes,
    "/date/extras",
    field_preset
);

/// Todo 4003: `open_to: Year` opens on the years; a year, then a month, lead down to its days.
async fn open_to_years<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let year = "#year-first [data-slot='cell'][data-date='2027-01-01']";
    eventually(d, "the years", async |d| d.exists(year).await).await?;
    ensure!(
        !d.exists("#year-first [data-slot='day']").await?,
        "days before a year is picked"
    );
    d.click(year).await?;
    let month = "#year-first [data-slot='cell'][data-date='2027-03-01']";
    eventually(d, "the months of 2027", async |d| d.exists(month).await).await?;
    d.click(month).await?;
    let day = "#year-first [data-slot='day'][data-date='2027-03-15']";
    eventually(d, "the days of March", async |d| d.exists(day).await).await?;
    d.click(day).await?;
    value_is(d, "year-first", "2027-03-15", "the picked day").await
}

e2e::scenario!(open_to_starts_from_the_years, "/date/extras", open_to_years);

/// Todo 4003: the hours step by `hour_step`, the seconds by `second_step`, both from zero.
async fn time_steps<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#steps [data-column='Hours']").await?;
    d.press(keyboard::ARROW_UP).await?;
    value_is(d, "steps", "16:00:30", "two hours on").await?;
    d.focus("#steps [data-column='Seconds']").await?;
    d.press(keyboard::ARROW_UP).await?;
    value_is(d, "steps", "16:00:45", "onto the next quarter minute").await
}

e2e::scenario!(
    the_hour_and_second_steps_turn_their_columns,
    "/date/extras",
    time_steps
);
