use std::rc::Rc;

use dioxus::core::Runtime;
use dioxus::prelude::{Code, Key, Modifiers, NamedKey, spawn};
use serde_json::json;

use super::{Slot, WebViewListener, eval_with, page, runs_scripts};
use crate::platform::keyboard::{CLICKED_INPUT_TYPES, warn_reserved_chord};
use crate::platform::{KeyChord, KeySubscription, KeyboardApi, OBSERVE_ATTR, OWNER_ATTR};

pub(crate) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    runs_scripts().then_some(&KEYBOARD as &'static dyn KeyboardApi)
}

/// Key presses at the window, bubbling as on Blitz: a press a handler took or
/// stopped never arrives.
struct WebViewKeyboard;

static KEYBOARD: WebViewKeyboard = WebViewKeyboard;

/// The answer crosses the IPC after the press is over, so the script prevents
/// a chord once its subscription has taken it: from the second press on.
const ON_KEY: &str = "const [skipTyping, clicked, seeded, observe, owner] = data;
    const taken = new Set(seeded);
    // The trigger of the nearest popup around `el`, outside it: named by its
    // `aria-controls`, or the anchor whose tag the popup's owner attribute holds.
    const trigger = (el) => {
        for (let at = el; at; at = at.parentElement) {
            const tag = at.getAttribute(owner);
            if (tag !== null) {
                const by = document.querySelector(`[${observe}=\"${CSS.escape(tag)}\"]`);
                if (by && !at.contains(by)) return by;
            }
            if (!at.id) continue;
            const id = CSS.escape(at.id);
            const by = document.querySelector(`[aria-haspopup]:not([aria-haspopup=false])[aria-controls~=${id}], [role=combobox][aria-controls~=${id}]`);
            if (by && !at.contains(by)) return by;
        }
        return null;
    };
    // The tagged elements around the target, nearest first, for `Hotkey::within`;
    // a popup's walk goes on from its trigger, as its portal sits elsewhere (1212).
    const scopes = (target) => {
        const found = [];
        let from = target instanceof Element ? target : null;
        for (let hops = 0; from && hops < 8; hops++, from = trigger(from)) {
            let at = from.closest(`[${observe}]`);
            for (; at; at = at.parentElement?.closest(`[${observe}]`)) {
                found.push(Number(at.getAttribute(observe)));
            }
        }
        return found;
    };
    const typing = (target) => {
        if (!(target instanceof Element)) return false;
        if (target.tagName === 'TEXTAREA' || target.tagName === 'SELECT') return true;
        if (target.tagName === 'INPUT') {
            return !clicked.includes((target.getAttribute('type') ?? '').trim().toLowerCase());
        }
        return target.isContentEditable === true;
    };
    const onKey = (event) => {
        // A sibling subscription's prevent still lets this one hear the press, as on the web (1223).
        if ((event.defaultPrevented && !event.liberoTaken) || event.isComposing) return;
        const entry = typing(event.target);
        if (skipTyping && entry) return;
        // WebKitGTK names Shift+Tab `Unidentified`; its `code` stays `Tab` (2868).
        const key = event.key === 'Unidentified' && event.code === 'Tab' ? 'Tab' : event.key;
        const chord = [key, event.ctrlKey, event.shiftKey, event.altKey, event.metaKey];
        const inside = scopes(event.target);
        // Per target kind and scope: a chord taken outside text entry stays the field's inside it.
        const name = chord.join(' ') + (entry ? ' typing' : '') + (inside.length ? ' in ' + inside.join(',') : '');
        if (taken.has(name)) {
            event.preventDefault();
            event.liberoTaken = true;
        }
        dioxus.send([name, ...chord, event.repeat, entry, inside, event.code]);
    };
    window.addEventListener('keydown', onKey);";

/// The open layer's Escape, prevented from the first press: a subscription's
/// answer comes too late for it (`ON_KEY`). Composition and repeats stay unclaimed.
const CLAIM_ESCAPE: &str = "if (data) {
        window.__liberoEscape ??= (event) => {
            if (event.key !== 'Escape' || event.repeat || event.isComposing) return;
            event.preventDefault();
            event.liberoTaken = true;
        };
        addEventListener('keydown', window.__liberoEscape);
    } else if (window.__liberoEscape) {
        removeEventListener('keydown', window.__liberoEscape);
        delete window.__liberoEscape;
    }";

pub(crate) fn claim_escape(open: bool) {
    if runs_scripts() {
        eval_with(json!(open), CLAIM_ESCAPE);
    }
}

impl WebViewKeyboard {
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        // Per scope: a hotkey re-subscribes on every open and close, and would
        // otherwise start over without its chord each time.
        let scope = Runtime::try_current().and_then(|runtime| runtime.try_current_scope_id());
        let page = page();
        let mut taken = (page.as_ref().zip(scope))
            .and_then(|(page, scope)| page.taken_chords.borrow().get(&scope).cloned())
            .unwrap_or_default();
        let slot = Rc::new(Slot::new());
        let script = eval_with(
            json!([
                skip_text_entry,
                CLICKED_INPUT_TYPES,
                &taken,
                OBSERVE_ATTR,
                OWNER_ATTR
            ]),
            &format!(
                "{ON_KEY}
                {}
                window.removeEventListener('keydown', onKey);",
                slot.park("take: (name) => taken.add(name)")
            ),
        );
        let task_slot = slot.clone();
        let task = spawn(async move {
            let mut script = script;
            while let Ok((name, key, ctrl, shift, alt, meta, repeat, text_entry, scopes, code)) =
                script
                    .recv::<(
                        String,
                        String,
                        bool,
                        bool,
                        bool,
                        bool,
                        bool,
                        bool,
                        Vec<u64>,
                        String,
                    )>()
                    .await
            {
                let key = key.parse().unwrap_or(Key::Named(NamedKey::Unidentified));
                let mut modifiers = Modifiers::empty();
                modifiers.set(Modifiers::CONTROL, ctrl);
                modifiers.set(Modifiers::SHIFT, shift);
                modifiers.set(Modifiers::ALT, alt);
                modifiers.set(Modifiers::META, meta);
                let chord_key = key.clone();
                if !callback(KeyChord {
                    key,
                    modifiers,
                    repeat,
                    code: code.parse().unwrap_or(Code::Unidentified),
                    text_entry,
                    scopes,
                }) {
                    continue;
                }
                if skip_text_entry {
                    warn_reserved_chord(&chord_key, modifiers);
                }
                if taken.insert(name.clone()) {
                    if let Some((page, scope)) = page.as_ref().zip(scope) {
                        (page.taken_chords.borrow_mut())
                            .entry(scope)
                            .or_default()
                            .insert(name.clone());
                    }
                    task_slot.call("take", json!(name));
                }
            }
        });
        Box::new(WebViewListener { task, _slot: slot })
    }
}

impl KeyboardApi for WebViewKeyboard {
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

impl KeySubscription for WebViewListener {}
