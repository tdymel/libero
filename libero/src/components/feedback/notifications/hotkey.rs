use std::{cell::RefCell, sync::atomic::Ordering};

use dioxus::prelude::*;

use super::{
    data::NotificationId,
    store::{NEXT_ID, NotificationStore},
};
use crate::hooks::{Hotkey, use_hotkeys};

thread_local! {
    /// Every mounted host's hotkey, by host, so one press moves focus once:
    /// into the newest notification across all hosts (todo 670).
    static HOTKEYS: RefCell<Vec<(u64, NotificationStore, Key)>> = const { RefCell::new(Vec::new()) };
}

/// `hotkey` focuses the newest notification (todo 575). A letter is not taken
/// from a text field; any other key is heard from anywhere.
pub(super) fn use_hotkey(store: NotificationStore, hotkey: Key) {
    let host = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    use_drop(move || HOTKEYS.with_borrow_mut(|hosts| hosts.retain(|(other, ..)| *other != host)));
    use_effect(use_reactive!(|hotkey| {
        HOTKEYS.with_borrow_mut(|hosts| {
            hosts.retain(|(other, ..)| *other != host);
            hosts.push((host, store, hotkey.clone()));
        });
    }));

    let chord = match &hotkey {
        Key::Character(text) if text == " " => "space".to_owned(),
        Key::Character(text) => text.clone(),
        key => key.to_string(),
    };
    let key = hotkey.clone();
    use_hotkeys([Hotkey::new(chord, move || store.focus_newest())
        .include_editable(!matches!(hotkey, Key::Character(_)))
        // Nothing on screen, or another host holds a newer one: not ours.
        .when(move || hotkey_owner(&key) == Some(host))]);
}

fn is_hotkey(pressed: &Key, key: &Key) -> bool {
    match (pressed, key) {
        (Key::Character(pressed), Key::Character(key)) => pressed.eq_ignore_ascii_case(key),
        (pressed, key) => pressed == key,
    }
}

/// The host that answers `pressed`: the one drawing the newest notification
/// among those listening for it. The first registered wins a shared store.
fn hotkey_owner(pressed: &Key) -> Option<u64> {
    let hosts = HOTKEYS.with_borrow(|hosts| hosts.clone());
    hosts
        .into_iter()
        .filter(|(_, store, key)| is_hotkey(pressed, key) && store.alive())
        .filter_map(|(host, store, _)| Some((store.newest()?, host)))
        .fold(
            None,
            |owner: Option<(NotificationId, u64)>, (id, host)| match owner {
                Some((newest, _)) if newest.0 >= id.0 => owner,
                _ => Some((id, host)),
            },
        )
        .map(|(_, host)| host)
}
