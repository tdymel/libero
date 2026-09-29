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
    a_drag_moves_a_row
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
    a_detail_drags_with_its_row
);

async fn sorted_is_off<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click("th [data-sort-button]").await?;
    eventually_text(d, "tbody tr:first-child th", "Apple", "the sort").await?;
    for selector in [
        handle("Banana"),
        "button[aria-label=\"Move Banana up\"]".into(),
    ] {
        if d.attr(&selector, "disabled").await?.is_none() {
            bail!("{selector} is enabled while sorted");
        }
    }
    if d.text(ORDER).await? != "Cherry Apple Banana Damson" {
        bail!("the sort changed the data");
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
    android: skip("1463: touch adjustment at times snaps the 12px grip to a header button")
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
