//! `Table` with `cell_focus` (1156 D1-D5): one header or body cell is the tab stop,
//! the first header cell at first, and the arrows, Home, End and the page keys move it.

use anyhow::{Result, bail};
use e2e::browser::block_on;
use e2e::driver::{Driver, Platform, eventually, eventually_focused, eventually_text};
use e2e::passes::keyboard;
use e2e::{Fixture, Suite, Viewport, ax};

const STOP: &str = "[data-cell-stop]";
const AREA: &str = "[data-table-scroll]";
const STATUS: &str = "[role=status]";

/// The body cell in row `row`, column `col`, both from 1.
fn cell(row: usize, col: usize) -> String {
    format!("tbody tr:nth-child({row}) > :nth-child({col})")
}

/// The header cell of column `col`, from 1.
fn head(col: usize) -> String {
    format!("thead tr > :nth-child({col})")
}

async fn at<D: Driver>(d: &mut D, row: usize, col: usize, after: &str) -> Result<()> {
    eventually_focused(d, &cell(row, col), after).await
}

/// Focus on the stop, the first header cell, then ArrowDown into the first row.
async fn into_body<D: Driver>(d: &mut D, col: usize) -> Result<()> {
    d.focus(STOP).await?;
    eventually_focused(d, &head(col), "focus on the stop, the first header cell").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    at(d, 1, col, "ArrowDown from the header").await
}

async fn arrows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    into_body(d, 1).await?;
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
    for (row, col, tabindex) in [(1, 1, "0"), (2, 2, "-1"), (1, 2, "-1")] {
        if d.attr(&cell(row, col), "tabindex").await?.as_deref() != Some(tabindex) {
            bail!("cell {row},{col} is not tabindex {tabindex}");
        }
    }
    d.press(keyboard::ARROW_UP).await?;
    eventually_focused(d, &head(1), "ArrowUp from the first row to the header").await?;
    d.press(keyboard::ARROW_UP).await?;
    eventually_focused(d, &head(1), "ArrowUp at the top").await
}

e2e::scenario!(
    arrows_move_the_cell_stop,
    "/table-grid",
    arrows,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

async fn home_end<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    into_body(d, 1).await?;
    d.press(keyboard::END).await?;
    at(d, 1, 3, "End").await?;
    d.press_ctrl(keyboard::END).await?;
    at(d, 5, 3, "Ctrl+End").await?;
    d.press(keyboard::HOME).await?;
    at(d, 5, 1, "Home").await?;
    // The header row is the grid's first.
    d.press_ctrl(keyboard::HOME).await?;
    eventually_focused(d, &head(1), "Ctrl+Home").await
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

/// Tab lands on the stop, which keeps its place after Tab leaves and Shift+Tab comes back.
async fn one_stop<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    // The header's sort buttons sit after the first header cell: Tab in from before.
    d.focus("#before").await?;
    eventually_focused(d, "#before", "focus").await?;
    d.press(keyboard::TAB).await?;
    eventually_focused(d, &head(1), "Tab into the grid").await?;
    d.press(keyboard::ARROW_DOWN).await?;
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
    // A press would move the stop to the header cell: a script click sorts alone.
    d.script_click("th:nth-child(2) [data-sort-button]").await?;
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
    into_body(d, 1).await?;
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
    into_body(d, 1).await?;
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
    into_body(d, 1).await?;
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
    eventually_focused(d, &head(1), "Ctrl+Home on the header").await?;
    d.press(keyboard::ARROW_DOWN).await?;
    eventually(
        d,
        "ArrowDown on the first row, below the header",
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

/// Enter on the Age header cell sorts it up, Space down; focus stays on the cell.
async fn header_sorts<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(STOP).await?;
    eventually_focused(d, &head(1), "focus on the stop").await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually_focused(d, &head(2), "ArrowRight to Age").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, &cell(1, 1), "Cleo", "Enter to sort by age").await?;
    eventually_focused(d, &head(2), "focus on the sorted header cell").await?;
    d.press(keyboard::SPACE).await?;
    eventually_text(d, &cell(1, 1), "Dev", "Space to sort down").await
}

e2e::scenario!(
    enter_on_a_header_sorts,
    "/table-grid",
    header_sorts,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Shift+Space on a body cell toggles its row; Ctrl+Enter on a header cell opens its menu.
async fn selects<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const ANA: &str = "tbody tr:nth-child(1)";
    // The leading checkbox cell is no grid cell: the stop starts on Name.
    into_body(d, 2).await?;
    for selected in ["true", "false"] {
        d.press_shift(keyboard::SPACE).await?;
        eventually(
            d,
            &format!("Shift+Space to make Ana {selected}"),
            async |d| Ok(d.attr(ANA, "aria-selected").await?.as_deref() == Some(selected)),
        )
        .await?;
    }
    // 3259: not run on an iOS device.
    if d.platform() == Platform::Ios {
        return Ok(());
    }
    d.press(keyboard::ARROW_UP).await?;
    eventually_focused(d, &head(2), "ArrowUp to the Name header").await?;
    d.press_ctrl(keyboard::ENTER).await?;
    eventually(d, "Ctrl+Enter to open the Name menu", async |d| {
        Ok(
            d.attr(&format!("{} [aria-haspopup]", head(2)), "aria-expanded")
                .await?
                .as_deref()
                == Some("true"),
        )
    })
    .await
}

e2e::scenario!(
    shift_space_selects_the_row,
    "/table-grid/selectable",
    selects,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// The browser's collator sorts by the app's language, not the browser's: in Swedish
/// "Ä" comes after "Z".
async fn sorts_by_language<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    eventually(d, "the page language", async |d| {
        Ok(d.evaluate("document.documentElement.lang").await? == "sv")
    })
    .await?;
    d.click(&format!("{} button", head(1))).await?;
    for (row, word) in [(1, "Anna"), (2, "Zoe"), (3, "Äpple")] {
        eventually_text(d, &cell(row, 1), word, "the Swedish order").await?;
    }
    Ok(())
}

e2e::scenario!(
    a_sort_follows_the_apps_language,
    "/table-grid/collation",
    sorts_by_language,
    android: skip("3294: the WebView app has no Intl.Collator to call")
);

const EDITOR: &str = "[data-cell-editor] input";

/// Types `text` over the open editor's value, which a renderer with `select()` has
/// selected on open.
async fn retype<D: Driver>(d: &mut D, text: &str) -> Result<()> {
    eventually_focused(d, EDITOR, "focus in the editor").await?;
    if matches!(d.platform(), Platform::Android | Platform::Ios) {
        d.evaluate("document.activeElement.select()").await?;
    }
    d.type_text(text).await
}

/// A double click opens Ana's age; leaving the field commits, and the caller stores it.
async fn double_click_edits<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.double_click(&cell(1, 2)).await?;
    retype(d, "40").await?;
    d.click("#after").await?;
    eventually_text(d, &cell(1, 2), "40", "the stored edit").await?;
    eventually_text(d, "#log", "Ana 0 Age: 40 Lisbon", "onrowupdate").await?;
    if d.exists(EDITOR).await? {
        bail!("the editor stays open after the blur");
    }
    Ok(())
}

e2e::scenario!(
    a_double_click_edits_and_blur_commits,
    "/table-grid/editable",
    double_click_edits
);

/// Enter opens and commits and moves the stop down, F2 opens and Escape cancels there.
async fn keys_edit<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(&cell(2, 3)).await?;
    at(d, 2, 3, "a click on Ben's city").await?;
    d.press(keyboard::ENTER).await?;
    retype(d, "Bonn").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, &cell(2, 3), "Bonn", "Enter to commit").await?;
    at(d, 3, 3, "Enter moved the stop down").await?;
    d.press(keyboard::F2).await?;
    retype(d, "Paris").await?;
    d.press(keyboard::ESCAPE).await?;
    at(d, 3, 3, "Escape back to the cell").await?;
    eventually_text(d, &cell(3, 3), "Cairo", "Escape to cancel").await
}

e2e::scenario!(
    enter_commits_and_escape_cancels,
    "/table-grid/editable",
    keys_edit,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Shift+Enter commits and moves up, Tab and Shift+Tab move right and left; no wrap.
async fn flows<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(&cell(2, 2)).await?;
    at(d, 2, 2, "a click on Ben's age").await?;
    d.press(keyboard::ENTER).await?;
    retype(d, "43").await?;
    d.press_shift(keyboard::ENTER).await?;
    eventually_text(d, &cell(2, 2), "43", "Shift+Enter to commit").await?;
    at(d, 1, 2, "Shift+Enter moved the stop up").await?;
    d.press(keyboard::ENTER).await?;
    retype(d, "32").await?;
    d.press(keyboard::TAB).await?;
    eventually_text(d, &cell(1, 2), "32", "Tab to commit").await?;
    at(d, 1, 3, "Tab moved the stop right").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, EDITOR, "Enter on the city").await?;
    d.press(keyboard::TAB).await?;
    at(d, 1, 3, "Tab at the last column").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, EDITOR, "Enter on the city again").await?;
    d.press_shift(keyboard::TAB).await?;
    at(d, 1, 2, "Shift+Tab moved the stop left").await
}

e2e::scenario!(
    tab_and_shift_enter_move_the_stop,
    "/table-grid/editable",
    flows,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// A second tap on the stop cell opens its editor; the first only makes it the stop.
async fn second_tap<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    let target = cell(2, 2);
    d.touch_down(&target).await?;
    d.touch_up(&target).await?;
    at(d, 2, 2, "a first tap").await?;
    if d.exists(EDITOR).await? {
        bail!("the first tap opened the editor");
    }
    d.touch_down(&target).await?;
    d.touch_up(&target).await?;
    eventually_focused(d, EDITOR, "a second tap").await
}

e2e::scenario!(
    a_second_tap_opens_the_editor,
    "/table-grid/editable",
    second_tap,
    native: skip("3242: Blitz fires no pointer events for a touch"),
    desktop: skip("no touch input in the desktop driver")
);

/// The editor opens with its text selected, so typing replaces it.
async fn selected_on_open<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(&cell(1, 2)).await?;
    at(d, 1, 2, "a click on Ana's age").await?;
    d.press(keyboard::ENTER).await?;
    eventually_focused(d, EDITOR, "Enter to open").await?;
    d.type_text("9").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, &cell(1, 2), "9", "typing over the selection").await
}

e2e::scenario!(
    the_editor_opens_with_its_text_selected,
    "/table-grid/editable",
    selected_on_open,
    android: skip("958: roving keys are inert in the emulator's WebView"),
    ios: skip("3262: not run on an iOS device")
);

/// An age the column's `validate` refuses keeps the editor open, marked invalid, with
/// its message shown and announced; typing clears it. A word is no number at all.
async fn refuses<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const ERROR: &str = "[data-cell-error]";
    d.double_click(&cell(1, 2)).await?;
    retype(d, "200").await?;
    d.press(keyboard::ENTER).await?;
    eventually(d, "the editor to be invalid", async |d| {
        Ok(d.attr(EDITOR, "aria-invalid").await?.as_deref() == Some("true"))
    })
    .await?;
    eventually_text(d, ERROR, "Too old", "the refused commit").await?;
    let id = d.attr(ERROR, "id").await?;
    if id.is_none() || d.attr(EDITOR, "aria-describedby").await? != id {
        bail!("the editor is not described by its message");
    }
    eventually_text(d, STATUS, "Age is invalid: Too old", "the announcement").await?;
    if !d.text("#log").await?.is_empty() {
        bail!("an invalid edit reached onrowupdate");
    }
    retype(d, "old").await?;
    eventually(d, "typing to clear the message", async |d| {
        Ok(!d.exists(ERROR).await?)
    })
    .await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, ERROR, "Not a valid value", "a word for an age").await
}

e2e::scenario!(
    an_invalid_value_keeps_the_editor,
    "/table-grid/editable",
    refuses
);

/// Leaving the field with a refused value closes the editor, puts the old value back and
/// announces it, so no editor stays behind in a row the focus left.
async fn refused_blur<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.double_click(&cell(1, 2)).await?;
    retype(d, "200").await?;
    d.click("#after").await?;
    eventually_text(d, STATUS, "Age not changed: Too old", "the announcement").await?;
    eventually(d, "the editor to close", async |d| {
        Ok(!d.exists(EDITOR).await?)
    })
    .await?;
    eventually_text(d, &cell(1, 2), "31", "the old value").await?;
    if !d.text("#log").await?.is_empty() {
        bail!("a refused edit reached onrowupdate");
    }
    Ok(())
}

e2e::scenario!(
    a_refused_blur_puts_the_old_value_back,
    "/table-grid/editable",
    refused_blur
);

/// The open checkbox editor's `(checked, mixed)`: a browser reads mixed from the property alone.
async fn box_state<D: Driver>(d: &mut D) -> Result<(bool, bool)> {
    let checked = d
        .evaluate(&format!("document.querySelector('{EDITOR}').checked"))
        .await?
        == true;
    let mixed = match d.platform() {
        Platform::Web => {
            d.evaluate(&format!("document.querySelector('{EDITOR}').indeterminate"))
                .await?
                == true
        }
        _ => d.attr(EDITOR, "aria-checked").await?.as_deref() == Some("mixed"),
    };
    Ok((checked, mixed))
}

async fn eventually_box<D: Driver>(d: &mut D, want: (bool, bool), what: &str) -> Result<()> {
    eventually(d, what, async |d| Ok(box_state(d).await? == want)).await
}

/// An `Option<bool>` editor is a checkbox that Space cycles empty, true, false, empty and
/// Enter commits; a date editor names its form as a placeholder.
async fn formats<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.double_click(&cell(1, 2)).await?;
    eventually_focused(d, EDITOR, "focus in the yes/no editor").await?;
    if d.attr(EDITOR, "type").await?.as_deref() != Some("checkbox") {
        bail!("the yes/no editor is not a checkbox");
    }
    eventually_box(d, (false, true), "an empty value is mixed").await?;
    let size = d.rect(EDITOR).await?;
    if size.width < 24.0 || size.height < 24.0 {
        bail!(
            "the checkbox editor is {}x{} px, under 24x24",
            size.width,
            size.height
        );
    }
    d.press(keyboard::SPACE).await?;
    eventually_box(d, (true, false), "Space checks it").await?;
    d.press(keyboard::SPACE).await?;
    eventually_box(d, (false, false), "Space unchecks it").await?;
    d.press(keyboard::SPACE).await?;
    eventually_box(d, (false, true), "Space empties it").await?;
    d.press(keyboard::SPACE).await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, &cell(1, 2), "true", "Enter commits the check").await?;
    d.double_click(&cell(1, 3)).await?;
    eventually_focused(d, EDITOR, "focus in the date editor").await?;
    if d.attr(EDITOR, "placeholder").await?.as_deref() != Some("YYYY-MM-DD") {
        bail!("the date editor has no placeholder");
    }
    d.type_text("2024-03-09").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, &cell(1, 3), "2024-03-09", "a typed date").await
}

e2e::scenario!(
    a_yes_no_editor_is_a_checkbox_and_a_date_editor_names_its_form,
    "/table-grid/editable/formats",
    formats,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// A custom editor is described by the refused value's message, and clears it by calling `changed`.
async fn custom_clears<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const CUSTOM: &str = "#custom-editor";
    d.double_click(&cell(1, 2)).await?;
    eventually_focused(d, CUSTOM, "focus in the custom editor").await?;
    d.type_text("200").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "[data-cell-error]", "Too old", "the refused commit").await?;
    if d.attr(CUSTOM, "aria-invalid").await?.as_deref() != Some("true") {
        bail!("the custom editor got no error");
    }
    let id = d.attr("[data-cell-error]", "id").await?;
    if id.is_none() || d.attr(CUSTOM, "aria-describedby").await? != id {
        bail!("the custom editor is not described by its message");
    }
    d.type_text("1").await?;
    eventually(d, "typing to clear the message", async |d| {
        Ok(!d.exists("[data-cell-error]").await?)
    })
    .await
}

e2e::scenario!(
    a_custom_editor_clears_the_error_by_changed,
    "/table-grid/editable/custom",
    custom_clears,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// In a fixed-row-height table the refused value's message hangs over the next row.
async fn error_overlay<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const ERROR: &str = "[data-cell-error]";
    d.double_click(&cell(1, 2)).await?;
    retype(d, "200").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, ERROR, "Too old", "the refused commit").await?;
    let shown = d
        .evaluate(&format!(
            "(e => {{ const r = e.getBoundingClientRect(); \
             const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height - 2); \
             return hit?.closest('[data-cell-error]') === e; }})(document.querySelector('{ERROR}'))"
        ))
        .await?;
    if shown != serde_json::Value::Bool(true) {
        bail!("the message is clipped by its row");
    }
    let row = d.rect("tbody tr:nth-child(1)").await?.height;
    if (row - 28.0).abs() > 0.5 {
        bail!("the message stretched its row to {row}px, the window's pitch is 28px");
    }
    Ok(())
}

e2e::scenario!(
    an_error_shows_in_a_fixed_row_height_table,
    "/table-grid/editable/windowed",
    error_overlay,
    android: skip("958: roving keys are inert in the emulator's WebView"),
    native: skip("no script reads")
);

/// A handler that stores nothing leaves the caller's value in the cell.
async fn kept<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.double_click(&cell(1, 2)).await?;
    retype(d, "40").await?;
    d.click("#after").await?;
    eventually_text(d, "#log", "Ana 0 Age: 40 Lisbon", "onrowupdate").await?;
    eventually_text(d, &cell(1, 2), "31", "the caller's age").await
}

e2e::scenario!(
    the_data_stays_the_callers,
    "/table-grid/editable/kept",
    kept
);

/// The cell keys are for the cell itself: Enter on a header's sort button sorts once.
async fn sort_button_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const SORT: &str = "th:nth-child(2) [data-sort-button]";
    d.focus(SORT).await?;
    eventually_focused(d, SORT, "focus on the sort button").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, &cell(1, 1), "Cleo", "Enter to sort by age").await?;
    d.settle().await?;
    eventually_text(d, &cell(1, 1), "Cleo", "the sort, once").await
}

e2e::scenario!(
    enter_on_a_sort_button_sorts_once,
    "/table-grid",
    sort_button_once,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Space on the select-all box selects every row once, and sorts nothing.
async fn select_box_once<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const BOX: &str = "thead input[type=checkbox]";
    d.focus(BOX).await?;
    eventually_focused(d, BOX, "focus on the select-all box").await?;
    d.press(keyboard::SPACE).await?;
    eventually(d, "Space to select every row", async |d| {
        Ok(d.attr("tbody tr:nth-child(1)", "aria-selected")
            .await?
            .as_deref()
            == Some("true"))
    })
    .await?;
    d.settle().await?;
    eventually_text(d, &cell(1, 2), "Ana", "no sort").await
}

e2e::scenario!(
    space_on_the_select_all_box_acts_once,
    "/table-grid/selectable",
    select_box_once,
    android: skip("958: roving keys are inert in the emulator's WebView")
);

/// Enter on a caller's button in an editable cell presses it and opens no editor.
async fn button_in_a_cell<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    const PING: &str = "tbody tr:nth-child(1) button";
    d.focus(PING).await?;
    eventually_focused(d, PING, "focus on the button").await?;
    d.press(keyboard::ENTER).await?;
    eventually_text(d, "#log", "Pinged Ana", "Enter on the button").await?;
    d.settle().await?;
    if d.exists(EDITOR).await? {
        bail!("Enter on the button opened the editor");
    }
    Ok(())
}

e2e::scenario!(
    enter_on_a_button_in_an_editable_cell_opens_no_editor,
    "/table-grid/editable/button",
    button_in_a_cell,
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
