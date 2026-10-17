use std::{
    cell::Cell,
    rc::{Rc, Weak},
};

use dioxus::prelude::*;

use super::{Doc, Origin, doc, typing_target};
use crate::platform::{KeyChord, KeySubscription, KeyboardApi, warn_reserved_chord};

thread_local! {
    pub(super) static NEXT_CALLBACK: Cell<u64> = const { Cell::new(0) };
}

pub(super) type KeyCallback = (u64, bool, Origin, Rc<dyn Fn(KeyChord) -> bool>);

/// Hands a press that bubbled out of the app to every key subscription.
pub(super) fn keyed(event: &Event<KeyboardData>) {
    // Bubble phase runs last: a press a handler below took is not the document's.
    if !event.default_action_enabled() || event.is_composing() {
        return;
    }
    let Some(doc) = doc() else {
        return;
    };
    let typing = typing_target();
    let callbacks: Vec<_> = doc
        .key_callbacks
        .borrow()
        .iter()
        .filter(|(_, skip_text_entry, ..)| !(typing && *skip_text_entry))
        .map(|(_, skip_text_entry, origin, callback)| (*skip_text_entry, *origin, callback.clone()))
        .collect();
    let (key, mut modifiers) = (event.key(), event.modifiers());
    // blitz-shell reports Cmd as SUPER; a chord reads it as META, as on the web.
    if modifiers.contains(Modifiers::SUPER) {
        modifiers.insert(Modifiers::META);
    }
    let mut taken = false;
    for (skip_text_entry, origin, callback) in callbacks {
        let chord = KeyChord {
            key: key.clone(),
            modifiers,
            repeat: event.is_auto_repeating(),
            code: event.code(),
            text_entry: typing,
            scopes: Vec::new(),
        };
        let handled = origin.run(|| callback(chord));
        if handled && skip_text_entry {
            warn_reserved_chord(&key, modifiers);
        }
        taken |= handled;
    }
    if taken {
        event.prevent_default();
    }
}

pub(in crate::platform::backend) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    Some(&KEYBOARD)
}

/// Key presses bubbling out of the app to [`Listener`](super::Listener): a stopped press, or one
/// with focus outside the wrapper, never arrives.
pub(super) struct BlitzKeyboard;

static KEYBOARD: BlitzKeyboard = BlitzKeyboard;

impl BlitzKeyboard {
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        let id = NEXT_CALLBACK.replace(NEXT_CALLBACK.get() + 1);
        let doc = doc();
        if let Some(doc) = &doc {
            doc.key_callbacks.borrow_mut().push((
                id,
                skip_text_entry,
                Origin::here(),
                Rc::from(callback),
            ));
        }
        Box::new(BlitzKeySubscription(id, doc.as_ref().map(Rc::downgrade)))
    }
}

impl KeyboardApi for BlitzKeyboard {
    fn on_key(&self, callback: Box<dyn Fn(KeyChord) -> bool>) -> Box<dyn KeySubscription> {
        self.listen(true, callback)
    }

    fn on_key_unfiltered(
        &self,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        self.listen(false, callback)
    }
}

struct BlitzKeySubscription(u64, Option<Weak<Doc>>);

impl KeySubscription for BlitzKeySubscription {}

impl Drop for BlitzKeySubscription {
    fn drop(&mut self) {
        if let Some(doc) = self.1.as_ref().and_then(Weak::upgrade) {
            doc.key_callbacks
                .borrow_mut()
                .retain(|(id, ..)| *id != self.0);
        }
    }
}
