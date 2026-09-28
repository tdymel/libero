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
