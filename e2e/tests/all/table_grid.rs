//! `Table` with `cell_focus` (1156 D1): one body cell is the tab stop, and the
//! arrows, Home, End and the page keys move it.

use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, ax};

const STOP: &str = "[data-cell-stop]";
const AREA: &str = "[data-table-scroll]";

/// The body cell in row `row`, column `col`, both from 1.
fn cell(row: usize, col: usize) -> String {
    format!("tbody tr:nth-child({row}) > :nth-child({col})")
}

async fn at<D: Driver>(d: &mut D, row: usize, col: usize, after: &str) -> Result<()> {
    eventually_focused(d, &cell(row, col), after).await
}

async fn arrows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(STOP).await?;
    at(d, 1, 1, "focus on the stop").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    at(d, 1, 2, "ArrowRight").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    at(d, 2, 2, "ArrowDown").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    at(d, 2, 1, "ArrowLeft").await?;
    // The edge holds: no wrap.
    d.press(keyboard::ARROW_LEFT).await?;
    d.press(keyboard::ARROW_UP).await?;
    at(d, 1, 1, "ArrowLeft at the edge, then ArrowUp").await?;
    d.press(keyboard::ARROW_UP).await?;
    at(d, 1, 1, "ArrowUp at the top").await?;
    for (row, col, tabindex) in [(1, 1, "0"), (2, 2, "-1"), (1, 2, "-1")] {
        if d.attr(&cell(row, col), "tabindex").await?.as_deref() != Some(tabindex) {
            bail!("cell {row},{col} is not tabindex {tabindex}");
        }
    }
    Ok(())
}

e2e::scenario!(
    arrows_move_the_cell_stop,
    "/table-grid",
    arrows,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

async fn home_end<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(STOP).await?;
    at(d, 1, 1, "focus on the stop").await?;
    d.press(keyboard::END).await?;
    at(d, 1, 3, "End").await?;
    d.press_ctrl(keyboard::END).await?;
    at(d, 5, 3, "Ctrl+End").await?;
    d.press(keyboard::HOME).await?;
    at(d, 5, 1, "Home").await?;
    d.press_ctrl(keyboard::HOME).await?;
    at(d, 1, 1, "Ctrl+Home").await
}

e2e::scenario!(
    home_end_and_ctrl_home_end,
    "/table-grid",
    home_end,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

async fn click_moves<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(&cell(3, 2)).await?;
    at(d, 3, 2, "a click").await?;
    eventually(d, "the clicked cell to be the stop", async |d| {
        Ok(
            d.attr(&cell(3, 2), "tabindex").await?.as_deref() == Some("0")
                && d.attr(&cell(1, 1), "tabindex").await?.as_deref() == Some("-1"),
        )
    })
    .await
}

e2e::scenario!(a_click_moves_the_stop, "/table-grid", click_moves);

/// Shift+Tab from after the table lands on the stop, which keeps its place after Tab leaves.
async fn one_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#after").await?;
    eventually_focused(d, "#after", "focus").await?;
    d.press_shift(keyboard::TAB).await?;
    at(d, 1, 1, "Shift+Tab into the grid").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    at(d, 2, 2, "ArrowDown, ArrowRight").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, "#after", "Tab out of the grid").await?;
    d.press_shift(keyboard::TAB).await?;
    at(d, 2, 2, "Shift+Tab back to the moved stop").await
}

e2e::scenario!(
    tab_enters_and_leaves_in_one_stop,
    "/table-grid",
    one_stop,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Ben's age stays the stop when the age sort moves his row from second to fourth.
async fn sort_keeps<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(&cell(2, 2)).await?;
    at(d, 2, 2, "a click on Ben's age").await?;
    d.click("th:nth-child(2) [data-sort-button]").await?;
    eventually_text(d, &cell(4, 1), "Ben", "the age sort").await?;
    eventually_text(d, STOP, "42", "the stop after the sort").await?;
    if d.attr(&cell(4, 2), "tabindex").await?.as_deref() != Some("0") {
        bail!("Ben's age, now in the fourth row, is not the stop");
    }
    Ok(())
}

e2e::scenario!(a_sort_keeps_the_focused_cell, "/table-grid", sort_keeps);

/// A button in a cell stays a Tab stop; an arrow from it moves to the next cell.
async fn from_a_widget<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const GREET: &str = "tbody tr:nth-child(1) button";
    d.focus(STOP).await?;
    at(d, 1, 1, "focus on the stop").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, GREET, "Tab to the button").await?;
    eventually(d, "the button's cell to be the stop", async |d| {
        Ok(d.attr(&cell(1, 4), "tabindex").await?.as_deref() == Some("0"))
    })
    .await?;
    d.press(keyboard::ARROW_DOWN).await?;
    at(d, 2, 4, "ArrowDown from the button").await?;
    d.press(keyboard::ARROW_LEFT).await?;
    at(d, 2, 3, "ArrowLeft").await
}

e2e::scenario!(
    an_arrow_leaves_a_widget_for_the_next_cell,
    "/table-grid/widgets",
    from_a_widget,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// The stop cell's number, the row's id in the windowed grid.
async fn stop_id<D: Driver>(d: &mut D) -> Result<u32> {
    Ok(d.text(STOP).await?.trim().parse()?)
}

/// 300px of 40px rows under a header: a view is about six rows.
async fn page_down<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(STOP).await?;
    eventually_focused(d, STOP, "focus on the stop").await?;
    d.press(keyboard::PAGE_DOWN).await?;
    eventually(d, "PageDown to move a view", async |d| {
        Ok(stop_id(d).await? > 1 && d.is_focused(STOP).await?)
    })
    .await?;
    let moved = stop_id(d).await?;
    if !(4..=9).contains(&moved) {
        bail!("PageDown moved to row {moved}, not about one view down");
    }
    d.press(keyboard::PAGE_UP).await?;
    eventually(d, "PageUp back to the first row", async |d| {
        Ok(stop_id(d).await? == 1 && d.is_focused(STOP).await?)
    })
    .await
}

e2e::scenario!(
    page_down_moves_a_view,
    "/table-grid/windowed",
    page_down,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Ctrl+End reaches the last row's last cell (500 squared), out of the window, and shows
/// it; Ctrl+Home comes back.
async fn reveals<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(STOP).await?;
    eventually_focused(d, STOP, "focus on the stop").await?;
    d.press_ctrl(keyboard::END).await?;
    eventually(d, "Ctrl+End on the last row, shown", async |d| {
        if stop_id(d).await? != 250_000 || !d.is_focused(STOP).await? {
            return Ok(false);
        }
        let (cell, area) = (d.rect(STOP).await?, d.rect(AREA).await?);
        Ok(cell.y >= area.y - 1.0 && cell.y + cell.height <= area.y + area.height + 1.0)
    })
    .await?;
    d.press_ctrl(keyboard::HOME).await?;
    eventually(
        d,
        "Ctrl+Home on the first row, below the header",
        async |d| {
            if stop_id(d).await? != 1 || !d.is_focused(STOP).await? {
                return Ok(false);
            }
            let (cell, head) = (d.rect(STOP).await?, d.rect("thead").await?);
            Ok(cell.y >= head.y + head.height - 1.0)
        },
    )
    .await
}

e2e::scenario!(
    a_windowed_grid_scrolls_to_the_focused_cell,
    "/table-grid/windowed",
    reveals,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

#[test]
fn the_grid_meets_the_baseline() {
    Suite::new("table_grid", "/table-grid")
        .focusable(STOP)
        .run();
}

/// Chromium reads a `role="grid"` table's cells as grid cells, its row headers as such.
#[test]
fn the_ax_tree_names_a_grid() {
    block_on(async {
        let fixture = Fixture::open("/table-grid", Viewport::Desktop)
            .await
            .unwrap();
        let tree = ax::snapshot(&fixture.page, "table").await.unwrap();
        assert!(tree.starts_with("grid \"Team\""), "{tree}");
        assert!(tree.contains("rowheader \"Ana\""), "{tree}");
        assert!(tree.contains("gridcell \"31\""), "{tree}");
        assert!(tree.contains("columnheader"), "{tree}");
        fixture.console.assert_clean("reading the grid").unwrap();
        fixture.close().await.unwrap();
    });
}
