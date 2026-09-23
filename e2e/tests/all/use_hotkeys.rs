//! `use_hotkeys`: `mod+k` runs once per press and is skipped in text entry, an
//! editable-including `ctrl+j` is not, and an unmounted listener stops.

use anyhow::Result;
use e2e::driver::{Driver, eventually};
use e2e::passes::keyboard::Key;

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
