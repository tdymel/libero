//! `Table` row reorder (1156-4b): move buttons, keyboard and handle drag
//! reorder `data`, off while sorted. Column order (4c): a column menu's Move
//! right moves the column, and its sort with it.

use anyhow::{Result, bail};
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;

const ORDER: &str = "#order";

fn handle(name: &str) -> String {
    format!("button[aria-label=\"Reorder {name}\"]")
}

async fn buttons_move_a_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let down = "button[aria-label=\"Move Cherry down\"]";
    // Disabled at the end once the rows have counted themselves.
    eventually(d, "the last row's Move down to disable", async |d| {
        Ok(
            d.attr("button[aria-label=\"Move Damson down\"]", "disabled")
                .await?
                .is_some(),
        )
    })
    .await?;
    d.click(down).await?;
    eventually_text(d, ORDER, "Apple Cherry Banana Damson", "Move Cherry down").await?;
    eventually_text(d, "#moves", "0>1 ", "one move by data index").await?;
    // The button keeps the focus through its row's move in the DOM.
    eventually_focused(d, down, "Move down").await?;
    eventually_text(
        d,
        "tbody tr:nth-child(2) th",
        "Cherry",
        "Cherry's row moved",
    )
    .await
}

e2e::scenario!(
    a_move_button_moves_its_row,
    "/table-reorder",
    buttons_move_a_row,
    android: skip("958: element identity on the WebView")
);

async fn keys_move_a_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let damson = handle("Damson");
    d.focus(&damson).await?;
    eventually_focused(d, &damson, "focus").await?;
    d.press(keyboard::SPACE).await?;
    eventually(d, "a lift announced", async |d| {
        Ok(!d.text("[role=status]").await?.is_empty())
    })
    .await?;
    d.press(keyboard::ARROW_UP).await?;
    d.press(keyboard::ARROW_UP).await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(
        d,
        ORDER,
        "Cherry Damson Apple Banana",
        "Space, Up, Up, Space",
    )
    .await?;
    eventually_focused(d, &damson, "the drop").await
}

e2e::scenario!(
    the_keyboard_lifts_and_moves_a_row,
    "/table-reorder",
    keys_move_a_row,
    android: skip("958: element identity on the WebView")
);

async fn a_drag_moves_a_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let (first, second) = (
        d.rect("tbody tr:nth-child(1)").await?,
        d.rect("tbody tr:nth-child(2)").await?,
    );
    let pitch = second.y - first.y;
    d.drag(&handle("Cherry"), 0.0, pitch * 2.0).await?;
    eventually_text(
        d,
        ORDER,
        "Apple Banana Cherry Damson",
        "a drag two rows down",
    )
    .await?;
    eventually_text(d, "#moves", "0>2 ", "one move").await?;
    // No offset left: the row sits in its slot.
    let row = d.rect("tbody tr:nth-child(3)").await?;
    if (row.y - first.y - 2.0 * pitch).abs() > 1.0 {
        bail!(
            "the dropped row sits at {}, its slot at {}",
            row.y,
            first.y + 2.0 * pitch
        );
    }
    Ok(())
}

e2e::scenario!(
    a_handle_drag_moves_a_row,
    "/table-reorder",
    a_drag_moves_a_row,
    native: skip("Blitz paints no `transform` on a `tr`; no native pointer row drag by decision (1156 Phase 4b), keyboard lift and Move buttons cover it")
);

/// Cherry's open detail drags with it: one row's travel past Apple moves it,
/// though the detail sits between them (1397).
async fn a_detail_drags_with_its_row<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let apple = d.rect("tbody tr:nth-child(3)").await?;
    d.drag(&handle("Cherry"), 0.0, apple.height).await?;
    eventually_text(
        d,
        ORDER,
        "Apple Cherry Banana Damson",
        "a drag one row down",
    )
    .await?;
    eventually(d, "the detail right under its row, at rest", async |d| {
        let row = d.rect("tbody tr:nth-child(2)").await?;
        let detail = d.rect("tbody tr[data-detail]").await?;
        Ok(d.text("tbody tr:nth-child(2) th").await? == "Cherry"
            && (detail.y - (row.y + row.height)).abs() <= 1.0)
    })
    .await
}

e2e::scenario!(
    an_open_detail_drags_with_its_row,
    "/table-reorder-detail",
    a_detail_drags_with_its_row,
    native: skip("Blitz paints no `transform` on a `tr`; no native pointer row drag by decision (1156 Phase 4b), keyboard lift and Move buttons cover it")
);

async fn sorted_is_off<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("th [data-sort-button]").await?;
    eventually_text(d, "tbody tr:first-child th", "Apple", "the sort").await?;
    // Todo 1430: off, yet a Tab stop that says why.
    for selector in [
        handle("Banana"),
        "button[aria-label=\"Move Banana up\"]".into(),
    ] {
        eventually(d, &format!("{selector} to be off"), async |d| {
            Ok(d.attr(&selector, "aria-disabled").await?.as_deref() == Some("true"))
        })
        .await?;
        if d.attr(&selector, "disabled").await?.is_some() {
            bail!("{selector} left the Tab order while sorted");
        }
        let reason = d
            .attr(&selector, "aria-describedby")
            .await?
            .unwrap_or_default();
        if d.text(&format!("[id=\"{reason}\"]")).await?
            != "Clear the sort and filters to reorder rows"
        {
            bail!("{selector} is described by {reason:?}, not the reason");
        }
    }
    // Neither a click, a drag nor Space moves an off row.
    d.click("button[aria-label=\"Move Banana up\"]").await?;
    let pitch = d.rect("tbody tr:nth-child(2)").await?.y - d.rect("tbody tr:nth-child(1)").await?.y;
    d.drag(&handle("Banana"), 0.0, pitch * 2.0).await?;
    d.focus(&handle("Banana")).await?;
    d.press(keyboard::SPACE).await?;
    if d.text(ORDER).await? != "Cherry Apple Banana Damson" || !d.text("#moves").await?.is_empty() {
        bail!("an off control or the sort moved a row");
    }
    Ok(())
}

e2e::scenario!(
    row_reorder_is_off_while_sorted,
    "/table-reorder",
    sorted_is_off
);

/// The menu of Name, sorted: sort ascending, descending, unsort, Filter, the moves.
async fn a_menu_moves_a_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("th[aria-label=Name] [data-sort-button]").await?;
    eventually(d, "Name sorted", async |d| {
        Ok(d.attr("th[aria-label=Name]", "aria-sort").await?.is_some())
    })
    .await?;
    d.click("button[aria-label=\"Name column options\"]")
        .await?;
    let right = "[role=menuitem][data-menu-index=\"5\"]";
    eventually(d, "the column menu to open", async |d| {
        d.exists(right).await
    })
    .await?;
    if d.text(right).await? != "Move right" {
        bail!("entry 5 reads {:?}", d.text(right).await?);
    }
    d.click(right).await?;
    eventually_text(d, "#columns", "Stock Name Origin", "Move right").await?;
    // Todo 1428: the closed menu hides the move, so it is said.
    eventually_text(
        d,
        "[role=status]",
        "Name moved to column 2 of 3",
        "the move announcement",
    )
    .await?;
    eventually(d, "Name to sit right of Stock, still sorted", async |d| {
        Ok(
            d.rect("th[aria-label=Name]").await?.x > d.rect("th[aria-label=Stock]").await?.x
                && d.attr("th[aria-label=Name]", "aria-sort").await?.is_some()
                && d.attr("th[aria-label=Stock]", "aria-sort").await?.is_none(),
        )
    })
    .await?;
    // The cells moved with their header.
    eventually_text(
        d,
        "tbody tr:first-child td:nth-child(2)",
        "12",
        "Apple's stock first",
    )
    .await
}

e2e::scenario!(
    a_column_menu_moves_a_column_and_its_sort,
    "/table-reorder",
    a_menu_moves_a_column
);

fn menu_button(header: &str) -> String {
    format!("button[aria-label=\"{header} column options\"]")
}

/// Opens `header`'s menu from the keyboard and picks `entry` with the arrows.
async fn pick_by_keys<D: Driver>(d: &mut D, header: &str, entry: &str) -> Result<()> {
    pick_from(d, &menu_button(header), entry).await
}

/// [`pick_by_keys`] from the menu `button`, whatever language names it.
async fn pick_from<D: Driver>(d: &mut D, button: &str, entry: &str) -> Result<()> {
    d.focus(button).await?;
    eventually_focused(d, button, "the menu button").await?;
    d.press(keyboard::ENTER).await?;
    // Todo 1502: the entry itself, not only the menu's first.
    let mut item = None;
    eventually(
        d,
        &format!("{entry:?} in the menu of {button}"),
        async |d| {
            for index in 0..12 {
                let at = format!("[role=menuitem][data-menu-index=\"{index}\"]");
                if d.exists(&at).await? && d.text(&at).await? == entry {
                    item = Some(at);
                    return Ok(true);
                }
            }
            Ok(false)
        },
    )
    .await?;
    let item = item.expect("the wait held");
    for _ in 0..12 {
        if d.is_focused(&item).await? {
            d.press(keyboard::ENTER).await?;
            return Ok(());
        }
        d.press(keyboard::ARROW_DOWN).await?;
    }
    bail!("the arrows never reached {entry:?}")
}

/// TM1 (todo 1427): a pin that moves the header keeps focus on its menu
/// button; Hide sends it to the next column's.
async fn menu_picks_keep_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    pick_by_keys(d, "Name", "Pin to end").await?;
    eventually(d, "Name to pin at the end", async |d| {
        Ok(d.rect("th[aria-label=Name]").await?.x > d.rect("th[aria-label=Origin]").await?.x)
    })
    .await?;
    eventually_focused(d, &menu_button("Name"), "Pin to end").await?;
    // Todo 1428: pins and hides are said.
    eventually_text(
        d,
        "[role=status]",
        "Name pinned to end",
        "the pin announcement",
    )
    .await?;
    pick_by_keys(d, "Stock", "Hide column").await?;
    eventually(d, "Stock to hide", async |d| {
        Ok(!d.exists("th[aria-label=Stock]").await?)
    })
    .await?;
    eventually_focused(d, &menu_button("Origin"), "Hide column").await?;
    eventually_text(d, "[role=status]", "Stock hidden", "the hide announcement").await
}

e2e::scenario!(
    a_pin_or_hide_from_the_menu_keeps_focus_on_a_menu_button,
    "/table-reorder",
    menu_picks_keep_focus,
    android: skip("958: element identity on the WebView")
);

/// Todo 2049: the column menu's move, pin, unpin and hide are said in German too.
async fn german_menu_announcements<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let button = |header: &str| format!("button[aria-label=\"Optionen für Spalte {header}\"]");
    for (header, entry, said) in [
        ("Name", "Nach rechts", "Name an Spalte 2 von 3 verschoben"),
        ("Name", "Am Ende fixieren", "Name am Ende fixiert"),
        ("Name", "Am Anfang fixieren", "Name am Anfang fixiert"),
        ("Name", "Fixierung lösen", "Fixierung von Name gelöst"),
        ("Stock", "Spalte ausblenden", "Stock ausgeblendet"),
    ] {
        pick_from(d, &button(header), entry).await?;
        eventually_text(d, "[role=status]", said, entry).await?;
    }
    Ok(())
}

e2e::scenario!(
    the_column_menu_announces_in_german,
    "/table-reorder-de",
    german_menu_announcements,
    android: skip("958: element identity on the WebView")
);

/// Todo 1490: a column hidden from its own menu's Columns submenu sends focus
/// to the neighbour's menu button, as Hide column does.
async fn hiding_its_own_column_keeps_focus<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    pick_by_keys(d, "Origin", "Columns").await?;
    let mut toggle = None;
    eventually(d, "Origin in the Columns submenu", async |d| {
        for index in 0..12 {
            let at = format!("[role=menuitemcheckbox][data-menu-index=\"{index}\"]");
            if d.exists(&at).await? && d.text(&at).await?.trim() == "Origin" {
                toggle = Some(at);
                return Ok(true);
            }
        }
        Ok(false)
    })
    .await?;
    let toggle = toggle.expect("the wait held");
    for _ in 0..12 {
        if d.is_focused(&toggle).await? {
            break;
        }
        d.press(keyboard::ARROW_DOWN).await?;
    }
    eventually_focused(d, &toggle, "the arrows in Columns").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "Origin to hide", async |d| {
        Ok(!d.exists("th[aria-label=Origin]").await?)
    })
    .await?;
    eventually_focused(d, &menu_button("Stock"), "Origin unticked in Columns").await
}

/// The lifted row shows whole in the scroll area, below its sticky header.
async fn shown_below_the_header<D: Driver>(d: &mut D, row: &str, during: &str) -> Result<()> {
    eventually(d, &format!("{row} in view after {during}"), async |d| {
        let (area, header) = (
            d.rect("[data-table-scroll]").await?,
            d.rect("thead th").await?,
        );
        let row = d.rect(row).await?;
        Ok(row.y >= header.y + header.height - 1.0
            && row.y + row.height <= area.y + area.height + 1.0)
    })
    .await
}

/// Todo 1408: a windowed table reorders. A move button steps; a keyboard lift
/// carries the view past the window's edge, End to the last slot, Home back.
async fn a_windowed_table_reorders<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const LIVE: &str = "table + [role=status]";
    const HANDLE: &str = "[aria-label=\"Reorder Row 1\"]";
    d.click("[aria-label=\"Move Row 1 down\"]").await?;
    eventually_text(d, "#moves", "0>1 ", "Move down").await?;
    d.click("[aria-label=\"Move Row 1 up\"]").await?;
    eventually_text(d, "#moves", "0>1 1>0 ", "Move up").await?;

    d.focus(HANDLE).await?;
    eventually_focused(d, HANDLE, "the handle").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, LIVE, "Lifted Row 1, position 1 of 200.", "Space").await?;
    d.press(keyboard::END).await?;
    eventually_text(d, LIVE, "Row 1 moved to position 200 of 200.", "End").await?;
    // The window follows the view: Row 200 steps up out of the last slot, Row 1 into it.
    eventually(d, "Row 200 just above the lifted Row 1", async |d| {
        let last = "[aria-label=\"Reorder Row 200\"]";
        Ok(d.exists(last).await?
            && (d.rect(HANDLE).await?.y - d.rect(last).await?.y - 40.0).abs() <= 1.0)
    })
    .await?;
    shown_below_the_header(d, HANDLE, "End").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#moves", "0>1 1>0 0>199 ", "the drop").await?;
    eventually_text(d, "#ends", "Row 2 Row 1", "the drop").await?;
    eventually_focused(d, HANDLE, "the drop").await?;

    d.press(keyboard::SPACE).await?;
    eventually_text(
        d,
        LIVE,
        "Lifted Row 1, position 200 of 200.",
        "a second Space",
    )
    .await?;
    d.press(keyboard::HOME).await?;
    eventually_text(d, LIVE, "Row 1 moved to position 1 of 200.", "Home").await?;
    shown_below_the_header(d, HANDLE, "Home").await?;
    // The lifted row shows by its transform before the window follows the scroll.
    eventually(d, "the window back at the top after Home", async |d| {
        d.exists("[aria-label=\"Reorder Row 2\"]").await
    })
    .await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, "#ends", "Row 1 Row 200", "the second drop").await?;
    eventually_focused(d, HANDLE, "the second drop").await
}

e2e::scenario!(
    a_windowed_table_reorders_past_its_window,
    "/table-reorder-windowed",
    a_windowed_table_reorders,
    android: skip("958: element identity on the WebView"),
    native: skip("1520/2038: Blitz paints no `transform` on a `tr`, so the lifted row keeps its slot")
);

/// Todo 1872: Row 1's grip held at the bottom edge scrolls the windowed rows on,
/// past the window's rows, and drops there.
#[test]
fn a_windowed_row_held_at_the_bottom_edge_scrolls_on() {
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::passes::pointer::{self, Point};
    use e2e::wait;
    const AREA: &str = "document.querySelector('[data-table-scroll]')";
    block_on(async {
        let fixture = Fixture::open("/table-reorder-windowed", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let bottom: f64 = page
            .evaluate(format!("{AREA}.getBoundingClientRect().bottom"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        let from = pointer::centre_of(page, "[aria-label=\"Reorder Row 1\"]")
            .await
            .unwrap();
        let edge = Point {
            x: from.x,
            y: bottom - 4.0,
        };
        pointer::drag_held(page, from, edge, 10).await.unwrap();
        // 20 rows past the 240px view: well beyond the rows rendered at the start.
        wait::for_js_true(
            page,
            &format!("{AREA}.scrollTop > 800"),
            "a grip held at the bottom edge to scroll the rows on",
        )
        .await
        .unwrap();
        pointer::release(page, edge).await.unwrap();
        wait::for_js_true(
            page,
            "(() => { const m = /^0>(\\d+) $/.exec(document.querySelector('#moves').textContent); \
             return !!m && +m[1] > 20; })()",
            "a drop at the slot the scroll brought in",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("an edge row drag").unwrap();
        fixture.close().await.unwrap();
    });
}

e2e::scenario!(
    hiding_a_column_from_its_own_columns_submenu_keeps_focus,
    "/table-reorder",
    hiding_its_own_column_keeps_focus,
    android: skip("958: element identity on the WebView")
);

/// Todo 1483: every draggable header's grip owns a 24px lane, its menu button
/// and label past it, an end-aligned one's too.
async fn grips_keep_their_lane<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    for header in ["Name", "Stock", "Origin"] {
        let cell = format!("th[aria-label={header}]");
        let grip = d.rect(&format!("{cell} [data-drag-handle]")).await?;
        let menu = d.rect(&format!("{cell} [data-column-menu]")).await?;
        let label = d.rect(&format!("{cell} [data-sort-button]")).await?;
        let lane = grip.x + grip.width;
        if grip.width < 23.5 || menu.x < lane - 0.5 || label.x < lane - 0.5 {
            bail!(
                "{header} ({:?}): grip {grip:?}, menu {menu:?}, label {label:?}",
                d.attr(&cell, "data-align").await?
            );
        }
    }
    Ok(())
}

e2e::scenario!(
    a_header_grip_has_its_own_lane,
    "/table-column-drag",
    grips_keep_their_lane,
    native: skip("no column drag grip on Blitz: the column menu moves columns")
);

/// Name's header grip dropped near Origin's end edge: one order change (1395).
async fn a_grip_drag_moves_a_column<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let grip = "th[aria-label=Name] [data-drag-handle]";
    let (from, origin) = (d.rect(grip).await?, d.rect("th[aria-label=Origin]").await?);
    let dx = origin.x + origin.width - 4.0 - (from.x + from.width / 2.0);
    d.drag(grip, dx, 0.0).await?;
    eventually_text(d, "#columns", "Stock Origin Name", "a drag past Origin").await?;
    eventually(d, "Name's cells to follow, the ghost gone", async |d| {
        Ok(d.text("tbody tr:first-child th").await? == "Cherry"
            && d.rect("th[aria-label=Name]").await?.x > d.rect("th[aria-label=Origin]").await?.x
            && !d.exists("[data-drag-ghost]").await?)
    })
    .await
}

e2e::scenario!(
    a_header_grip_drag_moves_a_column,
    "/table-reorder",
    a_grip_drag_moves_a_column,
    native: skip("no column drag grip on Blitz: the column menu moves columns")
);

/// Drags `header`'s grip so its centre lands at client `x`.
async fn drag_grip_to<D: Driver>(d: &mut D, header: &str, x: f64) -> Result<()> {
    let grip = format!("th[aria-label={header}] [data-drag-handle]");
    let from = d.rect(&grip).await?;
    d.drag(&grip, x - (from.x + from.width / 2.0), 0.0).await
}

/// A second drag after the first, of the moved column and then another (1463).
async fn two_grip_drags_move_columns<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let origin = d.rect("th[aria-label=Origin]").await?;
    drag_grip_to(d, "Name", origin.x + origin.width - 4.0).await?;
    eventually_text(d, "#columns", "Stock Origin Name", "the first drag").await?;
    eventually(d, "the ghost gone", async |d| {
        Ok(!d.exists("[data-drag-ghost]").await?)
    })
    .await?;
    let stock = d.rect("th[aria-label=Stock]").await?;
    drag_grip_to(d, "Name", stock.x + 4.0).await?;
    eventually_text(d, "#columns", "Name Stock Origin", "Name dragged back").await?;
    let origin = d.rect("th[aria-label=Origin]").await?;
    drag_grip_to(d, "Stock", origin.x + origin.width - 4.0).await?;
    eventually_text(d, "#columns", "Name Origin Stock", "Stock past Origin").await?;
    eventually(d, "headers and cells in the new order", async |d| {
        let (name, origin, stock) = (
            d.rect("th[aria-label=Name]").await?.x,
            d.rect("th[aria-label=Origin]").await?.x,
            d.rect("th[aria-label=Stock]").await?.x,
        );
        Ok(name < origin
            && origin < stock
            && d.text("tbody tr:first-child th").await? == "Cherry"
            && !d.exists("[data-drag-ghost]").await?)
    })
    .await
}

e2e::scenario!(
    header_grip_drags_move_columns_twice,
    "/table-reorder",
    two_grip_drags_move_columns,
    native: skip("no column drag grip on Blitz: the column menu moves columns")
);

e2e::scenario!(
    header_grip_drags_move_its_own_columns_twice,
    "/table-column-drag",
    two_grip_drags_move_columns,
    native: skip("no column drag grip on Blitz: the column menu moves columns"),
    android: skip("1463: a touch on an end-aligned header's grip at times snaps to its menu button")
);

/// Name's grip held at the scroll region's end edge scrolls Origin's end into
/// view, then drops after it (1463, the edge scroll as Kanban's 1364).
#[test]
fn a_grip_held_at_the_region_edge_scrolls_to_the_last_gap() {
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::passes::pointer::{self, Point};
    use e2e::wait;
    const REGION: &str = "document.querySelector('[data-table-scroll]').getBoundingClientRect()";
    const ORIGIN: &str = "document.querySelector('th[aria-label=Origin]').getBoundingClientRect()";
    block_on(async {
        let fixture = Fixture::open("/table-column-scroll", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let ends: Vec<f64> = page
            .evaluate(format!("[{REGION}.right, {ORIGIN}.right]"))
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert!(
            ends[1] > ends[0] + 50.0,
            "Origin's end starts in view: {ends:?}"
        );
        let from = pointer::centre_of(page, "th[aria-label=Name] [data-drag-handle]")
            .await
            .unwrap();
        let edge = Point {
            x: ends[0] - 6.0,
            y: from.y,
        };
        pointer::drag_held(page, from, edge, 10).await.unwrap();
        wait::for_js_true(
            page,
            &format!("Math.abs({ORIGIN}.right - {REGION}.right) < 2"),
            "a grip held at the end edge to scroll the region to its end",
        )
        .await
        .unwrap();
        pointer::release(page, edge).await.unwrap();
        wait::for_js_true(
            page,
            "document.querySelector('#columns').textContent === 'Stock Origin Name'",
            "a drop at the gap the scroll brought in",
        )
        .await
        .unwrap();
        fixture.console.assert_clean("an edge column drag").unwrap();
        fixture.close().await.unwrap();
    });
}

/// Todo 1450: held mid-drag, the ghost header and the drop line are each the
/// top box near their leading edge, so nothing covers or clips them.
#[test]
fn the_ghost_and_the_drop_line_show_mid_drag() {
    use e2e::browser::{Fixture, Viewport, block_on};
    use e2e::passes::pointer;
    block_on(async {
        let fixture = Fixture::open("/table-reorder", Viewport::Desktop)
            .await
            .unwrap();
        let page = &fixture.page;
        let grip = "th[aria-label=Name] [data-drag-handle]";
        let from = pointer::centre_of(page, grip).await.unwrap();
        let origin = pointer::centre_of(page, "th[aria-label=Origin]")
            .await
            .unwrap();
        pointer::drag_held(page, from, origin, 8).await.unwrap();
        // Hit-testable for the probe only: both are `pointer-events: none`.
        let state: String = page
            .evaluate(
                "(() => { const top = s => { const el = document.querySelector(s); \
                 if (!el) return 'missing'; const r = el.getBoundingClientRect(); \
                 el.style.pointerEvents = 'auto'; \
                 const hit = document.elementFromPoint(r.x + Math.min(r.width / 2, 8), r.y + r.height / 2); \
                 el.style.pointerEvents = ''; \
                 return hit === el ? 'top' : 'under ' + (hit && hit.outerHTML.slice(0, 80)); }; \
                 return top('[data-drag-ghost]') + '|' + top('[data-drop-line]'); })()",
            )
            .await
            .unwrap()
            .into_value()
            .unwrap();
        if state != "top|top" {
            let shot = fixture.screenshot("table-column-drag-held").await.unwrap();
            panic!("ghost|drop line mid-drag: {state} ({})", shot.display());
        }
        pointer::release(page, origin).await.unwrap();
        fixture.console.assert_clean("a held column drag").unwrap();
        fixture.close().await.unwrap();
    });
}
