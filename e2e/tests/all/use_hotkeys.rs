//! `use_hotkeys`: `mod+k` runs once per press and is skipped in text entry, an
//! editable-including `ctrl+j` is not, and an unmounted listener stops. A
//! `within` hotkey fires only with focus in its elements.

use anyhow::Result;
use e2e::driver::{Driver, eventually, eventually_focused, linger};
use e2e::passes::keyboard::{self, Key};

/// A chord's key carries no text: a held Ctrl types nothing.
const K: Key = Key {
    key: "k",
    code: "KeyK",
    vk: 75,
    text: None,
};
const J: Key = Key {
    key: "j",
    code: "KeyJ",
    vk: 74,
    text: None,
};

async fn count_is<D: Driver>(d: &mut D, selector: &str, expected: &str) -> Result<()> {
    eventually(d, &format!("{selector} to read {expected:?}"), async |d| {
        Ok(d.text(selector).await? == expected)
    })
    .await
}

async fn mod_k_runs_once_per_press<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#plain").await?;
    d.press_ctrl(K).await?;
    count_is(d, "#open", "1").await?;
    d.press_ctrl(K).await?;
    count_is(d, "#open", "2").await
}

async fn text_entry_keeps_the_chord_unless_included<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.focus("#field").await?;
    d.press_ctrl(K).await?;
    d.idle().await;
    count_is(d, "#open", "0").await?;
    d.press_ctrl(J).await?;
    count_is(d, "#typing", "1").await
}

async fn an_unmounted_listener_stops<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#plain").await?;
    d.press_ctrl(K).await?;
    count_is(d, "#open", "1").await?;
    d.click("#remove").await?;
    d.idle().await;
    d.press_ctrl(K).await?;
    d.idle().await;
    count_is(d, "#open", "1").await
}

async fn unmounting_one_instance_leaves_the_other<D: Driver>(
    d: &mut D,
    _route: &str,
) -> Result<()> {
    d.focus("#plain").await?;
    d.press_ctrl(K).await?;
    count_is(d, "#open", "1").await?;
    count_is(d, "#other", "1").await?;
    d.click("#remove").await?;
    d.idle().await;
    d.press_ctrl(K).await?;
    count_is(d, "#other", "2").await?;
    count_is(d, "#open", "1").await
}

const X: Key = Key {
    key: "x",
    code: "KeyX",
    vk: 88,
    text: Some("x"),
};

async fn within_fires_only_with_focus_in_scope<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#outside").await?;
    d.press(X).await?;
    d.idle().await;
    count_is(d, "#count", "0").await?;
    for (inside, count) in [("#inside", "1"), ("#inside-field", "2"), ("#in-popup", "3")] {
        d.focus(inside).await?;
        d.press(X).await?;
        count_is(d, "#count", count).await?;
    }
    Ok(())
}

/// Taken inside first, so a WebView's "taken" chord must not follow it out.
async fn a_press_outside_keeps_its_default<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    d.focus("#inside-field").await?;
    d.press(X).await?;
    count_is(d, "#count", "1").await?;
    d.focus("#outside-field").await?;
    for typed in ["x", "xx"] {
        d.press(X).await?;
        count_is(d, "#typed", typed).await?;
    }
    count_is(d, "#count", "1").await
}

const ITEM: &str = "[role=menuitem]";
const SHARE: &str = "[role=menuitem][aria-haspopup=menu]";
const SUBMENU_ITEM: &str = "[role=menu]:not(:has([aria-haspopup=menu])) [role=menuitem]";

/// Focuses `selector` again until it holds focus: a popover's box renders hidden
/// until placed, and `focus()` in it moves nothing (1276, a cold WebView).
async fn focus_shown<D: Driver>(d: &mut D, selector: &str) -> Result<()> {
    let settled = eventually(d, &format!("{selector} to take focus"), async |d| {
        d.focus(selector).await?;
        d.is_focused(selector).await
    })
    .await;
    match settled {
        Ok(()) => Ok(()),
        Err(_) => eventually_focused(d, selector, "focusing it").await,
    }
}

/// Opens the menu behind `trigger` and focuses its first item.
async fn open_menu<D: Driver>(d: &mut D, trigger: &str) -> Result<()> {
    d.click(trigger).await?;
    eventually(d, "the menu to open", async |d| d.exists(ITEM).await).await?;
    focus_shown(d, ITEM).await
}

async fn close_menus<D: Driver>(d: &mut D) -> Result<()> {
    while d.exists(ITEM).await? {
        d.press(keyboard::ESCAPE).await?;
        d.idle().await;
    }
    Ok(())
}

/// A `Menu` opened inside the scope, its submenu too, counts as inside; one opened outside not.
async fn a_menu_opened_inside_counts_as_inside<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    open_menu(d, "#outside button").await?;
    d.press_ctrl(J).await?;
    linger(d, 5).await;
    count_is(d, "#count", "0").await?;
    close_menus(d).await?;

    open_menu(d, "#scope button").await?;
    d.press_ctrl(J).await?;
    count_is(d, "#count", "1").await?;

    d.focus(SHARE).await?;
    d.press(keyboard::ARROW_RIGHT).await?;
    eventually(d, "the submenu to open", async |d| {
        d.exists(SUBMENU_ITEM).await
    })
    .await?;
    // A WebView's submenu opens without taking focus (958): the scope is under test, not Menu.
    focus_shown(d, SUBMENU_ITEM).await?;
    eventually(d, "focus in the submenu", async |d| {
        Ok(d.is_focused(ITEM).await? && !d.is_focused(SHARE).await?)
    })
    .await?;
    d.press_ctrl(J).await?;
    count_is(d, "#count", "2").await
}

/// A press from `outside` is not counted, one from `inside` is.
async fn only_the_inside_box_counts<D: Driver>(
    d: &mut D,
    outside: &str,
    inside: &str,
) -> Result<()> {
    focus_shown(d, outside).await?;
    d.press_ctrl(J).await?;
    linger(d, 5).await;
    count_is(d, "#count", "0").await?;
    focus_shown(d, inside).await?;
    d.press_ctrl(J).await?;
    count_is(d, "#count", "1").await
}

/// A `HoverCard` anchored inside the scope counts as inside, one outside not (1225).
async fn a_hover_card_inside_counts_as_inside<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    only_the_inside_box_counts(d, "#out-card", "#in-card").await
}

/// So does an app's own `use_popover` box (1225).
async fn a_popover_inside_counts_as_inside<D: Driver>(d: &mut D, _route: &str) -> Result<()> {
    only_the_inside_box_counts(d, "#outside-box", "#inside-box").await
}

e2e::scenario!(
    a_hover_card_opened_inside_the_scope_counts_as_inside,
    "/use-hotkeys/within-hover-card",
    a_hover_card_inside_counts_as_inside
);
e2e::scenario!(
    an_own_popover_opened_inside_the_scope_counts_as_inside,
    "/use-hotkeys/within-popover",
    a_popover_inside_counts_as_inside
);
e2e::scenario!(
    a_menu_opened_inside_the_scope_counts_as_inside,
    "/use-hotkeys/within-menu",
    a_menu_opened_inside_counts_as_inside
);
e2e::scenario!(
    a_scoped_hotkey_fires_only_with_focus_inside_its_elements,
    "/use-hotkeys/within",
    within_fires_only_with_focus_in_scope
);
e2e::scenario!(
    a_press_outside_the_scope_keeps_its_default_action,
    "/use-hotkeys/within",
    a_press_outside_keeps_its_default
);
e2e::scenario!(
    unmounting_one_hotkey_instance_keeps_the_other_listening,
    "/use-hotkeys",
    unmounting_one_instance_leaves_the_other
);
e2e::scenario!(
    mod_k_counts_each_press_once,
    "/use-hotkeys",
    mod_k_runs_once_per_press
);
e2e::scenario!(
    a_text_field_keeps_the_chord_unless_the_hotkey_includes_it,
    "/use-hotkeys",
    text_entry_keeps_the_chord_unless_included
);
e2e::scenario!(
    unmounting_the_component_stops_the_hotkey,
    "/use-hotkeys",
    an_unmounted_listener_stops
);
