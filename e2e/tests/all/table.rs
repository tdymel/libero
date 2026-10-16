//! `Table` core: header sort, row selection, pages, the column menu and the baseline.
//! Helpers the sibling `table_*` files use are exported from here.

use anyhow::{Result, bail};
use chromiumoxide::Page;
use e2e::browser::block_on;
use e2e::driver::{Driver, Host, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::{keyboard, pointer};
use e2e::{Fixture, Suite, Viewport, ax, wait};

const SORT: &str = "th[data-sortable] button";

async fn sorted<D: Driver>(d: &mut D, expected: &str) -> Result<()> {
    eventually(d, &format!("aria-sort {expected}"), async |d| {
        Ok(d.attr("th[data-sortable]", "aria-sort").await?.as_deref() == Some(expected))
    })
    .await
}

async fn the_arrow_turns<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const ARROW: &str = "th[data-sortable] svg";
    d.click(SORT).await?;
    sorted(d, "ascending").await?;
    // Past the turn's transition: the same angle across a drawn frame, not a fixed wait (1763).
    let mut ascending = d.style(ARROW, "transform").await?;
    eventually(d, "the arrow's turn to end", async |d| {
        d.frame().await?;
        let now = d.style(ARROW, "transform").await?;
        let still = now == ascending;
        ascending = now;
        Ok(still)
    })
    .await?;
    d.click(SORT).await?;
    sorted(d, "descending").await?;
    eventually(
        d,
        &format!("the arrow to turn from {ascending}"),
        async |d| Ok(d.style(ARROW, "transform").await? != ascending),
    )
    .await
}

e2e::scenario!(
    the_arrow_turns_when_the_sort_flips,
    "/table",
    the_arrow_turns
);

pub(super) const AREA: &str = "[data-table-scroll]";

/// A `NativeSelect`: natively a listbox combobox, no `<select>`.
pub(crate) const PICKER: &str = ":is(select, [role=combobox])";

/// Steps the focused `NativeSelect` `steps` options on. Natively the first
/// arrow opens its listbox on the current option and Enter picks.
pub(crate) async fn pick_later<D: Driver>(d: &mut D, steps: usize) -> Result<()> {
    let macos_webview = d.platform() == Platform::Desktop && Host::current() == Host::Macos;
    if d.platform() == Platform::Android || macos_webview {
        // Android's WebView sometimes moves focus on ArrowDown instead of stepping the select
        // (3257); WKWebView opens the system menu (3432).
        d.evaluate(&format!(
            "(() => {{ const s = document.activeElement; \
             s.selectedIndex = Math.min(s.selectedIndex + {steps}, s.options.length - 1); \
             for (const t of ['input', 'change']) s.dispatchEvent(new Event(t, {{ bubbles: true }})); \
             return 1; }})()"
        ))
        .await?;
        return Ok(());
    }
    let native = d.platform() == Platform::Native;
    if native {
        d.press(keyboard::ARROW_DOWN).await?;
    }
    let waits = matches!(d.platform(), Platform::Desktop | Platform::Web);
    for step in 0..steps {
        let before = match waits {
            true => d.value(":focus").await?,
            false => String::new(),
        };
        d.press(keyboard::ARROW_DOWN).await?;
        // A press before the last change's render is rewound: WebKitGTK, a loaded Chromium (3305).
        if waits {
            eventually(d, &format!("ArrowDown {}", step + 1), async |d| {
                Ok(d.value(":focus").await? != before)
            })
            .await?;
        }
    }
    if native {
        d.press(keyboard::ENTER).await?;
    }
    Ok(())
}

/// 1156-0b/0c: a row click reports its row; states, stripes, `sm` padding and
/// a formatted cell all show.
async fn the_rows_report_and_show_their_look<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const APPLE: &str = "tr[data-name=\"Apple\"]";
    // Before any click, so no row is hovered: the hover wins over a stripe.
    let clear = d.style("tbody tr:nth-child(1)", "background-color").await?;
    let stripe = d.style("tbody tr:nth-child(2)", "background-color").await?;
    if clear == stripe {
        bail!("the second row is not striped: {stripe}");
    }
    // 1156 follow-up: a hovered stripe takes its own shade.
    d.hover("tbody tr:nth-child(2) td").await?;
    eventually(d, "a hovered striped row to change", async |d| {
        let hovered = d.style("tbody tr:nth-child(2)", "background-color").await?;
        Ok(hovered != stripe && hovered != clear)
    })
    .await?;
    let padding = d.style("tbody td", "padding-top").await?;
    if padding != "6px" {
        bail!("an sm cell's padding is {padding}");
    }
    let color = d.style("tr[data-state~=\"sold-out\"] td", "color").await?;
    if color != "rgb(255, 0, 0)" {
        bail!("the sold-out row's state did not style it: {color}");
    }
    eventually_text(d, &format!("{APPLE} td"), "12 kg", "formatting").await?;
    d.click(&format!("{APPLE} td")).await?;
    eventually_text(d, "#clicked", "Apple", "a row click").await
}

e2e::scenario!(
    a_row_click_reports_its_row,
    "/table/rows",
    the_rows_report_and_show_their_look,
    ios: skip("a touch screen has no hover")
);

/// The checkbox box of body row `n` (1-based), or of the header with 0.
fn select_box(n: usize) -> String {
    match n {
        0 => "thead th[data-select] span[aria-hidden]".to_string(),
        n => format!("tbody tr:nth-child({n}) td[data-select] span[aria-hidden]"),
    }
}

const STOCK: &str = "th[data-sortable]:nth-child(3) button";
const NAME: &str = "th[data-sortable]:nth-child(2) button";

/// Select-all is mixed: a browser (web, WebView) reads the `indeterminate` property, since an
/// `aria-checked` on a native checkbox is forbidden there (2952); Blitz the attribute.
async fn select_all_mixed<D: Driver>(d: &mut D) -> Result<bool> {
    match d.platform() {
        Platform::Native => {
            Ok(d.attr("thead input", "aria-checked").await?.as_deref() == Some("mixed"))
        }
        _ => Ok(d
            .evaluate("document.querySelector('thead input').indeterminate")
            .await?
            == true),
    }
}

/// 1156-1a/1c: a row's box selects it without a row click, select-all mixes and
/// fills, the selection follows its rows through a sort; a tap on touch adds a
/// sorted column, a click on a pointer replaces it.
async fn rows_select_and_sort<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually_text(d, "#selection", "Apple", "the seeded selection").await?;
    eventually(d, "select-all mixed with one row selected", async |d| {
        select_all_mixed(d).await
    })
    .await?;
    // Source order: Cherry, Apple, Banana, Date, Elder.
    d.click(&select_box(3)).await?;
    eventually_text(d, "#selection", "Apple,Banana", "a row's box").await?;
    // Found by selector, not a mounted handle (todo 3252).
    eventually_focused(
        d,
        "tbody tr:nth-child(3) td[data-select] input",
        "a row's box",
    )
    .await?;
    if !d.text("#clicked").await?.is_empty() {
        bail!("a row's box also clicked its row");
    }

    d.click(STOCK).await?;
    eventually_text(d, "#sort", "Stock ascending", "a Stock click").await?;
    // Banana (0) first now, still selected; Cherry (3) next, not.
    eventually(d, "the selection to follow its rows", async |d| {
        Ok(d.attr("tbody tr:nth-child(1)", "aria-selected")
            .await?
            .as_deref()
            == Some("true")
            && d.attr("tbody tr:nth-child(2)", "aria-selected")
                .await?
                .as_deref()
                == Some("false"))
    })
    .await?;
    let selected = d.style("tbody tr:nth-child(1)", "background-color").await?;
    let plain = d.style("tbody tr:nth-child(2)", "background-color").await?;
    if selected == plain {
        bail!("a selected row looks like the others: {selected}");
    }

    d.click(NAME).await?;
    let expected = match d.platform().is_touch() {
        true => "Stock ascending,Name ascending",
        false => "Name ascending",
    };
    eventually_text(d, "#sort", expected, "a Name click").await?;

    d.click(&select_box(0)).await?;
    eventually_text(
        d,
        "#selection",
        "Apple,Banana,Cherry,Date,Elder",
        "select-all",
    )
    .await?;
    eventually(d, "select-all no longer mixed", async |d| {
        Ok(!select_all_mixed(d).await?)
    })
    .await?;
    Ok(())
}

e2e::scenario!(
    rows_select_by_box_and_keep_it_through_a_sort,
    "/table/select",
    rows_select_and_sort
);

/// 1982: the light boxes keep select-all's mixed property, and under forced colours
/// every outline shows and only a checked or mixed box shows its mark.
#[test]
fn the_boxes_show_their_state_in_forced_colours() {
    block_on(async {
        let fixture = Fixture::open("/table/select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        wait::for_js_true(
            page,
            "document.querySelector('thead input').indeterminate === true",
            "select-all's indeterminate property",
        )
        .await
        .unwrap();
        e2e::browser::force_colours(page).await.unwrap();
        let wrong: Vec<String> = page
            .evaluate(
                "(() => { const probe = document.createElement('div'); \
                 probe.style.background = 'Canvas'; document.body.append(probe); \
                 const canvas = getComputedStyle(probe).backgroundColor; probe.remove(); \
                 return [...document.querySelectorAll('[data-select] [data-slot=box]')].flatMap(box => { \
                   const input = box.previousElementSibling, mark = box.querySelector('svg'); \
                   const on = input.checked || input.indeterminate; \
                   const shown = getComputedStyle(mark).opacity === '1'; \
                   const name = input.getAttribute('aria-label'); \
                   return [ \
                     getComputedStyle(box).borderTopColor === canvas ? name + ': no outline' : null, \
                     getComputedStyle(mark).color === canvas ? name + ': a Canvas mark' : null, \
                     on !== shown ? name + ': mark shown ' + shown : null, \
                   ].filter(Boolean); }); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(wrong.is_empty(), "{wrong:?}");
        fixture.close().await.unwrap();
    });
}

/// 1156-1c: Shift adds a column after the sorted ones and ranks both headers.
#[test]
fn a_shift_click_adds_a_sorted_column() {
    block_on(async {
        let fixture = Fixture::open("/table/select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, STOCK).await.unwrap();
        wait_text(page, "#sort", "Stock ascending").await;
        pointer::click_with(page, NAME, SHIFT).await.unwrap();
        wait_text(page, "#sort", "Stock ascending,Name ascending").await;
        // Ties on Stock break by Name: Cherry before Date, Apple before Elder.
        rows(page, ":Banana:0|:Cherry:3|:Date:3|:Apple:12|:Elder:12")
            .await
            .unwrap();
        assert_eq!(count(page, "[aria-sort]").await, 2.0);
        assert_eq!(count(page, "[data-sort-order]").await, 2.0);
        let snapshot = ax::snapshot(page, "table").await.unwrap();
        assert!(snapshot.contains("sort order 2"), "{snapshot}");

        pointer::click_with(page, STOCK, SHIFT).await.unwrap();
        wait_text(page, "#sort", "Stock descending,Name ascending").await;
        pointer::click_with(page, STOCK, SHIFT).await.unwrap();
        wait_text(page, "#sort", "Name ascending").await;
        assert_eq!(count(page, "[data-sort-order]").await, 0.0);

        // The keyboard's way: Shift+Enter on a focused header adds it too.
        page.find_element(STOCK)
            .await
            .unwrap()
            .focus()
            .await
            .unwrap();
        keyboard::press_shift(page, keyboard::ENTER).await.unwrap();
        wait_text(page, "#sort", "Name ascending,Stock ascending").await;

        fixture.console.assert_clean("multi-sorting").unwrap();
        fixture.close().await.unwrap();
    });
}

const SHIFT: i64 = 8;

/// Ends a `touch_down` on `selector` as the browser does when it takes the touch to
/// scroll: CDP refuses a `touchCancel`.
async fn cancel_touch(page: &Page, selector: &str) -> Result<()> {
    let script = format!(
        "document.querySelector({selector:?}).dispatchEvent(\
         new PointerEvent('pointercancel', {{bubbles: true, pointerType: 'touch'}}))"
    );
    page.evaluate(script).await?;
    Ok(())
}

/// Todo 3202: a cancelled touch on a header leaves no mark that makes a later keyboard click add.
#[test]
fn a_cancelled_touch_does_not_make_the_next_key_press_add_a_column() {
    block_on(async {
        let fixture = Fixture::open("/table/select", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        pointer::click(page, STOCK).await.unwrap();
        wait_text(page, "#sort", "Stock ascending").await;
        pointer::touch_down(page, NAME).await.unwrap();
        cancel_touch(page, NAME).await.unwrap();
        page.find_element(NAME)
            .await
            .unwrap()
            .focus()
            .await
            .unwrap();
        keyboard::press(page, keyboard::ENTER).await.unwrap();
        wait_text(page, "#sort", "Name ascending").await;

        fixture.console.assert_clean("a cancelled touch").unwrap();
        fixture.close().await.unwrap();
    });
}

async fn wait_text(page: &Page, selector: &str, expected: &str) {
    wait::for_js_true(
        page,
        &format!("document.querySelector({selector:?}).textContent === {expected:?}"),
        &format!("{selector} to read {expected}"),
    )
    .await
    .unwrap();
}

/// 1156-1b: page buttons turn the page, a sort goes back to page 1, and a new
/// page size keeps the first shown row on screen.
async fn the_pages_turn_sort_and_resize<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const RANGE: &str = "[data-slot=range]";
    const LIVE: &str = "[data-slot=range] + [role=status]";
    const FIRST: &str = "tbody th";
    // Empty until a change: the first range is no news.
    eventually_text(d, LIVE, "", "the first render").await?;
    let shows = async |d: &mut D, range: &str, first: &str, after: &str| -> Result<()> {
        eventually_text(d, RANGE, range, after).await?;
        eventually_text(d, FIRST, first, after).await
    };
    shows(d, "1–3 of 7", "Fig", "the first page").await?;
    d.click("button[aria-label=\"Go to next page\"]").await?;
    shows(d, "4–6 of 7", "Cherry", "Next").await?;
    eventually_text(d, "#page", "2", "onpagechange").await?;
    eventually_text(d, LIVE, "4–6 of 7", "the page turn's announcement").await?;
    d.click(SORT).await?;
    shows(d, "1–3 of 7", "Apple", "a sort").await?;
    eventually_text(d, "#page", "1", "a sort's page reset").await?;
    d.click("button[aria-label=\"Go to page 3\"]").await?;
    shows(d, "7–7 of 7", "Grape", "page 3").await?;
    d.click("[role=combobox]").await?;
    eventually(d, "the page sizes", async |d| {
        d.exists("[role=listbox]").await
    })
    .await?;
    d.click("[role=option]:nth-child(2)").await?;
    shows(d, "6–7 of 7", "Fig", "5 a page").await
}

e2e::scenario!(
    a_paged_table_turns_sorts_and_resizes,
    "/table/paged",
    the_pages_turn_sort_and_resize
);

/// 1156-0b: with `row_key`, a row prepended on top leaves the others' nodes alone.
#[test]
fn a_keyed_row_keeps_its_node_when_a_row_is_prepended() {
    block_on(async {
        let fixture = Fixture::open("/table/rows", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        page.evaluate("document.querySelector('tr[data-name=\"Apple\"]').marked = true")
            .await
            .unwrap();
        pointer::click(page, "#prepend").await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('tbody tr').dataset.name === 'Apricot' \
             && document.querySelector('tr[data-name=\"Apple\"]').marked === true",
            "Apricot on top, Apple on its own node",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("prepending a row").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn it_meets_the_baseline() {
    Suite::new("table", "/table")
        .focusable(SORT)
        .targets(SORT)
        .run();
}

/// Todo 3152: the sort button's padding is least at `xs` and `sm`.
#[test]
fn the_smallest_sizes_meet_the_baseline() {
    Suite::new("table_small", "/table/small")
        .no_snapshot()
        .focusable(SORT)
        .targets(SORT)
        .run();
}

#[test]
fn a_selectable_table_meets_the_baseline() {
    Suite::new("table_select", "/table/select")
        .focusable("tbody td[data-select] input")
        // The box's control, so its own hidden input is no neighbour (as on `Checkbox`).
        .targets_spaced("td[data-select] span:has(> input)")
        .targets(SORT)
        .run();
}

#[test]
fn a_wide_table_meets_the_baseline() {
    Suite::new("table_wide", "/table/wide").run();
}

#[test]
fn a_paged_table_meets_the_baseline() {
    Suite::new("table_paged", "/table/paged").run();
}

#[test]
fn a_header_click_sorts_and_flips_the_rows() {
    block_on(async {
        let fixture = Fixture::open("/table", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;

        rows(page, "Cherry:3 left|Apple:12 left|Banana:0 left")
            .await
            .unwrap();
        click(page).await.unwrap();
        rows(page, "Apple:12 left|Banana:0 left|Cherry:3 left")
            .await
            .unwrap();
        click(page).await.unwrap();
        rows(page, "Cherry:3 left|Banana:0 left|Apple:12 left")
            .await
            .unwrap();
        // The third click unsorts: source order, no `aria-sort`.
        click(page).await.unwrap();
        rows(page, "Cherry:3 left|Apple:12 left|Banana:0 left")
            .await
            .unwrap();
        assert_eq!(count(page, "th[aria-sort]").await, 0.0);

        fixture.console.assert_clean("sorting a table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// The sort button draws the library's ring, not the UA's `auto` outline, and
/// keeps focus while Enter re-sorts the rows under it (todo 449).
#[test]
fn the_sort_button_keeps_focus_and_draws_the_library_ring() {
    block_on(async {
        e2e::browser::at_every_viewport(async |viewport| {
            let at = viewport.name();
            let fixture = Fixture::open("/table", viewport).await.unwrap();
            let page = &fixture.page;

            keyboard::tab_to(page, SORT, 5).await.unwrap();
            let outline: String = page
                .evaluate(format!(
                    "getComputedStyle(document.querySelector({SORT:?})).outlineStyle"
                ))
                .await
                .unwrap()
                .into_value()
                .unwrap();
            assert_eq!(outline, "solid", "at {at}: the sort button's focus ring");

            for expected in [
                "Apple:12 left|Banana:0 left|Cherry:3 left",
                "Cherry:3 left|Banana:0 left|Apple:12 left",
            ] {
                keyboard::press(page, keyboard::ENTER).await.unwrap();
                rows(page, expected)
                    .await
                    .unwrap_or_else(|e| panic!("at {at}: {e}"));
                e2e::passes::focus::assert_focused(page, SORT, "sorting by keyboard")
                    .await
                    .unwrap_or_else(|e| panic!("at {at}: {e}"));
            }

            fixture.console.assert_clean("sorting by keyboard").unwrap();
            fixture.close().await.unwrap();
        })
        .await;
    });
}

/// `aria-sort` sits on the sorted header only; an unsorted sortable header
/// still hints its arrow on hover, via `data-sortable` (todo 587).
#[test]
fn only_the_sorted_header_carries_aria_sort_and_the_others_still_hint() {
    block_on(async {
        let fixture = Fixture::open("/table/wide", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let first = "th[data-sortable]:nth-child(1)";
        let second = "th[data-sortable]:nth-child(2)";

        assert_eq!(count(page, "th[aria-sort]").await, 0.0);
        e2e::passes::pointer::hover(page, second).await.unwrap();
        arrow_opacity(page, second, "0.5").await.unwrap();

        pointer::click(page, &format!("{first} button"))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            &format!("document.querySelector('{first}').getAttribute('aria-sort') === 'ascending'"),
            "the first header to read ascending",
        )
        .await
        .unwrap();
        assert_eq!(count(page, "th[aria-sort]").await, 1.0);
        arrow_opacity(page, first, "1").await.unwrap();
        e2e::passes::pointer::hover(page, second).await.unwrap();
        arrow_opacity(page, second, "0.5").await.unwrap();

        fixture.console.assert_clean("hinting a sort").unwrap();
        fixture.close().await.unwrap();
    });
}

#[test]
fn an_empty_table_shows_its_empty_slot_across_every_column() {
    block_on(async {
        let fixture = Fixture::open("/table/empty", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "(() => { const td = document.querySelector('tbody td'); \
             return !!td && td.colSpan === 2 && td.textContent === 'No fruit'; })()",
            "the empty row to span both columns",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("an empty table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-2a: `width` on the header sizes the column, padding included; the
/// rendered header sorts like a text one.
#[test]
fn a_column_width_and_a_rendered_header_reach_the_browser() {
    block_on(async {
        let fixture = Fixture::open("/table/widths", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        wait::for_js_true(
            page,
            "(() => { const w = s => document.querySelector(s).getBoundingClientRect().width; \
             return Math.abs(w('thead th') - 200) <= 1 && Math.abs(w('tbody td') - 200) <= 1; })()",
            "the first column to be 200px wide",
        )
        .await
        .unwrap();
        assert_eq!(count(page, "th[data-sortable]").await, 1.0);
        pointer::click(page, "thead th:nth-child(2) button")
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "(() => { const th = document.querySelector('thead th:nth-child(2)'); \
             return th.getAttribute('aria-sort') === 'ascending' \
             && th.querySelector('small').textContent === '(boxes)'; })()",
            "the rendered header to sort",
        )
        .await
        .unwrap();

        fixture.console.assert_clean("a sized table").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 3598: on a coarse pointer the column menu button takes presses in a 44px column.
#[test]
fn the_column_menu_button_takes_a_finger_in_a_44px_box() {
    use chromiumoxide::cdp::browser_protocol::emulation::SetTouchEmulationEnabledParams;
    block_on(async {
        let fixture = Fixture::open("/table/menu", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        page.execute(SetTouchEmulationEnabledParams::new(true))
            .await
            .unwrap();
        wait::for_js_true(
            page,
            "matchMedia('(pointer: coarse)').matches",
            "a coarse pointer",
        )
        .await
        .unwrap();

        let misses: Vec<String> = page
            .evaluate(
                "(() => {
                    const el = document.querySelector('button[aria-label=\"Name column options\"]');
                    el.scrollIntoView({ block: 'center' });
                    const r = el.getBoundingClientRect();
                    const cx = r.x + r.width / 2, cy = r.y + r.height / 2;
                    return [-21, 0, 21].flatMap(dy => {
                        const at = document.elementFromPoint(cx, cy + dy);
                        return at && el.contains(at) ? [] : [String(dy)];
                    });
                })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            misses.is_empty(),
            "points of the 44px column land elsewhere: {misses:?}"
        );
        fixture
            .console
            .assert_clean("probing the menu button")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

/// 1156-2b: the column menu sorts, adds to the sort, hides a column and shows
/// it again from the Columns submenu.
#[test]
fn a_column_menu_sorts_hides_and_shows_columns() {
    block_on(async {
        let fixture = Fixture::open("/table/menu", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;

        // Opens `header`'s menu, then clicks the item labelled `item` once it shows.
        async fn pick(page: &Page, header: &str, item: &str) {
            pointer::click(
                page,
                &format!("button[aria-label=\"{header} column options\"]"),
            )
            .await
            .unwrap();
            choose(page, item).await;
        }
        async fn choose(page: &Page, item: &str) {
            let find = format!(
                "[...document.querySelectorAll('[role^=menuitem]')]\
                 .find(e => e.textContent.trim() === {item:?})"
            );
            // A real press at the item's centre, once the placed menu shows it there,
            // scrolled into a menu taller than the viewport.
            let centre = format!(
                "(() => {{ const e = {find}; if (!e) return null; e.scrollIntoView({{ block: 'nearest' }}); \
                 const r = e.getBoundingClientRect(); const [x, y] = [r.x + r.width / 2, r.y + r.height / 2]; \
                 return e.contains(document.elementFromPoint(x, y)) ? [x, y] : null; }})()"
            );
            wait::for_js_true(page, &format!("!!{centre}"), item)
                .await
                .unwrap();
            let (x, y): (f64, f64) = page.evaluate(centre).await.unwrap().into_value().unwrap();
            pointer::click_at(page, pointer::Point { x, y })
                .await
                .unwrap();
        }
        let sort_of = |column: usize| {
            format!(
                "document.querySelector('thead th:nth-child({column})')?.getAttribute('aria-sort')"
            )
        };

        // Todo 1260: with a mouse, the menu button shows on its header's hover or focus.
        const STOCK_MENU: &str = "button[aria-label=\"Stock column options\"]";
        // Headless Chromium has no hover-capable pointer and CDP cannot emulate
        // one: copy the fine-pointer rules out of their query, read the computed opacity.
        wait::for_js_true(
            page,
            "(() => { const all = rs => [...rs].flatMap(r => [r, ...all(r.cssRules ?? [])]); \
             const query = [...document.styleSheets, ...document.adoptedStyleSheets] \
             .flatMap(s => all(s.cssRules)).find(r => r.conditionText?.includes('pointer: fine')); \
             if (!query) return false; \
             const style = document.createElement('style'); \
             style.textContent = [...query.cssRules].map(r => r.cssText).join('\\n'); \
             document.head.append(style); return true; })()",
            "the fine-pointer rules copied out of their query",
        )
        .await
        .unwrap();
        let menu_opacity = |expected: &str| {
            format!(
                "getComputedStyle(document.querySelector({STOCK_MENU:?})).opacity === {expected:?}"
            )
        };
        wait::for_js_true(page, &menu_opacity("0"), "the idle menu button faded")
            .await
            .unwrap();
        page.evaluate(format!("document.querySelector({STOCK_MENU:?}).focus()"))
            .await
            .unwrap();
        wait::for_js_true(page, &menu_opacity("1"), "the focused menu button shown")
            .await
            .unwrap();

        pick(page, "Stock", "Sort descending").await;
        wait::for_js_true(
            page,
            &format!(
                "{} === 'descending' && document.querySelector('tbody th').textContent === 'Apple'",
                sort_of(2)
            ),
            "Stock to sort descending",
        )
        .await
        .unwrap();

        pick(page, "Name", "Add to sort").await;
        wait::for_js_true(
            page,
            &format!(
                "{} === 'ascending' && document.querySelectorAll('[data-sort-order]').length === 2",
                sort_of(1)
            ),
            "Name to sort second",
        )
        .await
        .unwrap();

        pick(page, "Stock", "Hide column").await;
        wait::for_js_true(
            page,
            "document.querySelectorAll('thead th').length === 1 \
             && document.querySelectorAll('tbody td').length === 0",
            "Stock to hide",
        )
        .await
        .unwrap();
        // The hidden column still sorts first.
        assert_eq!(
            page.evaluate("document.querySelector('tbody th').textContent")
                .await
                .unwrap()
                .into_value::<String>()
                .unwrap(),
            "Apple"
        );

        pick(page, "Name", "Columns").await;
        choose(page, "Stock").await;
        wait::for_js_true(
            page,
            "document.querySelectorAll('thead th').length === 2 \
             && document.querySelectorAll('tbody td').length === 3",
            "Stock to show again",
        )
        .await
        .unwrap();

        fixture
            .console
            .assert_clean("a table with column menus")
            .unwrap();
        fixture.close().await.unwrap();
    });
}

async fn count(page: &Page, selector: &str) -> f64 {
    page.evaluate(format!("document.querySelectorAll({selector:?}).length"))
        .await
        .unwrap()
        .into_value()
        .unwrap()
}

async fn arrow_opacity(page: &Page, header: &str, expected: &str) -> Result<()> {
    wait::for_js_true(
        page,
        // Its transition finished at once: a tab behind runs it at ~1 frame/s.
        &format!(
            "(svg => (svg.getAnimations().forEach((a) => a.finish()), getComputedStyle(svg).opacity === {expected:?}))\
             (document.querySelector('{header} svg'))"
        ),
        &format!("{header}'s arrow at opacity {expected}"),
    )
    .await
}

/// Todo 742: the row header column is `th scope="row"`, exposed as a row
/// header, and looks like the cells beside it.
#[test]
fn a_row_header_column_names_its_rows() {
    block_on(async {
        let fixture = Fixture::open("/table", Viewport::Desktop).await.unwrap();
        let page = &fixture.page;
        click(page).await.unwrap();
        rows(page, "Apple:12 left|Banana:0 left|Cherry:3 left")
            .await
            .unwrap();
        let same_look: bool = page
            .evaluate(
                "(() => { const ths = document.querySelectorAll('tbody th[scope=row]'); \
                 const td = document.querySelector('tbody td'); \
                 return ths.length === 3 && getComputedStyle(ths[0]).fontWeight === getComputedStyle(td).fontWeight; })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(same_look, "three row headers, set like the cells");
        let snapshot = ax::snapshot(page, "table").await.unwrap();
        assert_eq!(snapshot.matches("rowheader").count(), 3, "{snapshot}");
        fixture.close().await.unwrap();
    });
}

/// Not `Element::click`: its IntersectionObserver waits a frame, a second in a background tab.
async fn click(page: &Page) -> Result<()> {
    pointer::click(page, SORT).await
}

/// The body reads `name:cell` per row, in order, joined by `|`.
async fn rows(page: &Page, expected: &str) -> Result<()> {
    wait::for_js_true(
        page,
        &format!(
            "(() => [...document.querySelectorAll('tbody tr')]\
             .map(tr => [...tr.cells].map(td => td.textContent).join(':'))\
             .join('|') === {expected:?})()"
        ),
        &format!("the rows to read {expected}"),
    )
    .await
}

/// Waits for the open menu's entry reading `label`; todo 1502: the entries, not only its first.
pub(crate) async fn menu_entry<D: Driver>(d: &mut D, label: &str) -> Result<String> {
    let mut found = None;
    eventually(d, &format!("the menu's {label} entry"), async |d| {
        for index in 0..12 {
            let item = format!("[role=menuitem][data-menu-index=\"{index}\"]");
            if d.exists(&item).await? && d.text(&item).await?.trim() == label {
                found = Some(item);
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await?;
    Ok(found.expect("the wait held"))
}
