//! Render counts per interaction in a browser (821), each scope held to a budget once
//! rendering stops. The probe proves the counter first (463).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, clock, wait};

type Renders = BTreeMap<String, u32>;

async fn reset(page: &Page) {
    page.evaluate("window.__lsxRendersReset()").await.unwrap();
}

async fn read(page: &Page) -> Renders {
    let json: String = page
        .evaluate("window.__lsxRenders()")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    serde_json::from_str(&json).unwrap()
}

/// Finite, time-driven animations and transitions still running: an exit renders once it ends.
const ANIMATING: &str = "document.getAnimations().some((a) => a.playState === 'running' \
    && a.effect?.getComputedTiming().iterations !== Infinity \
    && !(typeof ScrollTimeline !== 'undefined' && a.timeline instanceof ScrollTimeline))";

/// The counts once a drawn frame changed none and no animation runs; a count still
/// growing after 6 s is a runaway.
async fn settled(page: &Page, what: &str) -> Renders {
    let started = Instant::now();
    let mut last = read(page).await;
    loop {
        // Resize reports, and the bars' re-measure on them, land with a frame (todo 1955).
        clock::frame(page).await.unwrap();
        let now = read(page).await;
        let animating: bool = page
            .evaluate(ANIMATING)
            .await
            .unwrap()
            .into_value()
            .unwrap();
        if now == last && !animating {
            return now;
        }
        assert!(
            started.elapsed() < Duration::from_secs(6),
            "{what}: still rendering after 3 s, a runaway: {now:?}"
        );
        last = now;
    }
}

/// Renders of every scope named `scope`, whatever its module path.
fn count(renders: &Renders, scope: &str) -> u32 {
    renders
        .iter()
        .filter(|(name, _)| name.rsplit("::").next() == Some(scope))
        .map(|(_, count)| count)
        .sum()
}

/// Every scope that rendered is in `budget` and within it. Budgets are the deterministic
/// counts of 2026-09-27: growth needs a look, a lower count passes.
fn assert_within(renders: &Renders, budget: &[(&str, u32)], what: &str) {
    let mut short = Renders::new();
    for (name, rendered) in renders {
        let name = name.split('<').next().unwrap_or(name);
        let scope = name.rsplit("::").next().unwrap_or(name);
        *short.entry(scope.to_string()).or_default() += rendered;
    }
    let mut over = Vec::new();
    for (scope, &rendered) in &short {
        let allowed = budget
            .iter()
            .find(|(listed, _)| *listed == scope)
            .map_or(0, |(_, allowed)| *allowed);
        if rendered > allowed {
            over.push(format!("{scope} {rendered} (budget {allowed})"));
        }
    }
    assert!(
        over.is_empty(),
        "{what}: over budget: {over:?}\nall: {renders:?}"
    );
}

async fn open(route: &str) -> Fixture {
    let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
    // The first render's counts are not an interaction's.
    settled(&fixture.page, route).await;
    reset(&fixture.page).await;
    fixture
}

/// A closure handler prop re-renders its child on every parent render, a `use_callback`
/// one skips; the counter must see that for any budget to mean anything.
#[test]
fn the_counter_sees_a_closure_handler_defeat_memo() {
    block_on(async {
        let fixture = open("/perf/probe").await;
        let page = &fixture.page;
        for n in 1..=5 {
            pointer::click(page, "#rerender").await.unwrap();
            wait::for_js_true(
                page,
                &format!("document.getElementById('rerender').textContent === 'Rerender {n}'"),
                &format!("parent render {n}"),
            )
            .await
            .unwrap();
        }
        let renders = settled(page, "five parent renders").await;
        assert_eq!(count(&renders, "ProbePage"), 5, "{renders:?}");
        assert_eq!(count(&renders, "ClosureChild"), 5, "{renders:?}");
        assert_eq!(count(&renders, "StableChild"), 0, "{renders:?}");
        fixture.console.assert_clean("the render probe").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A keystroke redraws the field's own scope, never its siblings.
#[test]
fn typing_stays_inside_the_field() {
    block_on(async {
        let fixture = open("/perf/typing").await;
        let page = &fixture.page;
        pointer::click(page, "input").await.unwrap();
        settled(page, "focusing the field").await;
        reset(page).await;
        keyboard::type_text(page, "hello").await.unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('typed').dataset.typed === 'hello'",
            "five keystrokes",
        )
        .await
        .unwrap();
        let renders = settled(page, "typing").await;
        // The outer field scope and its live control (todo 29); the shell skips.
        assert_within(
            &renders,
            &[("Typing", 5), ("TextField", 5), ("LiveControl", 5)],
            "five keystrokes",
        );
        fixture.console.assert_clean("typing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Opening a select and picking a row.
#[test]
fn a_select_pick_stays_in_budget() {
    block_on(async {
        let fixture = open("/perf/select").await;
        let page = &fixture.page;
        pointer::click(page, "[role=combobox]").await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        let opened = settled(page, "opening the select").await;
        // Popup passes: mount, the Min width landing, then placing at that width (5c038ff5c, combobox audit C5).
        assert_within(
            &opened,
            &[
                ("Select", 1),
                ("SelectCore", 1),
                ("ComboboxCore", 1),
                ("ComboboxPopup", 3),
                ("ComboboxDropdown", 1),
                ("ComboboxRow", 5),
                ("ComboboxOption", 5),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                // The drawn bar (1010): mount, the measure, and its re-read.
                ("ScrollAreaBars", 3),
                ("Fragment", 3),
                ("PortalOutlet", 3),
                ("StyleOutlet", 1),
            ],
            "opening a select of five",
        );
        reset(page).await;
        page.evaluate(
            "[...document.querySelectorAll('[role=option]')].find((o) => o.textContent.trim() === 'Cherry').setAttribute('data-e2e', 'cherry')",
        )
        .await
        .unwrap();
        pointer::click(page, "[data-e2e=cherry]").await.unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('picked').dataset.picked === 'Some(Cherry)'",
            "the pick",
        )
        .await
        .unwrap();
        let picked = settled(page, "picking").await;
        assert_within(
            &picked,
            &[
                ("SelectPage", 1),
                ("Flex", 1),
                ("Select", 1),
                ("SelectCore", 1),
                ("SelectValue", 1),
                ("ComboboxCore", 1),
                ("PortalOutlet", 1),
                ("StyleOutlet", 1),
                // The bars re-measure once as the list closes.
                ("ScrollAreaBars", 1),
            ],
            "a pick",
        );
        fixture.console.assert_clean("picking").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Picking a row of an open multi-select (todo 876), which stays open.
#[test]
fn a_multi_select_pick_stays_in_budget() {
    block_on(async {
        let fixture = open("/perf/multi-select").await;
        let page = &fixture.page;
        pointer::click(page, "[role=combobox]").await.unwrap();
        wait::for_visible(page, "[role=option]").await.unwrap();
        settled(page, "opening the multi-select").await;
        reset(page).await;
        page.evaluate(
            "[...document.querySelectorAll('[role=option]')].find((o) => o.textContent.trim() === 'Cherry').setAttribute('data-e2e', 'cherry')",
        )
        .await
        .unwrap();
        pointer::click(page, "[data-e2e=cherry]").await.unwrap();
        wait::for_js_true(
            page,
            "document.getElementById('picked').dataset.picked === '[Cherry]'",
            "the pick",
        )
        .await
        .unwrap();
        let picked = settled(page, "picking").await;
        // One pass: before the pick wrote `picked` itself, every scope here ran twice.
        assert_within(
            &picked,
            &[
                ("MultiSelectPage", 1),
                ("Flex", 1),
                ("MultiSelect", 1),
                ("SelectCore", 1),
                ("Chip", 1),
                ("ActionIcon", 1),
                ("Glyph", 1),
                ("VisuallyHidden", 2),
                ("ComboboxCore", 1),
                ("ComboboxPopup", 2),
                ("ComboboxDropdown", 1),
                ("ComboboxRow", 5),
                ("ComboboxOption", 5),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                ("Fragment", 2),
                ("PortalOutlet", 2),
                ("StyleOutlet", 1),
            ],
            "a multi-select pick",
        );
        fixture.console.assert_clean("picking").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The rows a key or an arrow leaves alone skip (todo 2022). A kept row keeps its key, id and
/// pick slot through a filter, so only the highlight's old and new rows redraw (todo 2046).
#[test]
fn a_searchable_select_filter_and_arrow_stay_in_budget() {
    block_on(async {
        let fixture = open("/perf/search-select").await;
        let page = &fixture.page;
        pointer::click(page, "[role=combobox]").await.unwrap();
        wait::for_js_true(
            page,
            "document.activeElement.tagName === 'INPUT'",
            "the search box",
        )
        .await
        .unwrap();
        settled(page, "opening").await;
        reset(page).await;
        keyboard::type_text(page, "1").await.unwrap();
        wait::for_js_true(page, &option_count_is(138), "the filter")
            .await
            .unwrap();
        let filtered = settled(page, "filtering").await;
        assert_within(
            &filtered,
            &[
                ("SelectCore", 1),
                ("ComboboxCore", 1),
                ("ComboboxPopup", 1),
                ("ComboboxDropdown", 1),
                ("ComboboxRow", 2),
                ("ComboboxOption", 2),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                ("ScrollAreaBars", 2),
                ("Fragment", 1),
                ("PortalOutlet", 1),
                ("StyleOutlet", 1),
            ],
            "a key filtering 300 rows to 138",
        );
        reset(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(page, &active_row_is(1), "the arrow")
            .await
            .unwrap();
        let arrowed = settled(page, "arrowing").await;
        assert_within(
            &arrowed,
            &[
                ("SelectCore", 1),
                ("ComboboxCore", 1),
                ("ComboboxPopup", 1),
                ("ComboboxDropdown", 1),
                ("ComboboxRow", 2),
                ("ComboboxOption", 2),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                ("ScrollAreaBars", 2),
                ("Fragment", 1),
                ("PortalOutlet", 1),
                ("StyleOutlet", 1),
            ],
            "an arrow over 138 rows",
        );
        fixture
            .console
            .assert_clean("the searchable select")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// One pass per key: the state's row count, read back through the trigger's ARIA, ran a second.
#[test]
fn an_autocomplete_key_draws_each_row_once() {
    block_on(async {
        let fixture = open("/perf/autocomplete").await;
        let page = &fixture.page;
        pointer::click(page, "input").await.unwrap();
        settled(page, "focusing").await;
        reset(page).await;
        keyboard::type_text(page, "1").await.unwrap();
        wait::for_js_true(page, &option_count_is(19), "the filter")
            .await
            .unwrap();
        let filtered = settled(page, "filtering").await;
        assert_within(
            &filtered,
            &[
                ("TimedAutocompletePage", 1),
                ("Flex", 1),
                ("Autocomplete", 1),
                ("ComboboxCore", 1),
                ("ComboboxPopup", 3),
                ("ComboboxDropdown", 1),
                ("ComboboxRow", 19),
                ("ComboboxOption", 19),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                ("ScrollAreaBars", 3),
                ("Fragment", 3),
                ("PortalOutlet", 3),
                ("StyleOutlet", 1),
            ],
            "a key opening 19 of 100 rows",
        );
        reset(page).await;
        keyboard::press(page, keyboard::ARROW_DOWN).await.unwrap();
        wait::for_js_true(page, &active_row_is(0), "the arrow")
            .await
            .unwrap();
        let arrowed = settled(page, "arrowing").await;
        assert_within(
            &arrowed,
            &[
                ("Autocomplete", 1),
                ("ComboboxCore", 1),
                ("ComboboxPopup", 1),
                ("ComboboxDropdown", 1),
                ("ComboboxRow", 1),
                ("ComboboxOption", 1),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                ("ScrollAreaBars", 2),
                ("Fragment", 1),
                ("PortalOutlet", 1),
                ("StyleOutlet", 1),
            ],
            "an arrow onto the first of 19 rows",
        );
        fixture.console.assert_clean("the autocomplete").unwrap();
        fixture.close().await.unwrap();
    });
}

fn option_count_is(count: usize) -> String {
    format!("document.querySelectorAll('[role=option]').length === {count}")
}

/// The focused control names the `row`th drawn option as active.
fn active_row_is(row: usize) -> String {
    format!(
        "(() => {{ const id = document.activeElement.getAttribute('aria-activedescendant'); \
         const option = document.querySelectorAll('[role=option]')[{row}]; \
         return !!id && !!option && option.id === id; }})()"
    )
}

/// Three page changes.
#[test]
fn paging_stays_in_budget() {
    block_on(async {
        let fixture = open("/perf/pagination").await;
        let page = &fixture.page;
        for n in 2..=4 {
            pointer::click(page, &format!("[aria-label=\"Go to page {n}\"]"))
                .await
                .unwrap();
            wait::for_js_true(
                page,
                &format!("document.getElementById('page').dataset.page === '{n}'"),
                &format!("page {n}"),
            )
            .await
            .unwrap();
        }
        let renders = settled(page, "paging").await;
        // Only the arrow that enables at page 2 redraws.
        assert_within(
            &renders,
            &[
                ("PaginationPage", 3),
                ("Flex", 3),
                ("Pagination", 3),
                ("PaginationArrow", 1),
                ("ActionIcon", 1),
            ],
            "three page changes",
        );
        fixture.console.assert_clean("paging").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Opening a menu of a hundred builds its rows twice, mount and placement; the first
/// focus landing on the first item draws nothing (todo 2070; it was a third pass).
#[test]
fn opening_a_menu_draws_its_rows_twice() {
    block_on(async {
        let fixture = open("/perf/menu-100").await;
        let page = &fixture.page;
        pointer::click(page, "button[aria-haspopup]").await.unwrap();
        wait::for_js_true(page, &focused_nth(MENU_ITEM, 0), "the first focus")
            .await
            .unwrap();
        let renders = settled(page, "opening").await;
        assert_within(
            &renders,
            &[
                ("Menu100Page", 1),
                ("Flex", 1),
                ("Button", 1),
                ("Menu", 1),
                ("MenuLevel", 2),
                ("Fragment", 2),
                ("PortalOutlet", 2),
                ("StyleOutlet", 1),
            ],
            "opening a menu of 100",
        );
        fixture.console.assert_clean("the menu").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Ten scroll steps through a virtualized list, then quiet.
#[test]
fn scrolling_a_virtual_list_stays_in_budget() {
    const AREA: &str = "[...document.querySelectorAll('#pane *')].find((el) => el.scrollHeight > el.clientHeight + 1)";
    block_on(async {
        let fixture = open("/perf/scroll").await;
        let page = &fixture.page;
        for step in 1..=10 {
            page.evaluate(format!("{AREA}.scrollTop = {}", step * 200))
                .await
                .unwrap();
            let row = step * 10 + 5;
            wait::for_js_true(
                page,
                &format!("!!document.querySelector('[data-row=\"{row}\"]')"),
                &format!("row {row} after scroll step {step}"),
            )
            .await
            .unwrap();
        }
        let renders = settled(page, "scrolling").await;
        // No row is a scope of its own: a step redraws the window and the
        // padding box around it, never the area. The drawn thumb rides a scroll timeline (1954).
        assert_within(
            &renders,
            &[
                ("ScrollAreaContent", 10),
                ("Virtualize", 10),
                ("ScrollAreaBars", 0),
            ],
            "ten scroll steps",
        );
        fixture.console.assert_clean("scrolling").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Typing into an Autocomplete of a hundred options: the first key opens the list, the
/// next one and a Backspace keep every row, which must not redraw.
#[test]
fn typing_into_an_autocomplete_stays_in_budget() {
    block_on(async {
        let fixture = open("/perf/autocomplete").await;
        let page = &fixture.page;
        pointer::click(page, "input").await.unwrap();
        settled(page, "focusing the field").await;
        // The scopes a keystroke redraws; `ComboboxRow` passes its row through.
        let keystroke = [
            ("TimedAutocompletePage", 1),
            ("Flex", 1),
            ("Autocomplete", 1),
            ("ComboboxCore", 1),
            ("ComboboxPopup", 1),
            ("ComboboxDropdown", 1),
            ("ComboboxRow", 100),
            ("ScrollArea", 1),
            ("ScrollAreaContent", 1),
            ("ScrollAreaBars", 1),
            ("Fragment", 1),
            ("PortalOutlet", 1),
        ];
        // Opening: two passes of the list, each row drawn once (it was twice, 2026-10-05).
        let opening = [
            ("TimedAutocompletePage", 1),
            ("Flex", 1),
            ("Autocomplete", 2),
            ("ComboboxCore", 2),
            ("ComboboxPopup", 4),
            ("ComboboxDropdown", 2),
            ("ComboboxRow", 200),
            ("AutocompleteRow", 100),
            ("ComboboxOption", 100),
            ("ScrollArea", 2),
            ("ScrollAreaContent", 2),
            ("ScrollAreaBars", 3),
            ("Fragment", 4),
            ("PortalOutlet", 4),
            ("StyleOutlet", 1),
        ];
        for (key, typed) in [("c", "c"), ("i", "ci"), ("", "c")] {
            reset(page).await;
            match key {
                "" => keyboard::press(page, keyboard::BACKSPACE).await.unwrap(),
                key => keyboard::type_text(page, key).await.unwrap(),
            }
            wait::for_js_true(
                page,
                &format!("document.querySelector('input').value === '{typed}'"),
                "the keystroke",
            )
            .await
            .unwrap();
            let renders = settled(page, "typing").await;
            let budget: &[(&str, u32)] = if key == "c" { &opening } else { &keystroke };
            assert_within(&renders, budget, &format!("typing to {typed:?}"));
        }
        fixture.console.assert_clean("typing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A ten-move drag of a slider thumb (todo 1953).
#[test]
fn a_slider_drag_stays_in_budget() {
    const VALUE: &str = "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";
    block_on(async {
        let fixture = open("/perf/slider").await;
        let page = &fixture.page;
        let from = pointer::centre_of(page, "[role=slider]").await.unwrap();
        let to = pointer::Point {
            x: from.x + 100.0,
            y: from.y,
        };
        pointer::drag(page, from, to, 10).await.unwrap();
        wait::for_js_true(page, &format!("{VALUE} !== '50'"), "the drag")
            .await
            .unwrap();
        let renders = settled(page, "dragging").await;
        // Ten moves, the press and the release each redraw the page, the slider, its
        // thumbs and the pinned bubble; the closed tooltip takes the new label too.
        assert_within(
            &renders,
            &[
                ("TimedSliderPage", 12),
                ("Slider", 12),
                ("SliderCore", 10),
                ("SliderBody", 2),
                ("SliderThumbs", 12),
                ("Tooltip", 15),
                ("TooltipBubble", 4),
                ("TooltipPinned", 12),
                ("Fragment", 16),
                ("PortalOutlet", 17),
                ("StyleOutlet", 4),
            ],
            "a ten-move drag",
        );
        fixture.console.assert_clean("dragging").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Opening a palette of sixty actions (todo 1952).
#[test]
fn opening_the_spotlight_stays_in_budget() {
    block_on(async {
        let fixture = open("/perf/spotlight").await;
        let page = &fixture.page;
        pointer::click(page, "#open").await.unwrap();
        wait::for_visible(page, "[role=dialog]").await.unwrap();
        let renders = settled(page, "opening").await;
        // The sixty rows are no scopes of their own: the open is the modal's mount.
        assert_within(
            &renders,
            &[
                ("Button", 1),
                ("Modal", 2),
                ("ModalSlot", 1),
                ("Overlay", 1),
                ("Dialog", 1),
                ("FocusTrap", 2),
                ("Box", 1),
                ("ScrollArea", 1),
                ("ScrollAreaContent", 1),
                ("ScrollAreaBars", 2),
                ("Fragment", 1),
                ("PortalOutlet", 1),
                ("StyleOutlet", 1),
            ],
            "opening the palette",
        );
        fixture.console.assert_clean("opening").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A tab switch builds the panel once: the focus moves redraw only the strip (todo 2023).
#[test]
fn a_tab_switch_builds_the_panel_once() {
    block_on(async {
        let fixture = open("/perf/tabs").await;
        let page = &fixture.page;
        let to = |n| {
            (
                "switch",
                Act::Nth("[role=tab]", n),
                nth_is("[role=tab]", n, "aria-selected"),
            )
        };
        step(page, &to(1)).await.unwrap();
        settled(page, "the first switch").await;
        reset(page).await;
        // From a focused tab: its focusout, the new tab's focus, then the pick.
        step(page, &to(0)).await.unwrap();
        let renders = settled(page, "switching back").await;
        assert_within(
            &renders,
            &[
                ("TabsPage", 1),
                ("Flex", 1),
                ("Tabs", 1),
                ("TabStrip", 3),
                ("Text", 8),
            ],
            "a tab switch",
        );
        fixture.console.assert_clean("switching").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A field's first blur redraws that field alone, not every field of the form (todo 2025).
#[test]
fn a_first_blur_redraws_only_its_field() {
    block_on(async {
        let fixture = open("/perf/big-form").await;
        let page = &fixture.page;
        let toggle = |on: bool| {
            let checked = nth_is("#s3", 0, "aria-checked");
            let done = if on { checked } else { format!("!{checked}") };
            ("toggle", Act::Nth("#s3", 0), done)
        };
        step(page, &toggle(true)).await.unwrap();
        // The second toggle touches the switch; every field used to redraw for it.
        step(page, &toggle(false)).await.unwrap();
        let renders = settled(page, "two toggles").await;
        assert_within(&renders, &[("Switch", 3)], "two toggles of one switch");
        fixture.console.assert_clean("toggling").unwrap();
        fixture.close().await.unwrap();
    });
}

/// A keystroke that changes no toolbar state leaves the toolbar alone; it redrew 8 buttons
/// and 7 tooltips per key (todo 2002).
#[test]
fn a_rich_text_key_redraws_no_toolbar_button() {
    block_on(async {
        let fixture = open("/perf/rich-text").await;
        let page = &fixture.page;
        step(
            page,
            &("focus", Act::Click("[contenteditable]"), "true".into()),
        )
        .await
        .unwrap();
        // The first key enables Undo, a toolbar change.
        step(page, &("key", Act::Type("1"), value_is("1")))
            .await
            .unwrap();
        settled(page, "the first key").await;
        reset(page).await;
        step(page, &("key", Act::Type("2"), value_is("12")))
            .await
            .unwrap();
        let renders = settled(page, "the second key").await;
        assert_within(
            &renders,
            &[("RichTextPage", 1), ("Flex", 1), ("RichTextEditor", 1)],
            "a key in the rich text editor",
        );
        fixture.console.assert_clean("typing").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Opening a date picker draws its picker once; focus then click both opened it, and the
/// picker redrew with the field (todo 2002).
#[test]
fn a_date_picker_opens_in_one_pass() {
    block_on(async {
        let fixture = open("/perf/date-picker").await;
        let page = &fixture.page;
        reset(page).await;
        step(
            page,
            &(
                "open",
                Act::Click("input[data-controlled]"),
                shown("[role=dialog]"),
            ),
        )
        .await
        .unwrap();
        let renders = settled(page, "opening").await;
        assert_within(
            &renders,
            &[
                ("ChronoField", 2),
                ("PortalOutlet", 2),
                ("StyleOutlet", 1),
                ("Fragment", 2),
                ("ChronoPicker", 1),
                ("Calendar", 2),
                ("Nav", 2),
                ("ActionIcon", 2),
                ("Glyph", 2),
                ("Weekdays", 1),
                ("Week", 6),
            ],
            "opening a date picker",
        );
        fixture.console.assert_clean("opening").unwrap();
        fixture.close().await.unwrap();
    });
}

/// One user action of an interaction survey.
#[derive(Clone, Copy)]
enum Act {
    Click(&'static str),
    /// The `n`th match of the selector.
    Nth(&'static str, usize),
    Hover(&'static str),
    /// Focus by script, so no click acts on the element.
    Focus(&'static str),
    Key(keyboard::Key),
    Type(&'static str),
    /// Ten moves by `(x, y)` from the selector's centre.
    Drag(&'static str, f64, f64),
}

impl Act {
    async fn run(self, page: &Page) -> anyhow::Result<()> {
        match self {
            Act::Click(selector) => pointer::click(page, selector).await,
            Act::Nth(selector, n) => {
                page.evaluate(format!(
                    "(() => {{ document.querySelectorAll('[data-e2e-nth]').forEach((e) => e.removeAttribute('data-e2e-nth')); \
                     const e = document.querySelectorAll({selector:?})[{n}]; \
                     e.setAttribute('data-e2e-nth', ''); e.scrollIntoView({{ block: 'center' }}); }})()"
                ))
                .await?;
                // At its centre, whatever covers it: a hidden native input under its label.
                let at = pointer::centre_of(page, "[data-e2e-nth]").await?;
                pointer::click_at(page, at).await
            }
            Act::Hover(selector) => pointer::hover(page, selector).await,
            Act::Focus(selector) => {
                page.evaluate(format!("document.querySelector({selector:?}).focus()"))
                    .await?;
                Ok(())
            }
            Act::Key(key) => keyboard::press(page, key).await,
            Act::Type(text) => keyboard::type_text(page, text).await,
            Act::Drag(selector, x, y) => {
                let from = pointer::centre_of(page, selector).await?;
                let to = pointer::Point {
                    x: from.x + x,
                    y: from.y + y,
                };
                pointer::drag(page, from, to, 10).await
            }
        }
    }
}

/// An action, the row it reports under, and a JS condition true once it took effect.
type Step = (&'static str, Act, String);

/// One component's interaction cycle on `/perf/<stem>` (counted) and `/timing/<stem>` (timed);
/// `setup` runs once first.
struct Case {
    component: &'static str,
    stem: &'static str,
    setup: Vec<Step>,
    steps: Vec<Step>,
}

fn shown(selector: &str) -> String {
    format!(
        "(() => {{ const e = document.querySelector({selector:?}); \
         return !!e && e.getClientRects().length > 0 && getComputedStyle(e).visibility !== 'hidden'; }})()"
    )
}

fn hidden(selector: &str) -> String {
    format!("!{}", shown(selector))
}

fn nth_is(selector: &str, n: usize, attribute: &str) -> String {
    format!(
        "(() => {{ const e = document.querySelectorAll({selector:?})[{n}]; \
         return !!e && (e.getAttribute({attribute:?}) === 'true' || e.checked === true); }})()"
    )
}

/// The focused field's text: an input's value, else a contenteditable's.
fn value_is(text: &str) -> String {
    format!("(e => (e.value ?? e.innerText).trim())(document.activeElement) === {text:?}")
}

/// Two toggles of the `n`th match of `selector`, on then off.
fn toggles(component: &'static str, stem: &'static str, selector: &'static str, n: usize) -> Case {
    Case {
        component,
        stem,
        setup: vec![],
        steps: vec![
            (
                "toggle",
                Act::Nth(selector, n),
                nth_is(selector, n, "aria-checked"),
            ),
            (
                "toggle",
                Act::Nth(selector, n),
                format!("!{}", nth_is(selector, n, "aria-checked")),
            ),
        ],
    }
}

fn open_close(component: &'static str, stem: &'static str, trigger: Act, shows: &str) -> Case {
    Case {
        component,
        stem,
        setup: vec![],
        steps: vec![
            ("open", trigger, shown(shows)),
            ("close", Act::Key(keyboard::ESCAPE), hidden(shows)),
        ],
    }
}

/// Two keys typed and deleted again, after `setup` focused the field.
fn typing(component: &'static str, stem: &'static str, setup: Vec<Step>) -> Case {
    Case {
        component,
        stem,
        setup,
        steps: vec![
            ("key", Act::Type("1"), value_is("1")),
            ("key", Act::Type("2"), value_is("12")),
            ("backspace", Act::Key(keyboard::BACKSPACE), value_is("1")),
            ("backspace", Act::Key(keyboard::BACKSPACE), value_is("")),
        ],
    }
}

const SLIDER: &str = "[role=slider]";
const PAD: &str = "[role=slider][aria-label=Saturation]";
const RADIO: &str = "input[type=radio]";
const SEGMENT: &str = "label[for*='-segment-']";
const PRESSED: &str = "document.querySelector('#press').textContent.includes('Pressed N')";
const FIRST_ITEM: &str = "document.querySelector('[role=list] li')?.textContent.trim()";
const BRANCH: &str = "[role=treeitem][data-tree-id='b0']";
const MENU_ITEM: &str = "[role=menuitem]";

/// Flipping back to the platform's own scheme drops the pin, the attribute with it.
fn scheme_is(scheme: &str) -> String {
    format!("(document.documentElement.getAttribute('data-lsx-theme') ?? 'light') === {scheme:?}")
}

fn expanded(selector: &str, open: bool) -> String {
    format!("document.querySelector({selector:?}).getAttribute('aria-expanded') === '{open}'")
}

fn focused_nth(selector: &str, n: usize) -> String {
    format!("document.activeElement === document.querySelectorAll({selector:?})[{n}]")
}

fn toasts_are(n: usize) -> String {
    format!("document.querySelectorAll('[data-notification]').length === {n}")
}

fn step_is(n: usize) -> String {
    format!("document.querySelector('#stepper-step-{n}')?.getAttribute('aria-current') === 'step'")
}

fn page_is(n: u32) -> String {
    format!("document.querySelector('#page').dataset.page === '{n}'")
}

/// A page down and back up in the focused `#scroller`.
fn paging(component: &'static str, stem: &'static str) -> Case {
    const TOP: &str = "document.getElementById('scroller').scrollTop";
    Case {
        component,
        stem,
        setup: vec![("focus", Act::Focus("#scroller"), "true".into())],
        steps: vec![
            (
                "page down",
                Act::Key(keyboard::PAGE_DOWN),
                format!("{TOP} > 0"),
            ),
            (
                "page up",
                Act::Key(keyboard::PAGE_UP),
                format!("{TOP} === 0"),
            ),
        ],
    }
}

fn cases() -> Vec<Case> {
    let focus = |selector: &'static str| vec![("focus", Act::Click(selector), "true".to_string())];
    vec![
        open_close("Select (50)", "long-select", Act::Click("[role=combobox]"), "[role=option]"),
        open_close(
            "MultiSelect (50)",
            "long-multi-select",
            Act::Click("[role=combobox]"),
            "[role=option]",
        ),
        open_close(
            "Menu",
            "menu",
            Act::Click("button[aria-haspopup]"),
            "[role=menu]",
        ),
        open_close("Popover", "popover", Act::Click("#open"), "[role=dialog]"),
        open_close("Modal", "modal", Act::Click("#open"), "[role=dialog]"),
        open_close("Drawer", "drawer", Act::Click("#open"), "[role=dialog]"),
        open_close(
            "DatePicker",
            "date-picker",
            Act::Click("input[data-controlled]"),
            "[role=dialog]",
        ),
        Case {
            component: "Tooltip",
            stem: "tooltip",
            setup: vec![],
            steps: vec![
                ("open", Act::Hover("#open"), shown("[role=tooltip]")),
                ("close", Act::Hover("#away"), hidden("[role=tooltip]")),
            ],
        },
        Case {
            component: "Calendar",
            stem: "calendar",
            setup: vec![],
            steps: vec![
                (
                    "month",
                    Act::Click("[aria-label='Next month']"),
                    "document.querySelector('[role=grid]').getAttribute('aria-label') === 'April 2026'".into(),
                ),
                (
                    "month",
                    Act::Click("[aria-label='Previous month']"),
                    "document.querySelector('[role=grid]').getAttribute('aria-label') === 'March 2026'".into(),
                ),
            ],
        },
        Case {
            component: "Tabs",
            stem: "tabs",
            setup: vec![],
            steps: vec![
                ("switch", Act::Nth("[role=tab]", 1), nth_is("[role=tab]", 1, "aria-selected")),
                ("switch", Act::Nth("[role=tab]", 0), nth_is("[role=tab]", 0, "aria-selected")),
            ],
        },
        Case {
            component: "Accordion",
            stem: "accordion",
            setup: vec![],
            steps: vec![
                (
                    "open",
                    Act::Nth("button[aria-expanded]", 0),
                    nth_is("button[aria-expanded]", 0, "aria-expanded"),
                ),
                (
                    "switch",
                    Act::Nth("button[aria-expanded]", 1),
                    nth_is("button[aria-expanded]", 1, "aria-expanded"),
                ),
                (
                    "close",
                    Act::Nth("button[aria-expanded]", 1),
                    format!("!{}", nth_is("button[aria-expanded]", 1, "aria-expanded")),
                ),
            ],
        },
        Case {
            component: "SegmentedControl",
            stem: "segmented",
            setup: vec![],
            steps: vec![
                ("switch", Act::Nth(SEGMENT, 1), nth_is(RADIO, 1, "aria-checked")),
                ("switch", Act::Nth(SEGMENT, 0), nth_is(RADIO, 0, "aria-checked")),
            ],
        },
        typing("Autocomplete (100)", "autocomplete", focus("input")),
        typing(
            "Select search (300)",
            "search-select",
            vec![(
                "open",
                Act::Click("[role=combobox]"),
                "document.activeElement.tagName === 'INPUT'".into(),
            )],
        ),
        typing("NumberField", "number-field", focus("input")),
        toggles("Form (30) Switch", "big-form", "#s3", 0),
        toggles("Form (30) Checkbox", "big-form", "#c3", 0),
        Case {
            component: "Slider",
            stem: "slider",
            setup: vec![],
            steps: vec![
                ("drag", Act::Drag(SLIDER, 100.0, 0.0), "true".into()),
                ("drag", Act::Drag(SLIDER, -100.0, 0.0), "true".into()),
            ],
        },
        Case {
            component: "ColorPicker pad",
            stem: "color-picker",
            setup: vec![],
            steps: vec![
                ("drag", Act::Drag(PAD, -40.0, 40.0), "true".into()),
                ("drag", Act::Drag(PAD, 40.0, -40.0), "true".into()),
            ],
        },
        Case {
            component: "List (40) sort",
            stem: "list",
            setup: vec![],
            steps: vec![
                ("sort", Act::Click("#sort"), format!("{FIRST_ITEM} === 'City 39'")),
                ("sort", Act::Click("#sort"), format!("{FIRST_ITEM} === 'City 0'")),
            ],
        },
        typing("List (40) filter", "list", focus("input")),
        open_close("Spotlight (60)", "spotlight", Act::Click("#open"), "[role=dialog]"),
        typing("TextField", "text-field", focus("input")),
        typing("Textarea", "textarea", focus("textarea")),
        typing("RichTextEditor", "rich-text", focus("[contenteditable]")),
        Case {
            component: "Button",
            stem: "button",
            setup: vec![],
            steps: vec![
                ("press", Act::Click("#press"), PRESSED.replace('N', "true")),
                ("press", Act::Click("#press"), PRESSED.replace('N', "false")),
            ],
        },
        toggles("Checkbox", "checkbox", "#check", 0),
        toggles("Switch", "switch", "#switch", 0),
        Case {
            component: "Theme switch",
            stem: "theme",
            setup: vec![],
            steps: vec![
                ("toggle", Act::Click("#switcher button"), scheme_is("dark")),
                ("toggle", Act::Click("#switcher button"), scheme_is("light")),
            ],
        },
        Case {
            component: "Tree (510) keys",
            stem: "tree-500",
            setup: vec![("focus", Act::Focus(BRANCH), "true".into())],
            steps: vec![
                ("expand", Act::Key(keyboard::ARROW_RIGHT), expanded(BRANCH, true)),
                ("collapse", Act::Key(keyboard::ARROW_LEFT), expanded(BRANCH, false)),
            ],
        },
        paging("List (300) keys", "list-scroll"),
        paging("DataList (300) keys", "data-list-scroll"),
        Case {
            component: "Menu (100)",
            stem: "menu-100",
            setup: vec![],
            steps: vec![
                ("open", Act::Click("button[aria-haspopup]"), shown("[role=menu]")),
                ("arrow", Act::Key(keyboard::ARROW_DOWN), focused_nth(MENU_ITEM, 1)),
                ("type-ahead", Act::Type("a"), focused_nth(MENU_ITEM, 2)),
                ("close", Act::Key(keyboard::ESCAPE), hidden("[role=menu]")),
            ],
        },
        Case {
            component: "Tabs (20) keys",
            stem: "tabs-20",
            setup: vec![("focus", Act::Focus("[role=tab]"), "true".into())],
            steps: vec![
                ("arrow", Act::Key(keyboard::ARROW_RIGHT), focused_nth("[role=tab]", 1)),
                ("arrow", Act::Key(keyboard::ARROW_LEFT), focused_nth("[role=tab]", 0)),
            ],
        },
        Case {
            component: "Toast stack (10)",
            stem: "toasts",
            setup: (1..10)
                .map(|n| ("show", Act::Click("#show"), toasts_are(n)))
                .collect(),
            steps: vec![
                ("show", Act::Click("#show"), toasts_are(10)),
                ("hide", Act::Click("#hide"), toasts_are(9)),
            ],
        },
        Case {
            component: "Stepper (9)",
            stem: "stepper",
            setup: vec![],
            steps: vec![
                ("next", Act::Click("#next"), step_is(1)),
                ("back", Act::Click("#back"), step_is(0)),
            ],
        },
        Case {
            component: "Pagination (20)",
            stem: "pagination",
            setup: vec![],
            steps: vec![
                ("page", Act::Click("[aria-label='Go to page 2']"), page_is(2)),
                ("page", Act::Click("[aria-label='Go to page 1']"), page_is(1)),
            ],
        },
        Case {
            component: "Radio",
            stem: "radio",
            setup: vec![],
            steps: vec![
                ("press", Act::Nth(RADIO, 1), nth_is(RADIO, 1, "aria-checked")),
                ("press", Act::Nth(RADIO, 0), nth_is(RADIO, 0, "aria-checked")),
            ],
        },
    ]
}

/// `PERF_CASE`, comma separated parts of component names; unset picks every case.
fn picked(only: Option<&str>, component: &str) -> bool {
    only.is_none_or(|only| only.split(',').any(|part| component.contains(part)))
}

async fn step(page: &Page, (name, act, done): &Step) -> anyhow::Result<()> {
    act.run(page).await?;
    wait::for_js_true(page, done, name).await?;
    Ok(())
}

/// The render counts of every survey step, twice through each cycle: a cost table for
/// picking what to fix, no budget. `PERF_CASE` picks cases by name;
/// `cargo run -p e2e -- perf::survey --ignored --nocapture`.
#[test]
#[ignore = "interaction survey, run on request"]
fn survey() {
    let only = std::env::var("PERF_CASE").ok();
    block_on(async {
        for case in cases() {
            if !picked(only.as_deref(), case.component) {
                continue;
            }
            let fixture = open(&format!("/perf/{}", case.stem)).await;
            let page = &fixture.page;
            let outcome: anyhow::Result<()> = async {
                for setup in &case.setup {
                    step(page, setup).await?;
                }
                for pass in 0..2 {
                    for s in &case.steps {
                        settled(page, s.0).await;
                        reset(page).await;
                        step(page, s).await?;
                        let running: String = page.evaluate(ANIMATIONS).await?.into_value()?;
                        if !running.is_empty() {
                            println!("animations | {} | {} | {running}", case.component, s.0);
                        }
                        print_renders(case.component, s.0, pass, &settled(page, s.0).await);
                    }
                }
                Ok(())
            }
            .await;
            if let Err(e) = outcome {
                println!("renders | {:<20} | FAILED: {e}", case.component);
            }
            fixture.close().await.unwrap();
        }
    });
}

/// The mount cases `/perf/mount` and `/timing/mount` offer, `PERF_CASE` applied.
async fn mount_cases(page: &Page) -> Vec<String> {
    let only = std::env::var("PERF_CASE").ok();
    let all: Vec<String> = page
        .evaluate("[...document.querySelectorAll('[id^=m-]')].map((b) => b.id.slice(2))")
        .await
        .unwrap()
        .into_value()
        .unwrap();
    all.into_iter()
        .filter(|stem| picked(only.as_deref(), stem))
        .collect()
}

/// Every element on the page, portals and style tags included.
const NODES: &str = "document.getElementsByTagName('*').length";

/// The mounted subtree's elements, its markup length and the page's style tags, so a
/// render-count fix shows it left the DOM alone.
const MOUNTED_SHAPE: &str = "(() => { const m = document.getElementById('mounted'); \
     return `${m.querySelectorAll('*').length} in #mounted, ${m.innerHTML.length} chars, \
     ${document.querySelectorAll('style').length} style tags`; })()";

/// Clicks `#m-<stem>` (or `#unmount` for `None`) and waits for `#mounted` to show it.
async fn mount(page: &Page, stem: Option<&str>) -> anyhow::Result<()> {
    let button = stem.map_or("#unmount".to_string(), |stem| format!("#m-{stem}"));
    pointer::click(page, &button).await?;
    wait::for_js_true(
        page,
        &format!(
            "document.getElementById('mounted').dataset.case === {:?}",
            stem.unwrap_or("")
        ),
        "the mount",
    )
    .await
}

/// The mounts todo 2031 fixed: a `Box` scope per CodeBlock row part (1504 for 500 lines) and
/// a second render of every closed `Collapse` (100 for an Accordion of 50).
#[test]
fn mounting_stays_in_budget() {
    block_on(async {
        let fixture = open("/perf/mount").await;
        let page = &fixture.page;
        let page_scopes = [("MountPage", 1), ("Flex", 1), ("StyleOutlet", 2)];
        for (stem, budget) in [
            (
                "code-block-500",
                // The second CodeBlock pass is the highlight landing.
                &[
                    ("MountCodeBlock", 1),
                    ("CodeBlock", 2),
                    ("Box", 4),
                    ("Copy", 1),
                    ("ActionIcon", 1),
                    ("Glyph", 1),
                    ("VisuallyHidden", 1),
                ][..],
            ),
            (
                "accordion-50",
                &[
                    ("MountAccordion", 1),
                    ("Accordion", 1),
                    ("AccordionSection", 50),
                    ("Collapse", 50),
                ][..],
            ),
            (
                // Closed: no option rows built, and the mount effect's highlight redraws nothing.
                "combobox-300",
                &[
                    ("MountCombobox", 1),
                    ("Combobox", 1),
                    ("ComboboxCore", 1),
                    ("Button", 1),
                ][..],
            ),
            (
                // A fixed `today` skips the clock write: one Calendar pass.
                "calendar",
                &[
                    ("MountCalendar", 1),
                    ("ChronoPicker", 1),
                    ("Calendar", 1),
                    ("Weekdays", 1),
                    ("Week", 6),
                    ("Nav", 2),
                    ("ActionIcon", 2),
                    ("Glyph", 2),
                ][..],
            ),
        ] {
            reset(page).await;
            mount(page, Some(stem)).await.unwrap();
            let renders = settled(page, stem).await;
            let all: Vec<_> = page_scopes.iter().chain(budget).copied().collect();
            assert_within(&renders, &all, stem);
            mount(page, None).await.unwrap();
            settled(page, "unmount").await;
        }
        fixture.console.assert_clean("mounting").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Scope renders and elements added per mount, first and second time (todo 2031):
/// `cargo run -p e2e -- perf::mount_survey --ignored --nocapture`, `PERF_CASE` picks.
#[test]
#[ignore = "mount survey, run on request"]
fn mount_survey() {
    block_on(async {
        let fixture = open("/perf/mount").await;
        let page = &fixture.page;
        for stem in mount_cases(page).await {
            for pass in 0..2 {
                mount(page, None).await.unwrap();
                settled(page, "unmount").await;
                let before: u32 = page.evaluate(NODES).await.unwrap().into_value().unwrap();
                reset(page).await;
                mount(page, Some(&stem)).await.unwrap();
                let renders = settled(page, &stem).await;
                let after: u32 = page.evaluate(NODES).await.unwrap().into_value().unwrap();
                let shape: String = page
                    .evaluate(MOUNTED_SHAPE)
                    .await
                    .unwrap()
                    .into_value()
                    .unwrap();
                println!(
                    "nodes   | {stem:<20} | pass {pass} | {} | {shape}",
                    after - before
                );
                print_renders(&stem, "mount", pass, &renders);
            }
        }
        fixture.console.assert_clean("mounting").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The transitions and animations running right after a step, counted by property and target.
const ANIMATIONS: &str = "(() => { const seen = {}; for (const a of document.getAnimations()) { \
     const t = a.effect && a.effect.target; \
     const what = (a.transitionProperty || a.animationName) + ' on ' + (t ? t.tagName.toLowerCase() + (t.dataset.slot ? '[' + t.dataset.slot + ']' : '') + (t.getAttribute('role') ? '(' + t.getAttribute('role') + ')' : '') : '?') + ' ' + Math.round(a.effect.getComputedTiming().duration) + 'ms'; \
     seen[what] = (seen[what] || 0) + 1; } \
     return Object.entries(seen).map(([k, v]) => v + 'x ' + k).join('; '); })()";

/// One survey line: the total, the scope count and the eight busiest scopes.
fn print_renders(component: &str, step: &str, pass: usize, renders: &Renders) {
    let total: u32 = renders.values().sum();
    let mut top: Vec<_> = renders.iter().collect();
    top.sort_by(|a, b| b.1.cmp(a.1));
    let top: Vec<String> = top
        .iter()
        .take(8)
        .map(|(name, n)| {
            let name = name.split('<').next().unwrap_or(name);
            format!("{} {n}", name.rsplit("::").next().unwrap_or(name))
        })
        .collect();
    println!(
        "renders | {component:<20} | {step:<9} | pass {pass} | total {total:>4} | scopes {:>3} | {}",
        renders.len(),
        top.join(", ")
    );
}

/// Main-thread time per interaction in a real browser, each repeated: one report row per
/// interaction, its median task time held to a budget of about 5x the release medians of
/// 2026-10-05, so only a real regression trips it. Opt-in, a shared CPU makes milliseconds
/// noise in the gate; run it on the release build:
/// `E2E_RELEASE=1 cargo run -p e2e -- perf::timing:: --ignored --nocapture`. The helpers
/// serve `table_perf` too.
pub(crate) mod timing {
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    use anyhow::{Result, bail};
    use chromiumoxide::Page;
    use chromiumoxide::cdp::browser_protocol::input::{
        DispatchMouseEventParams, DispatchMouseEventType,
    };
    use chromiumoxide::cdp::browser_protocol::performance::{EnableParams, GetMetricsParams};
    use e2e::browser::block_on;
    use e2e::passes::keyboard::{self, BACKSPACE, ESCAPE};
    use e2e::passes::pointer::{self, Point};
    use e2e::{Fixture, Viewport, frames, wait};
    use serde::{Deserialize, Serialize};

    /// Unmeasured passes first: the first run of a path pays for its wasm and style warm-up.
    pub(crate) const WARM_UP: usize = 2;

    pub(crate) const RECORDER: &str = r#"(() => {
        const t = { frames: [], loaf: [], events: [], last: null };
        const tick = (now) => {
            if (t.last !== null) t.frames.push(now - t.last);
            t.last = now;
            requestAnimationFrame(tick);
        };
        requestAnimationFrame(tick);
        try {
            new PerformanceObserver((l) => t.loaf.push(...l.getEntries().map((e) => e.duration)))
                .observe({ type: 'long-animation-frame' });
        } catch (_) {}
        try {
            new PerformanceObserver((l) => t.events.push(...l.getEntries().map((e) => e.duration)))
                .observe({ type: 'event', durationThreshold: 16 });
        } catch (_) {}
        t.reset = () => { t.frames = []; t.loaf = []; t.events = []; };
        t.read = () => JSON.stringify({
            frame: Math.max(0, ...t.frames),
            loaf: t.loaf.length,
            event: Math.max(0, ...t.events),
        });
        window.__timing = t;
    })()"#;

    /// One interaction, milliseconds.
    #[derive(Debug, Clone, Copy, Default, Serialize)]
    struct Rep {
        /// Every main-thread task: script, style, layout, paint.
        task: f64,
        script: f64,
        /// Style recalculation and layout.
        layout: f64,
        /// The longest main-thread (rAF) frame while it ran: over 16.7 skipped one. Not
        /// what showed: a scroll that commits DOM under a scroll timeline skips a main
        /// frame while the compositor still draws every vsync (todo 1987).
        frame: f64,
        /// Long animation frames (50 ms and over).
        long: usize,
        /// The slowest event's input-to-paint time; 0 under the API's 16 ms floor.
        event: f64,
    }

    #[derive(Deserialize)]
    struct Seen {
        frame: f64,
        loaf: usize,
        event: f64,
    }

    /// `name -> seconds` for the counters a rep reads.
    async fn metrics(page: &Page) -> Result<BTreeMap<String, f64>> {
        let reply = page.execute(GetMetricsParams::default()).await?;
        Ok(reply
            .result
            .metrics
            .iter()
            .map(|metric| (metric.name.clone(), metric.value))
            .collect())
    }

    fn delta_ms(after: &BTreeMap<String, f64>, before: &BTreeMap<String, f64>, key: &str) -> f64 {
        (after.get(key).copied().unwrap_or(0.0) - before.get(key).copied().unwrap_or(0.0)) * 1e3
    }

    /// Until 60 ms pass with under 2 ms of main-thread work: effects, placement passes and
    /// exit animations belong to the interaction that started them.
    pub(crate) async fn quiet(page: &Page) -> Result<()> {
        let started = Instant::now();
        let mut last = metrics(page).await?;
        loop {
            tokio::time::sleep(Duration::from_millis(60)).await;
            let now = metrics(page).await?;
            if delta_ms(&now, &last, "TaskDuration") < 2.0 {
                return Ok(());
            }
            // A 5000-row Table mount works for seconds.
            if started.elapsed() > Duration::from_secs(20) {
                bail!("the page never went quiet");
            }
            last = now;
        }
    }

    pub(crate) struct Rows(BTreeMap<&'static str, Vec<Rep>>);

    /// Runs `act` `WARM_UP + reps` times; each run names the row it belongs to.
    pub(crate) async fn measure(
        page: &Page,
        reps: usize,
        mut act: impl AsyncFnMut(usize) -> Result<&'static str>,
    ) -> Result<Rows> {
        let mut rows = Rows(BTreeMap::new());
        for i in 0..WARM_UP + reps {
            quiet(page).await?;
            page.evaluate("window.__timing.reset()").await?;
            let before = metrics(page).await?;
            let row = act(i).await?;
            quiet(page).await?;
            let after = metrics(page).await?;
            let seen: Seen = serde_json::from_str(
                &page
                    .evaluate("window.__timing.read()")
                    .await?
                    .into_value::<String>()?,
            )?;
            if i < WARM_UP {
                continue;
            }
            rows.0.entry(row).or_default().push(Rep {
                task: delta_ms(&after, &before, "TaskDuration"),
                script: delta_ms(&after, &before, "ScriptDuration"),
                layout: delta_ms(&after, &before, "LayoutDuration")
                    + delta_ms(&after, &before, "RecalcStyleDuration"),
                frame: seen.frame,
                long: seen.loaf,
                event: seen.event,
            });
        }
        Ok(rows)
    }

    #[derive(Debug, Clone, Copy, Serialize)]
    struct Summary {
        reps: usize,
        median: Rep,
        worst: Rep,
    }

    fn median(mut values: Vec<f64>) -> f64 {
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    }

    fn summarise(reps: &[Rep]) -> Summary {
        let pick = |f: fn(&Rep) -> f64| reps.iter().map(f).collect::<Vec<_>>();
        let worst = |f: fn(&Rep) -> f64| pick(f).into_iter().fold(0.0, f64::max);
        Summary {
            reps: reps.len(),
            median: Rep {
                task: median(pick(|r| r.task)),
                script: median(pick(|r| r.script)),
                layout: median(pick(|r| r.layout)),
                frame: median(pick(|r| r.frame)),
                long: reps.iter().map(|r| r.long).sum(),
                event: median(pick(|r| r.event)),
            },
            worst: Rep {
                task: worst(|r| r.task),
                script: worst(|r| r.script),
                layout: worst(|r| r.layout),
                frame: worst(|r| r.frame),
                long: reps.iter().map(|r| r.long).max().unwrap_or(0),
                event: worst(|r| r.event),
            },
        }
    }

    /// Prints the rows, merges them into `interaction-time.json` in the target dir, then
    /// holds each row's median task time to its budget.
    pub(crate) fn report(component: &str, rows: &Rows, budgets: &[(&str, f64)]) {
        let mut all: BTreeMap<String, Summary> = std::fs::read_to_string(json_path())
            .ok()
            .and_then(|text| {
                serde_json::from_str::<BTreeMap<String, serde_json::Value>>(&text).ok()
            })
            .map(|old| {
                old.into_iter()
                    .filter_map(|(k, v)| Some((k, summary_from(&v)?)))
                    .collect()
            })
            .unwrap_or_default();
        let mut over = Vec::new();
        for (row, reps) in &rows.0 {
            let name = format!("{component} {row}");
            let s = summarise(reps);
            println!(
                "timing | {name:<28} | n {:>2} | task {:>6.1} / {:>6.1} | script {:>6.1} / {:>6.1} | \
                 layout {:>5.1} / {:>5.1} | frame {:>5.1} / {:>5.1} | long {:>2} | event {:>5.1} / {:>5.1}",
                s.reps,
                s.median.task,
                s.worst.task,
                s.median.script,
                s.worst.script,
                s.median.layout,
                s.worst.layout,
                s.median.frame,
                s.worst.frame,
                s.median.long,
                s.median.event,
                s.worst.event,
            );
            let budget = budgets
                .iter()
                .find(|(listed, _)| listed == row)
                .map(|(_, budget)| *budget)
                .unwrap_or_else(|| panic!("{name}: no budget"));
            if s.median.task > budget {
                over.push(format!(
                    "{name}: median task {:.1} ms (budget {budget} ms)",
                    s.median.task
                ));
            }
            all.insert(name, s);
        }
        let _ = std::fs::write(json_path(), serde_json::to_string_pretty(&all).unwrap());
        assert!(over.is_empty(), "over budget: {over:?}");
    }

    fn summary_from(value: &serde_json::Value) -> Option<Summary> {
        let rep = |v: &serde_json::Value| -> Option<Rep> {
            Some(Rep {
                task: v["task"].as_f64()?,
                script: v["script"].as_f64()?,
                layout: v["layout"].as_f64()?,
                frame: v["frame"].as_f64()?,
                long: v["long"].as_u64()? as usize,
                event: v["event"].as_f64()?,
            })
        };
        Some(Summary {
            reps: value["reps"].as_u64()? as usize,
            median: rep(&value["median"])?,
            worst: rep(&value["worst"])?,
        })
    }

    fn json_path() -> PathBuf {
        let exe = std::env::current_exe().unwrap();
        // <target>/<profile>/deps/<binary>
        exe.ancestors()
            .nth(3)
            .unwrap()
            .join("interaction-time.json")
    }

    /// The page in front, the recorder and the CDP counters on; `run` gets the page.
    pub(crate) async fn timed(route: &str, run: impl AsyncFnOnce(&Page) -> Result<()>) {
        let fixture = Fixture::open(route, Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        let front = frames::bring_to_front(page).await.unwrap();
        let outcome = async {
            page.execute(EnableParams::default()).await?;
            page.evaluate(RECORDER).await?;
            run(page).await
        }
        .await;
        front.release().await.unwrap();
        outcome.unwrap();
        fixture.console.assert_clean(route).unwrap();
        fixture.close().await.unwrap();
    }

    pub(crate) async fn js<T: serde::de::DeserializeOwned>(
        page: &Page,
        expression: &str,
    ) -> Result<T> {
        Ok(page.evaluate(expression).await?.into_value()?)
    }

    /// The first element under `#pane` that scrolls.
    pub(crate) const SCROLLER: &str = "[...document.querySelectorAll('#pane *')].find((el) => el.scrollHeight > el.clientHeight + 1)";

    /// One wheel notch of 120 px per rep, half the reps down, then back up.
    pub(crate) async fn wheel_reps(page: &Page, reps: usize) -> Result<Rows> {
        let at: Point = js(
            page,
            &format!("(() => {{ const r = {SCROLLER}.getBoundingClientRect(); return {{ x: r.x + r.width / 2, y: r.y + r.height / 2 }}; }})()"),
        )
        .await?;
        let total = WARM_UP + reps;
        measure(page, reps, async |i| {
            let down = i < total / 2;
            let before: f64 = js(page, &format!("{SCROLLER}.scrollTop")).await?;
            page.execute(
                DispatchMouseEventParams::builder()
                    .r#type(DispatchMouseEventType::MouseWheel)
                    .x(at.x)
                    .y(at.y)
                    .delta_x(0.0)
                    .delta_y(if down { 120.0 } else { -120.0 })
                    .build()
                    .map_err(anyhow::Error::msg)?,
            )
            .await?;
            wait::for_js_true(
                page,
                &format!("{SCROLLER}.scrollTop !== {before}"),
                "the wheel to scroll",
            )
            .await?;
            Ok("wheel")
        })
        .await
    }

    /// A press, `moves` pointer moves `by` along x (or y), a release, on `selector`'s
    /// centre; every other rep goes back.
    pub(crate) async fn drag_reps(
        page: &Page,
        selector: &str,
        by: (f64, f64),
        moves: usize,
        changed: &str,
        reps: usize,
    ) -> Result<Rows> {
        measure(page, reps, async |i| {
            let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
            let from = pointer::centre_of(page, selector).await?;
            let to = Point {
                x: from.x + by.0 * sign,
                y: from.y + by.1 * sign,
            };
            let before: String = js(page, changed).await?;
            pointer::drag(page, from, to, moves).await?;
            wait::for_js_true(
                page,
                &format!("{changed} !== {}", serde_json::to_string(&before)?),
                "the drag to move it",
            )
            .await?;
            Ok("drag")
        })
        .await
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn scrolling() {
        block_on(async {
            // The Table's wheel rows are table_perf's.
            for (route, component, budget) in [
                ("/timing/virtualize", "Virtualize", 40.0),
                ("/timing/scroll-area", "ScrollArea", 40.0),
                ("/timing/native-scroll", "plain div (control)", 40.0),
            ] {
                timed(route, async |page| {
                    let rows = wheel_reps(page, 12).await?;
                    report(component, &rows, &[("wheel", budget)]);
                    Ok(())
                })
                .await;
            }
        });
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn carousel_controls() {
        const INDEX: &str = "[...document.querySelectorAll('[aria-roledescription=slide]')]\
                             .findIndex(el => el.hasAttribute('data-current'))";
        block_on(timed("/timing/carousel", async |page| {
            let rows = measure(page, 12, async |i| {
                let forward = (i / 4) % 2 == 0;
                let now: i64 = js(page, INDEX).await?;
                let (label, next) = match forward {
                    true => ("Next slide", now + 1),
                    false => ("Previous slide", now - 1),
                };
                pointer::click(
                    page,
                    &format!("[aria-roledescription=carousel] button[aria-label='{label}']"),
                )
                .await?;
                wait::for_js_true(page, &format!("{INDEX} === {next}"), "the slide to move")
                    .await?;
                Ok("control click")
            })
            .await?;
            report("Carousel", &rows, &[("control click", 60.0)]);
            Ok(())
        }));
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn tree_branch() {
        const BRANCH: &str = "[role=treeitem][data-tree-id='b0']";
        block_on(timed("/timing/tree", async |page| {
            let rows = measure(page, 24, async |i| {
                let expand = i % 2 == 0;
                pointer::click(page, &format!("{BRANCH} [data-tree-chevron]")).await?;
                wait::for_js_true(
                    page,
                    &format!(
                        "document.querySelector(\"{BRANCH}\").getAttribute('aria-expanded') === '{expand}'"
                    ),
                    "the branch to toggle",
                )
                .await?;
                Ok(if expand { "expand" } else { "collapse" })
            })
            .await?;
            report("Tree", &rows, &[("expand", 80.0), ("collapse", 80.0)]);
            Ok(())
        }));
    }

    /// Open on a click on `trigger`, close on Escape; `shown` is the overlay's selector.
    async fn open_close(page: &Page, trigger: &str, shown: &str) -> Result<Rows> {
        measure(page, 24, async |i| {
            if i % 2 == 0 {
                pointer::click(page, trigger).await?;
                wait::for_visible(page, shown).await?;
                Ok("open")
            } else {
                keyboard::press(page, ESCAPE).await?;
                wait::for_hidden(page, shown).await?;
                Ok("close")
            }
        })
        .await
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn overlays() {
        block_on(async {
            for (route, component, trigger, shown, budget) in [
                ("/timing/modal", "Modal", "#open", "[role=dialog]", 80.0),
                (
                    "/timing/menu",
                    "Menu",
                    "button[aria-haspopup]",
                    "[role=menu]",
                    80.0,
                ),
                (
                    "/timing/spotlight",
                    "Spotlight",
                    "#open",
                    "[role=dialog]",
                    120.0,
                ),
            ] {
                timed(route, async |page| {
                    let rows = open_close(page, trigger, shown).await?;
                    report(component, &rows, &[("open", budget), ("close", budget)]);
                    Ok(())
                })
                .await;
            }
        });
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn day_picker() {
        const GRID: &str = "document.querySelector('[role=grid]').getAttribute('aria-label')";
        block_on(timed("/timing/calendar", async |page| {
            let rows = measure(page, 24, async |i| match i % 4 {
                0 | 1 => {
                    let (button, month) = match i % 4 {
                        0 => ("Next month", "April 2026"),
                        _ => ("Previous month", "March 2026"),
                    };
                    pointer::click(page, &format!("[aria-label='{button}']")).await?;
                    wait::for_js_true(page, &format!("{GRID} === '{month}'"), "the page").await?;
                    Ok("page")
                }
                _ => {
                    let day = if i % 4 == 2 {
                        "2026-03-10"
                    } else {
                        "2026-03-12"
                    };
                    let cell = format!("[role=grid] [data-date='{day}']");
                    pointer::click(page, &cell).await?;
                    wait::for_js_true(
                        page,
                        &format!(
                            "document.querySelector(\"{cell}\").hasAttribute('data-selected')"
                        ),
                        "the pick",
                    )
                    .await?;
                    Ok("pick")
                }
            })
            .await?;
            report("DayPicker", &rows, &[("page", 80.0), ("pick", 80.0)]);
            Ok(())
        }));
    }

    /// One key per rep: `text` typed, then deleted again, `cycles` times. `reopen` closes the
    /// list with an Escape after each cycle, so every first key opens it.
    async fn typing(page: &Page, text: &str, cycles: usize, reopen: bool) -> Result<Rows> {
        pointer::click(page, "input").await?;
        let chars: Vec<char> = text.chars().collect();
        let cycle = 2 * chars.len() + usize::from(reopen);
        measure(page, cycle * cycles - WARM_UP, async |i| {
            let step = i % cycle;
            if step == 2 * chars.len() {
                keyboard::press(page, ESCAPE).await?;
                wait::for_hidden(page, "[role=listbox]").await?;
                return Ok("escape");
            }
            let expected: String = match step < chars.len() {
                true => {
                    keyboard::type_text(page, &chars[step].to_string()).await?;
                    chars[..=step].iter().collect()
                }
                false => {
                    keyboard::press(page, BACKSPACE).await?;
                    chars[..2 * chars.len() - step - 1].iter().collect()
                }
            };
            wait::for_js_true(
                page,
                &format!(
                    "document.querySelector('input').value === {}",
                    serde_json::to_string(&expected)?
                ),
                "the keystroke",
            )
            .await?;
            Ok(if step == 0 { "first key" } else { "keystroke" })
        })
        .await
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn typing_in_fields() {
        block_on(async {
            for (route, component, text, cycles, reopen, budget) in [
                (
                    "/timing/text-field",
                    "TextField",
                    "hello world",
                    2,
                    false,
                    40.0,
                ),
                (
                    "/timing/autocomplete",
                    "Autocomplete",
                    "city 12",
                    12,
                    true,
                    80.0,
                ),
            ] {
                timed(route, async |page| {
                    let rows = typing(page, text, cycles, reopen).await?;
                    let budgets = [
                        ("first key", budget),
                        ("keystroke", budget),
                        ("escape", budget),
                    ];
                    report(component, &rows, &budgets);
                    Ok(())
                })
                .await;
            }
        });
    }

    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn drags() {
        block_on(async {
            timed("/timing/slider", async |page| {
                let value = "document.querySelector('[role=slider]').getAttribute('aria-valuenow')";
                let rows = drag_reps(page, "[role=slider]", (100.0, 0.0), 10, value, 12).await?;
                report("Slider", &rows, &[("drag", 120.0)]);
                Ok(())
            })
            .await;
            timed("/timing/splitter", async |page| {
                let value =
                    "document.querySelector('[role=separator]').getAttribute('aria-valuenow')";
                let rows = drag_reps(page, "[role=separator]", (80.0, 0.0), 10, value, 12).await?;
                report("Splitter", &rows, &[("drag", 120.0)]);
                Ok(())
            })
            .await;
            timed("/timing/floating-window", async |page| {
                pointer::click(page, "#open").await?;
                wait::for_visible(page, "[role=dialog]").await?;
                let x = "String(Math.round(document.querySelector('[role=dialog]').getBoundingClientRect().x))";
                let rows =
                    drag_reps(page, "[role=dialog] [data-slot=handle]", (80.0, 40.0), 10, x, 12)
                        .await?;
                report("FloatingWindow", &rows, &[("drag", 120.0)]);
                Ok(())
            })
            .await;
        });
    }

    /// Every survey case, each step about eight times; `PERF_CASE` picks cases by name.
    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn survey() {
        let only = std::env::var("PERF_CASE").ok();
        block_on(async {
            for case in super::cases() {
                if !super::picked(only.as_deref(), case.component) {
                    continue;
                }
                timed(&format!("/timing/{}", case.stem), async |page| {
                    // Extra CSS for an ablation, e.g. `*, ::after { animation: none !important }`.
                    if let Ok(css) = std::env::var("PERF_CSS") {
                        page.evaluate(format!(
                            "document.head.insertAdjacentHTML('beforeend', {:?})",
                            format!("<style>{css}</style>")
                        ))
                        .await?;
                    }
                    for setup in &case.setup {
                        super::step(page, setup).await?;
                    }
                    let n = case.steps.len();
                    let rows = measure(page, 8 * n, async |i| {
                        let s = &case.steps[i % n];
                        super::step(page, s).await?;
                        Ok(s.0)
                    })
                    .await?;
                    let budgets: Vec<_> = case.steps.iter().map(|s| (s.0, 500.0)).collect();
                    report(case.component, &rows, &budgets);
                    Ok(())
                })
                .await;
            }
        });
    }

    /// Main-thread time of each mount case, mounted and dropped again eight times; the
    /// `empty` row is the click's own floor. `E2E_RELEASE=1`, `PERF_CASE` picks.
    #[test]
    #[ignore = "mount timing report, run on request"]
    fn mount() {
        block_on(timed("/timing/mount", async |page| {
            for stem in super::mount_cases(page).await {
                let rows = measure(page, 16, async |i| {
                    let mounting = i % 2 == 0;
                    super::mount(page, mounting.then_some(stem.as_str())).await?;
                    Ok(if mounting { "mount" } else { "unmount" })
                })
                .await?;
                report(&stem, &rows, &[("mount", 500.0), ("unmount", 500.0)]);
            }
            Ok(())
        }));
    }

    /// Nothing done: what a rep costs the page at rest, the floor under every row.
    #[test]
    #[ignore = "interaction timing report, run on request"]
    fn control() {
        block_on(timed("/timing/text-field", async |page| {
            let rows = measure(page, 12, async |_| Ok("idle")).await?;
            report("Control", &rows, &[("idle", 10.0)]);
            Ok(())
        }));
    }
}
