use std::sync::atomic::Ordering;

use dioxus::core::provide_root_context;
use dioxus::prelude::*;

use super::{
    data::NotificationId,
    store::{NEXT_ID, NotificationStore},
};
use crate::hooks::{Hotkey, use_hotkeys};

/// Every mounted host's hotkey, by host, so one press moves focus once: into the
/// newest notification across all hosts (todo 670). Root context, not a
/// `thread_local!`: Android runs key handlers on another thread than effects (2216).
#[derive(Clone, Copy)]
struct Hotkeys(CopyValue<Vec<(u64, NotificationStore, Key)>>);

fn hotkeys() -> Hotkeys {
    try_consume_context::<Hotkeys>().unwrap_or_else(|| {
        provide_root_context(Hotkeys(CopyValue::new_in_scope(Vec::new(), ScopeId::ROOT)))
    })
}

/// `hotkey` focuses the newest notification (todo 575). A letter is not taken
/// from a text field; any other key is heard from anywhere.
pub(super) fn use_hotkey(store: NotificationStore, hotkey: Key) {
    let host = use_hook(|| NEXT_ID.fetch_add(1, Ordering::Relaxed));
    let Hotkeys(mut hosts) = use_hook(hotkeys);
    use_drop(move || {
        // The root's registry is gone already when the whole dom drops.
        if let Ok(mut hosts) = hosts.try_write() {
            hosts.retain(|(other, ..)| *other != host);
        }
    });
    use_effect(use_reactive!(|hotkey| {
        let mut hosts = hosts.write();
        hosts.retain(|(other, ..)| *other != host);
        hosts.push((host, store, hotkey.clone()));
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
        .when(move || hotkey_owner(hosts, &key) == Some(host))]);
}

fn is_hotkey(pressed: &Key, key: &Key) -> bool {
    match (pressed, key) {
        (Key::Character(pressed), Key::Character(key)) => pressed.eq_ignore_ascii_case(key),
        (pressed, key) => pressed == key,
    }
}

/// The host that answers `pressed`: the one drawing the newest notification
/// among those listening for it. The first registered wins a shared store.
fn hotkey_owner(
    hosts: CopyValue<Vec<(u64, NotificationStore, Key)>>,
    pressed: &Key,
) -> Option<u64> {
    let hosts = hosts
        .try_read()
        .map(|hosts| hosts.clone())
        .unwrap_or_default();
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
