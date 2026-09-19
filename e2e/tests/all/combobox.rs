//! Bare `Combobox`, its rows drawn by the caller with `ComboboxOption`, and
//! the probes the family's shared web/native scenarios read.

use anyhow::{Result, ensure};
use e2e::driver::{Driver, eventually, eventually_focused};
use e2e::passes::keyboard;

pub const TRIGGER: &str = "[role=combobox]";
pub const LISTBOX: &str = "[role=listbox]";

/// Todo 642: the selected-row bar, until now pinned only through `Select` and
/// `MultiSelect`.
#[test]
fn the_selected_row_shows_the_on_state_line() {
    crate::select::selected_row_is_marked("/combobox");
}

pub async fn is_expanded<D: Driver>(d: &mut D) -> Result<bool> {
    Ok(d.attr(TRIGGER, "aria-expanded").await?.as_deref() == Some("true"))
}

pub async fn until_expanded<D: Driver>(d: &mut D, open: bool, after: &str) -> Result<()> {
    eventually(d, &format!("{after}: expanded={open}"), async |d| {
        Ok(is_expanded(d).await? == open)
    })
    .await
}

/// The fixture's `#echo` reads `expected`.
pub async fn echoes<D: Driver>(d: &mut D, expected: &str, after: &str) -> Result<()> {
    eventually(
        d,
        &format!("{after}: #echo to read {expected:?}"),
        async |d| Ok(d.text("#echo").await? == expected),
    )
    .await
}

async fn listbox_shows<D: Driver>(d: &mut D, after: &str) -> Result<()> {
    eventually(d, &format!("{after}: the listbox to show"), async |d| {
        d.exists(LISTBOX).await
    })
    .await
}

// `Autocomplete`, on `/autocomplete/echo`.

async fn autocomplete_typing_and_enter<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    d.type_text("be").await?;
    until_expanded(d, true, "typing \"be\"").await?;
    eventually(d, "\"be\" to leave one option", async |d| {
        Ok(d.exists("[role=option]").await? && !d.exists("[role=option]:nth-child(2)").await?)
    })
    .await?;
    d.press(keyboard::ARROW_DOWN).await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, "Berlin", "Enter on the option").await?;
    until_expanded(d, false, "the pick").await
}

async fn autocomplete_escape<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.press(keyboard::ESCAPE).await?;
    until_expanded(d, false, "Escape").await?;
    eventually_focused(d, TRIGGER, "Escape").await
}

async fn autocomplete_click_picks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.click("[role=option]:nth-child(3)").await?;
    echoes(d, "Copenhagen", "a click on the third option").await
}

e2e::scenario!(
    typing_filters_an_autocomplete_and_enter_picks_the_option,
    "/autocomplete/echo",
    autocomplete_typing_and_enter
);
e2e::scenario!(
    escape_closes_an_open_autocomplete,
    "/autocomplete/echo",
    autocomplete_escape
);
e2e::scenario!(
    a_click_on_an_autocomplete_option_picks_it,
    "/autocomplete/echo",
    autocomplete_click_picks
);

// `MultiSelect`, on `/multi-select/echo`: Cherry held.

async fn multi_keys_add<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    // Opens on the held Cherry: Enter drops it, the list stays open.
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, "[]", "Enter on the held Cherry").await?;
    ensure!(is_expanded(d).await?, "Enter closed the multi-select");
    d.press(keyboard::ARROW_UP).await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, "[Banana]", "ArrowUp, Enter").await
}

async fn multi_click_toggles<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    listbox_shows(d, "a click on the trigger").await?;
    d.click("[role=option]:nth-child(2)").await?;
    echoes(d, "[Cherry, Banana]", "a click on Banana").await
}

/// The rows cancel `mousedown` to keep focus on the trigger, which closes on
/// blur. Blitz moves focus on the press regardless; libero moves it back and
/// the trigger ignores that blur (todo 472).
async fn multi_click_keeps_open<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    listbox_shows(d, "a click on the trigger").await?;
    d.click("[role=option]:nth-child(2)").await?;
    echoes(d, "[Cherry, Banana]", "a click on Banana").await?;
    ensure!(
        is_expanded(d).await?,
        "a click on an option closed the list"
    );
    eventually_focused(d, TRIGGER, "a click on an option").await
}

async fn multi_backspace_drops<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::BACKSPACE).await?;
    echoes(d, "[]", "Backspace").await
}

e2e::scenario!(
    the_keyboard_adds_a_multi_select_option,
    "/multi-select/echo",
    multi_keys_add
);
e2e::scenario!(
    a_click_on_a_multi_select_option_toggles_it,
    "/multi-select/echo",
    multi_click_toggles
);
e2e::scenario!(
    a_click_on_a_multi_select_option_keeps_the_list_open,
    "/multi-select/echo",
    multi_click_keeps_open
);
e2e::scenario!(
    backspace_in_an_empty_multi_select_drops_the_last_pick,
    "/multi-select/echo",
    multi_backspace_drops
);

// `TagsField`, on `/tags-field/echo`: "rust" held.

async fn tags_typing_adds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    d.type_text("css").await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, r#"["rust", "css"]"#, "typing css, Enter").await
}

async fn tags_suggestion_adds<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    listbox_shows(d, "ArrowDown").await?;
    d.press(keyboard::ENTER).await?;
    echoes(d, r#"["rust", "dioxus"]"#, "Enter on the first suggestion").await
}

async fn tags_backspace_removes<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    d.press(keyboard::BACKSPACE).await?;
    d.press(keyboard::BACKSPACE).await?;
    echoes(d, "[]", "Backspace twice").await
}

e2e::scenario!(
    typing_and_enter_add_a_tag,
    "/tags-field/echo",
    tags_typing_adds
);
e2e::scenario!(
    a_suggestion_picked_by_the_keys_becomes_a_tag,
    "/tags-field/echo",
    tags_suggestion_adds
);
e2e::scenario!(
    backspace_in_an_empty_tags_field_removes_the_last_tag,
    "/tags-field/echo",
    tags_backspace_removes
);

// `Cascader`, on `/cascader`: Europe, Asia (disabled), Oceania; `#picked`.

async fn picked<D: Driver>(d: &mut D, expected: &str, after: &str) -> Result<()> {
    eventually(
        d,
        &format!("{after}: #picked to read {expected}"),
        async |d| Ok(d.text("#picked").await? == expected),
    )
    .await
}

async fn cascader_keys<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus(TRIGGER).await?;
    d.press(keyboard::ARROW_DOWN).await?;
    until_expanded(d, true, "ArrowDown").await?;
    // Europe -> France -> Paris.
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    d.press(keyboard::ENTER).await?;
    picked(d, "paris", "Right, Right, Enter").await
}

/// A click on a branch opens its column. Natively the press blurs the
/// trigger all the same, which must not close the list (todo 472).
async fn cascader_clicks<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.click(TRIGGER).await?;
    until_expanded(d, true, "a click on the trigger").await?;
    d.click("[id$=-option-0-2]").await?;
    eventually(d, "Oceania's column", async |d| {
        d.exists("[id$=-option-1-0]").await
    })
    .await?;
    ensure!(
        is_expanded(d).await?,
        "the click on Oceania closed the list"
    );
    d.click("[id$=-option-1-0]").await?;
    eventually(d, "Australia's column", async |d| {
        d.exists("[id$=-option-2-0]").await
    })
    .await?;
    d.click("[id$=-option-2-0]").await?;
    picked(d, "sydney", "clicks on Oceania, Australia, Sydney").await
}

e2e::scenario!(
    the_keys_walk_a_cascader_to_a_leaf,
    "/cascader",
    cascader_keys
);
e2e::scenario!(
    clicks_walk_a_cascader_to_a_leaf,
    "/cascader",
    cascader_clicks
);
