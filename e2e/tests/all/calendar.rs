//! `ChronoPicker`'s day grid keys, and a month change that keeps focus in the grid (406).
//! `/calendar` picks 2026-03-18, also `today`, so nothing depends on the clock.

use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use e2e::browser::block_on;
use e2e::passes::keyboard::{self, Key};
use e2e::passes::pointer;
use e2e::{Fixture, Suite, Viewport, wait};

const STOP: &str = "[role=grid] [data-date='2026-03-18']:not([data-outside])";

#[test]
fn it_meets_the_baseline() {
    Suite::new("calendar", "/calendar")
        .focusable(STOP)
        .targets("[role=grid] [data-slot=day]")
        .run();
}

/// Todo 745: a day before `min` is `GrayText` in forced colours, and draws no box.
#[test]
fn a_disabled_day_grays_out_in_forced_colours() {
    const CLOSED: &str = r#"[role=grid] [data-date="2026-03-09"]:not([data-outside])"#;
    block_on(async {
        let fixture = Fixture::open("/calendar/limited", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::browser::force_colours(page).await.unwrap();
        crate::button::assert_gray_in_forced_colours(page, CLOSED).await;
        let [border, canvas]: [String; 2] = page
            .evaluate(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.color = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).color; probe.remove(); \
                 return [getComputedStyle(document.querySelector({CLOSED:?})).borderTopColor, canvas]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(border, canvas, "a disabled day draws a box");
        fixture.close().await.unwrap();
    });
}

/// Every key the grid owns, read back from where focus lands. Focus left on the old cell
/// or on the neighbour month's copy (`data-outside`) fails the step that caused it.
#[test]
fn the_keys_move_focus_through_the_grid() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/calendar", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, STOP, 10).await.unwrap();
            expect_focus(page, "2026-03-18", "March 2026", "Tab into the grid", at).await;

            let steps: &[(Key, &str, &str, &str)] = &[
                (
                    keyboard::ARROW_RIGHT,
                    "2026-03-19",
                    "March 2026",
                    "ArrowRight",
                ),
                (
                    keyboard::ARROW_LEFT,
                    "2026-03-18",
                    "March 2026",
                    "ArrowLeft",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "2026-03-25",
                    "March 2026",
                    "ArrowDown",
                ),
                (keyboard::ARROW_UP, "2026-03-18", "March 2026", "ArrowUp"),
                (keyboard::HOME, "2026-03-15", "March 2026", "Home (Sunday)"),
                (keyboard::END, "2026-03-21", "March 2026", "End (Saturday)"),
                (keyboard::PAGE_DOWN, "2026-04-21", "April 2026", "PageDown"),
                (keyboard::PAGE_UP, "2026-03-21", "March 2026", "PageUp"),
                (
                    keyboard::PAGE_UP,
                    "2026-02-21",
                    "February 2026",
                    "PageUp again",
                ),
                (
                    keyboard::ARROW_DOWN,
                    "2026-02-28",
                    "February 2026",
                    "ArrowDown in February",
                ),
                // Mar 7 is drawn in February's grid as an outside day; the
                // step must page to March and focus March's own cell.
                (
                    keyboard::ARROW_DOWN,
                    "2026-03-07",
                    "March 2026",
                    "ArrowDown off the month",
                ),
                (
                    keyboard::HOME,
                    "2026-03-01",
                    "March 2026",
                    "Home to the 1st",
                ),
                (
                    keyboard::ARROW_LEFT,
                    "2026-02-28",
                    "February 2026",
                    "ArrowLeft off the month",
                ),
            ];
            for (key, date, month, what) in steps {
                keyboard::press(page, *key).await.unwrap();
                expect_focus(page, date, month, what, at).await;
            }

            keyboard::press_shift(page, keyboard::PAGE_DOWN)
                .await
                .unwrap();
            expect_focus(
                page,
                "2027-02-28",
                "February 2027",
                "Shift+PageDown (a year)",
                at,
            )
            .await;

            fixture
                .console
                .assert_clean(&format!("the calendar keys at {at}"))
                .unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// The page buttons read their target on click, so each click pages from the
/// month shown, also after a pick (todo 29: they skip the re-render).
#[test]
fn the_page_buttons_follow_the_month_shown() {
    block_on(async {
        let viewport = Viewport::ALL[0];
        let fixture = Fixture::open("/calendar", viewport).await.unwrap();
        let page = &fixture.page;
        let clicks: &[(&str, &str)] = &[
            ("[aria-label='Next month']", "April 2026"),
            ("[aria-label='Next month']", "May 2026"),
            ("[aria-label='Previous month']", "April 2026"),
            ("[role=grid] [data-date='2026-04-10']", "April 2026"),
            ("[aria-label='Next month']", "May 2026"),
            ("[aria-label='Previous month']", "April 2026"),
        ];
        for (target, month) in clicks {
            pointer::click(page, target).await.unwrap();
            let check = format!(
                "document.querySelector('[role=grid]').getAttribute('aria-label') === {month:?}"
            );
            wait::for_js_true(page, &check, &format!("{target} to show {month}"))
                .await
                .unwrap();
        }
        wait::for_js_true(
            page,
            "document.querySelector(\"[data-date='2026-04-10']\").hasAttribute('data-selected')",
            "the picked day to stay marked",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the page buttons").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Enter on Next pages to `max`'s month and disables the button: focus stays on it,
/// and a second Enter pages nowhere (2301).
#[test]
fn paging_to_max_keeps_focus_on_the_page_button() {
    const NEXT: &str = "[aria-label='Next month']";
    block_on(async {
        let fixture = Fixture::open("/calendar/limited", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        keyboard::tab_to(page, NEXT, 6).await.unwrap();
        for what in ["Enter on Next", "Enter on the disabled Next"] {
            keyboard::press(page, keyboard::ENTER).await.unwrap();
            wait::for_js_true(
                page,
                "document.querySelector('[role=grid]').getAttribute('aria-label') === 'April 2026'",
                &format!("{what} to show April 2026"),
            )
            .await
            .unwrap();
            wait::for_js_true(
                page,
                &format!(
                    "document.activeElement?.matches({NEXT:?}) \
                     && document.activeElement.getAttribute('aria-disabled') === 'true'"
                ),
                &format!("focus on the disabled Next after {what}"),
            )
            .await
            .unwrap();
        }
        fixture.console.assert_clean("paging to max").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A range picked, previewed and paged: the days are drawn in place on a
/// page (todo 29), so their marks must follow the dates, not the columns.
#[test]
fn a_range_keeps_its_marks_across_pages() {
    block_on(async {
        let fixture = Fixture::open("/calendar/range", Viewport::ALL[0])
            .await
            .unwrap();
        let page = &fixture.page;
        let day = |month: &str, date: &str| {
            format!("[role=grid][aria-label='{month}'] [data-date='{date}']:not([data-outside])")
        };
        let has = |month: &str, date: &str, mark: &str| {
            format!(
                "(() => {{ const el = document.querySelector({:?}); return !!el && el.hasAttribute('{mark}'); }})()",
                day(month, date)
            )
        };
        let expect = |check: String, what: &'static str| async move {
            wait::for_js_true(page, &check, what).await.unwrap();
        };

        pointer::click(page, &day("March 2026", "2026-03-10"))
            .await
            .unwrap();
        pointer::hover(page, &day("March 2026", "2026-03-14"))
            .await
            .unwrap();
        expect(
            has("March 2026", "2026-03-12", "data-in-range"),
            "the hover to preview the range",
        )
        .await;
        pointer::click(page, &day("March 2026", "2026-03-14"))
            .await
            .unwrap();
        expect(
            has("March 2026", "2026-03-14", "data-selected"),
            "the end to be picked",
        )
        .await;

        pointer::click(page, "[aria-label='Next month']")
            .await
            .unwrap();
        expect(
            has("April 2026", "2026-04-12", "aria-label"),
            "the grids to page forward",
        )
        .await;
        expect(
            format!(
                "!document.querySelector('[data-in-range]:not([data-outside])') || {}",
                has("March 2026", "2026-03-12", "data-in-range")
            ),
            "no April day to keep March's marks",
        )
        .await;
        for _ in 0..2 {
            pointer::click(page, "[aria-label='Previous month']")
                .await
                .unwrap();
        }
        expect(
            has("March 2026", "2026-03-12", "data-in-range"),
            "March's marks to return on its new column",
        )
        .await;
        expect(
            has("March 2026", "2026-03-10", "data-selected"),
            "the start to stay picked",
        )
        .await;
        expect(
            "document.querySelectorAll('[data-in-range]:not([data-outside])').length === 3"
                .to_string(),
            "only the three days inside the range to be marked",
        )
        .await;
        fixture.console.assert_clean("the range picker").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1553: the keyboard previews a waiting range as the mouse does, and every
/// day of a picked range is `aria-selected`, a preview's none past the start.
#[test]
fn the_keyboard_previews_a_range_and_its_days_are_selected() {
    block_on(async {
        let fixture = Fixture::open("/calendar/range", Viewport::ALL[0])
            .await
            .unwrap();
        let page = &fixture.page;
        let day = |date: &str| {
            format!("[role=grid][aria-label='March 2026'] [data-date='{date}']:not([data-outside])")
        };
        let selected = |date: &str| {
            format!(
                "document.querySelector({:?}).parentElement.getAttribute('aria-selected')",
                day(date)
            )
        };

        pointer::click(page, &day("2026-03-10")).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "document.querySelector({:?}).hasAttribute('data-in-range')",
                day("2026-03-11")
            ),
            "the focused day to preview the range",
        )
        .await
        .unwrap();
        let preview: [String; 3] = page
            .evaluate(format!(
                "[{}, {}, {}]",
                selected("2026-03-10"),
                selected("2026-03-11"),
                selected("2026-03-12")
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(
            preview,
            ["true", "false", "false"],
            "a preview is not selected"
        );

        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait::for_js_true(
            page,
            &format!(
                "[{}, {}, {}].every(s => s === 'true')",
                selected("2026-03-10"),
                selected("2026-03-11"),
                selected("2026-03-12")
            ),
            "every day of the picked range to be selected",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("the keyboard range").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Climbing by the titles keeps focus in the calendar, and a year paged by
/// key climbs back down to the month it came from, not January (todo 26).
#[test]
fn the_levels_keep_focus_and_the_month() {
    block_on(async {
        let fixture = Fixture::open("/calendar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let focused = |what: &'static str, check: &'static str| async move {
            let js = format!(
                "(() => {{ const el = document.activeElement; return !!el && ({check}); }})()"
            );
            wait::for_js_true(page, &js, what).await.unwrap();
        };

        keyboard::tab_to(page, "[data-slot=title]", 10)
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focused(
            "the month view's title to hold focus",
            "el.dataset.slot === 'title' && el.textContent === '2026'",
        )
        .await;
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focused(
            "the year view's 2026 cell to hold focus",
            "el.dataset.date === '2026-01-01'",
        )
        .await;
        keyboard::press(page, keyboard::ARROW_RIGHT).await.unwrap();
        focused(
            "ArrowRight to focus 2027",
            "el.dataset.date === '2027-01-01'",
        )
        .await;
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        focused(
            "picking 2027 to focus March 2027",
            "el.dataset.date === '2027-03-01'",
        )
        .await;

        fixture.console.assert_clean("the level climb").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn the_month_view_meets_the_baseline() {
    Suite::new("calendar-month", "/calendar/month")
        .focusable("[role=grid] [data-date='2026-03-01']")
        .targets("[role=grid] [data-slot=cell]")
        .run();
}

/// Todo 2308: the year level, gridcells around buttons and a plain-text title (2307).
#[test]
fn the_year_view_meets_the_baseline() {
    Suite::new("calendar-year", "/calendar/year")
        .focusable("[role=grid] [data-date='2026-01-01']")
        .targets("[role=grid] [data-slot=cell]")
        .run();
}

/// The other variants (todo 1773).
#[test]
fn the_range_and_limited_calendars_meet_the_baseline() {
    for (name, route) in [
        ("calendar-range", "/calendar/range"),
        ("calendar-limited", "/calendar/limited"),
    ] {
        Suite::new(name, route).focusable(STOP).run();
    }
}

#[test]
fn the_mini_calendar_meets_the_baseline() {
    Suite::new("calendar-mini", "/calendar/mini")
        .focusable(STOP)
        .run();
}

/// Rows of three gridcells, one of them selected, under a labelled grid.
const GRID_SHAPE: &str = "(() => { const grid = document.querySelector('[role=grid]'); \
     const rows = [...grid.children]; \
     const cells = grid.querySelectorAll('[role=gridcell]'); \
     return rows.length === 4 && rows.every(r => r.getAttribute('role') === 'row' \
       && [...r.children].filter(c => c.getAttribute('role') === 'gridcell').length === 3) \
       && cells.length === 12 \
       && [...cells].every(c => ['true', 'false'].includes(c.getAttribute('aria-selected'))) \
       && grid.querySelectorAll('[aria-selected=true]').length === 1; })()";

/// The month and year views are APG grids like the day view: rows of cells,
/// arrows, Home/End and PageUp/PageDown, a step off the page pages it (473).
#[test]
fn the_month_and_year_views_are_grids() {
    block_on(async {
        let fixture = Fixture::open("/calendar/month", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(page, GRID_SHAPE, "the month view's grid shape")
            .await
            .unwrap();

        keyboard::tab_to(page, "[role=grid] [data-date='2026-03-01']", 10)
            .await
            .unwrap();
        let steps: &[(Key, &str, &str, &str)] = &[
            (keyboard::ARROW_RIGHT, "2026-04-01", "2026", "ArrowRight"),
            (keyboard::ARROW_DOWN, "2026-07-01", "2026", "ArrowDown"),
            (keyboard::END, "2026-09-01", "2026", "End"),
            (keyboard::HOME, "2026-07-01", "2026", "Home"),
            (keyboard::ARROW_UP, "2026-04-01", "2026", "ArrowUp"),
            (keyboard::PAGE_DOWN, "2027-04-01", "2027", "PageDown"),
            (keyboard::PAGE_UP, "2026-04-01", "2026", "PageUp"),
            (keyboard::ARROW_UP, "2026-01-01", "2026", "ArrowUp"),
            (
                keyboard::ARROW_LEFT,
                "2025-12-01",
                "2025",
                "ArrowLeft off the year",
            ),
        ];
        for (key, date, grid, what) in steps {
            keyboard::press(page, *key).await.unwrap();
            expect_focus(page, date, grid, what, "desktop").await;
        }

        pointer::click(page, "[data-slot=title]").await.unwrap();
        wait::for_js_true(page, GRID_SHAPE, "the year view's grid shape")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('[role=grid]').getAttribute('aria-label') === '2020 – 2029' \
             && document.querySelector(\"[data-date='2026-01-01']\").closest('[role=gridcell]').getAttribute('aria-selected') === 'true' \
             && document.querySelector('[data-slot=title]').tagName === 'DIV'",
            "the year view to label its decade and select 2026",
        )
        .await
        .unwrap();
        // The title hands focus to the year the view stood on.
        expect_focus(page, "2025-01-01", "2020 – 2029", "the climb", "desktop").await;
        // Rows run 2019-2021, ..., 2028-2030; 2030 belongs to the next decade.
        let steps: &[(Key, &str, &str, &str)] = &[
            (
                keyboard::ARROW_DOWN,
                "2028-01-01",
                "2020 – 2029",
                "ArrowDown",
            ),
            (
                keyboard::END,
                "2030-01-01",
                "2030 – 2039",
                "End onto the next decade",
            ),
            (
                keyboard::ARROW_DOWN,
                "2033-01-01",
                "2030 – 2039",
                "ArrowDown",
            ),
            (keyboard::PAGE_UP, "2023-01-01", "2020 – 2029", "PageUp"),
        ];
        for (key, date, grid, what) in steps {
            keyboard::press(page, *key).await.unwrap();
            expect_focus(page, date, grid, what, "desktop").await;
        }

        fixture
            .console
            .assert_clean("the month and year grids")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// Alt+ArrowLeft is Back, Ctrl+PageUp switches tabs: the day grid, the month
/// grid and the mini strip leave a browser chord alone (todo 562).
#[test]
fn modifier_chords_are_left_to_the_browser() {
    const FOCUS: &str = "[document.activeElement.getAttribute('data-date'), \
         document.querySelector('[role=grid]')?.getAttribute('aria-label')]";
    let keys = [
        keyboard::ARROW_LEFT,
        keyboard::ARROW_RIGHT,
        keyboard::ARROW_UP,
        keyboard::ARROW_DOWN,
        keyboard::PAGE_UP,
        keyboard::PAGE_DOWN,
        keyboard::HOME,
        keyboard::END,
    ];
    // The three pages at once, as a `Suite` runs its pages.
    block_on(futures::future::join_all(
        [
            ("/calendar", STOP),
            ("/calendar/month", "[role=grid] [data-date='2026-03-01']"),
            ("/calendar/mini", "[data-date='2026-03-18']"),
        ]
        .map(|(path, stop)| async move {
            let fixture = Fixture::open(path, Viewport::Desktop).await.unwrap();
            let page = &fixture.page;
            keyboard::tab_to(page, stop, 10).await.unwrap();
            keyboard::assert_chords_ignored(page, &keys, FOCUS)
                .await
                .unwrap_or_else(|e| panic!("{path}: {e}"));
            fixture.close().await.unwrap();
        }),
    ));
}

/// Waits until focus is on `date`'s own cell in the grid titled `month`.
async fn expect_focus(page: &chromiumoxide::Page, date: &str, month: &str, what: &str, at: &str) {
    let check = format!(
        "(() => {{ const el = document.activeElement; \
         const grid = el && el.closest('[role=grid]'); \
         return !!grid && grid.getAttribute('aria-label') === {month:?} \
           && el.getAttribute('data-date') === {date:?} && !el.hasAttribute('data-outside'); }})()"
    );
    if let Err(e) =
        wait::for_js_true(page, &check, &format!("{what} to focus {date} in {month}")).await
    {
        let actual: String = page
            .evaluate(
                "(() => { const el = document.activeElement; const grid = el && el.closest('[role=grid]'); \
                 return `${el && el.getAttribute('data-date')} in ${grid && grid.getAttribute('aria-label')}`; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        panic!("at {at}: {e}; focus is on {actual}");
    }
}

/// Forced colours turn every day's transparent border into a visible one, so
/// today's did not stand out, and the range tint went `Canvas` (todo 537).
#[test]
fn today_and_the_range_show_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/calendar/range", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        e2e::browser::force_colours(page).await.unwrap();
        let day = |date: &str| format!("[role=grid] [data-date='{date}']:not([data-outside])");
        pointer::click(page, &day("2026-03-10")).await.unwrap();
        pointer::click(page, &day("2026-03-13")).await.unwrap();
        wait::for_visible(page, "[data-in-range]").await.unwrap();

        // `Canvas` as this page resolves it; a border or fill that colour is none.
        let colours: [String; 5] = page
            .evaluate(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 const at = s => getComputedStyle(document.querySelector(s)); \
                 return [canvas, at({:?}).borderTopColor, at({:?}).borderTopColor, \
                 at({range:?}).borderTopColor, at({range:?}).backgroundColor]; }})()",
                day("2026-03-05"),
                day("2026-03-18"),
                range = day("2026-03-11"),
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let [canvas, plain, today, range_border, range_fill] = colours;
        assert_eq!(plain, canvas, "a plain day draws a border");
        assert_ne!(today, canvas, "today's border is not drawn");
        assert!(
            range_border != canvas || range_fill != canvas,
            "an in-range day is marked neither by border ({range_border}) nor fill ({range_fill})"
        );
        fixture.close().await.unwrap();
    });
}

/// Forced colours paint the picked day's fill `Canvas`, like every other day's,
/// so only `aria-selected` told it apart (todo 524).
#[test]
fn the_picked_day_shows_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/calendar", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        e2e::browser::force_colours(page).await.unwrap();
        // The page itself is `Canvas`; a fill the same colour is no fill.
        let [picked, canvas]: [String; 2] = page
            .evaluate(format!(
                "(() => {{ const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return [getComputedStyle(document.querySelector({STOP:?})).backgroundColor, \
                 canvas]; }})()"
            ))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_ne!(picked, canvas, "the picked day's fill is the page's own");
        fixture.close().await.unwrap();
    });
}

/// Todo 1552: the mini strip's seven days fit a 320px page; the days narrow instead.
#[test]
fn the_mini_strip_fits_320px() {
    block_on(async {
        let fixture = Fixture::open("/calendar/mini", Viewport::Mobile)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetDeviceMetricsOverrideParams::new(320, 800, 1.0, true))
            .await
            .unwrap();
        wait::for_visible(page, "[data-slot=strip]").await.unwrap();
        // Against 320, not `innerWidth`: a mobile layout viewport widens to its content.
        let [overflow, right, days]: [f64; 3] = page
            .evaluate(
                "(() => { const strip = document.querySelector('[data-slot=strip]'); \
                 return [document.documentElement.scrollWidth - 320, \
                   strip.getBoundingClientRect().right - 320, \
                   strip.querySelectorAll('[data-slot=day]').length]; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(days, 7.0);
        assert!(overflow <= 0.0, "the page scrolls sideways by {overflow}px");
        assert!(right <= 0.0, "the strip ends {right}px past the viewport");
        fixture.close().await.unwrap();
    });
}
