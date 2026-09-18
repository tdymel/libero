//! Render counts per interaction in a real browser (todo 821): each row drives
//! one interaction, waits until nothing renders any more, and holds the count
//! per scope to a budget. The probe proves the counter first (todo 463).

use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Viewport, wait};

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

/// The counts once nothing has rendered for 300 ms; a count still growing
/// after 3 s is a runaway.
async fn settled(page: &Page, what: &str) -> Renders {
    let started = Instant::now();
    let mut last = read(page).await;
    loop {
        tokio::time::sleep(Duration::from_millis(300)).await;
        let now = read(page).await;
        if now == last {
            return now;
        }
        assert!(
            started.elapsed() < Duration::from_secs(3),
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

/// Every scope that rendered is in `budget` and stayed within its count. The
/// budgets are the counts measured on 2026-09-27: they are deterministic, so
/// any growth is a change to look at, and a fix that lowers one passes.
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

/// A closure handler prop re-renders its child on every parent render; a
/// `use_callback` one skips. If the counter could not see that, no budget
/// below would mean anything.
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
        // The popup's second pass is the popover placing the measured box.
        assert_within(
            &opened,
            &[
                ("Select", 1),
                ("SelectCore", 1),
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
                ("CloseIcon", 1),
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
        // padding box around it, never the area.
        assert_within(
            &renders,
            &[("ScrollAreaContent", 10), ("Virtualize", 10)],
            "ten scroll steps",
        );
        fixture.console.assert_clean("scrolling").unwrap();
        fixture.close().await.unwrap();
    });
}
