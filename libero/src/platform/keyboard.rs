use std::rc::Rc;

use dioxus::prelude::{Event, Key, KeyboardData, Modifiers, MountedData};

use super::backend;

/// A live key subscription. Dropping it unsubscribes.
pub trait KeySubscription {}

/// One key press, seen at the document: the same `Key` and `Modifiers` a
/// `KeyboardData` handler gets. Handed out, never built, hence `non_exhaustive`.
#[non_exhaustive]
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
    /// Whether the platform is repeating a held key. Sticky Keys or a switch
    /// device turn one press into many: a toggle should ignore a repeat.
    pub repeat: bool,
    /// Whether the press started in text entry; set on every press, filtered or not.
    pub(crate) text_entry: bool,
    /// A WebView's observe tags around the target, nearest first; empty elsewhere.
    pub(crate) scopes: Vec<u64>,
}

impl KeyChord {
    /// Whether the press reached `mounted`'s subtree, the element tagged `tag` on
    /// a WebView: focus is on it or inside it.
    pub(crate) fn within(&self, mounted: &Rc<MountedData>, tag: Option<u64>) -> bool {
        tag.is_some_and(|tag| self.scopes.contains(&tag)) || backend::focus_is_in(mounted)
    }
}

/// Hearing a key press anywhere in the document, for a global shortcut: a root
/// `onkeydown` misses presses while focus is on `<body>` or in a portal.
pub trait KeyboardApi {
    /// Calls `callback` for every key press until the returned subscription is
    /// dropped. Returning `true` prevents the default action.
    ///
    /// Never return `true` for Tab: this listens in capture on the window, so
    /// focus could not move anywhere. Answer one chord, `false` for the rest.
    ///
    /// Presses into text entry (a text `input`, `textarea`, `select`,
    /// `contenteditable`) never arrive. A debug build warns once on a reserved
    /// chord (Tab, Escape, arrows, Ctrl+L and the like).
    ///
    /// ```no_run
    /// # use dioxus::prelude::*;
    /// # use libero::platform::keyboard;
    /// # fn app() -> Element {
    /// let _hotkey = use_hook(|| {
    ///     keyboard().map(|keyboard| {
    ///         std::rc::Rc::new(keyboard.on_key(Box::new(|chord| {
    ///             let open = chord.modifiers.ctrl() && chord.key == Key::Character("k".into());
    ///             open && !chord.repeat
    ///         })))
    ///     })
    /// });
    /// # rsx! {}
    /// # }
    /// ```
    fn on_key(&self, callback: Box<dyn Fn(KeyChord) -> bool>) -> Box<dyn KeySubscription>;

    /// [`on_key`](Self::on_key) without the text-entry filter, for Escape: a
    /// surface must close from its own text field. Answer one chord only.
    fn on_key_unfiltered(
        &self,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription>;
}

/// The keyboard, `None` on a server. Blitz hears presses bubbling out of
/// `LiberoProvider`; a WebView prevents a default only from the second press.
pub fn keyboard() -> Option<&'static dyn KeyboardApi> {
    backend::keyboard()
}

/// Whether something nearer already took the press (prevented its default), so
/// one Escape closes one overlay layer. Also asks the web's capture listener.
pub(crate) fn key_taken(event: &Event<KeyboardData>) -> bool {
    !event.default_action_enabled() || backend::key_taken(event)
}

/// Whether this press landed in text entry. A root key handler acting on arrows,
/// Home or End asks this first, so a nested field keeps its caret.
pub(crate) fn typing_target(event: &Event<KeyboardData>) -> bool {
    backend::typing_target(event)
}

/// Whether this press landed on a control that steps on arrows ([`takes_arrows`]).
/// Ask it beside [`typing_target`], never instead: the two are complements.
pub(crate) fn arrow_target(event: &Event<KeyboardData>) -> bool {
    backend::arrow_target(event)
}

/// Whether a text field's caret sits at its start and at its end, `false` with
/// text selected; `None` off a text field or where the renderer cannot tell.
pub(crate) fn caret_edges(event: &Event<KeyboardData>) -> Option<(bool, bool)> {
    backend::caret_edges(event)
}

/// Whether the platform's shortcut modifier is Cmd (Apple) rather than Ctrl.
pub(crate) fn mod_is_meta() -> bool {
    backend::mod_is_meta()
}

/// Whether a tap on a field raises a soft keyboard that covers its dropdown: a
/// mobile WebView app. The web answers `false`, touch screens included.
pub(crate) fn soft_keyboard_app() -> bool {
    backend::soft_keyboard_app()
}

/// Runs `onback` on each Android Back that pops a [`back_entry`]; `false` where
/// Back never reaches the page (no WebView).
#[cfg(target_os = "android")]
pub(crate) fn watch_back(onback: impl Fn() + 'static) -> bool {
    backend::watch_back(onback)
}

/// Adds the history entry Back pops, or with `false` takes it back unheard.
#[cfg(target_os = "android")]
pub(crate) fn back_entry(armed: bool) {
    backend::back_entry(armed);
}

/// Whether this press landed in right-to-left content: the target's computed
/// `direction` on the web, the focused node's natively and in a WebView (read
/// when it took focus). `false` on a server.
pub(crate) fn rtl_target(event: &Event<KeyboardData>) -> bool {
    backend::rtl_target(event)
}

/// The press's key with ArrowLeft and ArrowRight swapped under RTL, so a
/// handler matching "ArrowRight = next" reads the logical key. Home, End and
/// the vertical arrows pass through.
pub(crate) fn logical_key(event: &Event<KeyboardData>) -> Key {
    let key = event.key();
    // Direction is read only for the two keys it changes.
    if matches!(key, Key::ArrowLeft | Key::ArrowRight) {
        logical_arrow(key, rtl_target(event))
    } else {
        key
    }
}

/// ArrowLeft and ArrowRight swapped when `rtl`, every other key unchanged.
pub(crate) fn logical_arrow(key: Key, rtl: bool) -> Key {
    match key {
        Key::ArrowLeft if rtl => Key::ArrowRight,
        Key::ArrowRight if rtl => Key::ArrowLeft,
        key => key,
    }
}

/// Whether an element takes typing: the filter behind [`KeyboardApi::on_key`].
/// `tag` upper case, as `tagName`; a clicked `input` type does not count.
pub(crate) fn takes_typing(tag: &str, input_type: Option<&str>) -> bool {
    match tag {
        "TEXTAREA" | "SELECT" => true,
        "INPUT" => !input_type.is_some_and(|kind| {
            CLICKED_INPUT_TYPES.contains(&kind.trim().to_ascii_lowercase().as_str())
        }),
        _ => false,
    }
}

/// The `input` types [`takes_typing`] does not count, lower case. Also handed
/// to the WebView's listener, which filters in the page's script.
pub(crate) const CLICKED_INPUT_TYPES: &[&str] = &[
    "checkbox", "radio", "button", "submit", "reset", "image", "file", "range", "color", "hidden",
];

/// Whether an element steps on arrows without taking typing: a `range` or
/// `radio`, the complement of [`takes_typing`]. A backstop for raw caller HTML.
pub(crate) fn takes_arrows(tag: &str, input_type: Option<&str>) -> bool {
    tag == "INPUT"
        && input_type.is_some_and(|kind| {
            matches!(kind.trim().to_ascii_lowercase().as_str(), "range" | "radio")
        })
}

/// What a global hotkey on this chord would take from the user, or `None`.
/// Navigation keys count with any modifier; browser shortcuts with Ctrl/Cmd.
pub(crate) fn reserved_chord(key: &Key, modifiers: Modifiers) -> Option<&'static str> {
    let named = match key {
        Key::Tab => Some("moving focus"),
        Key::Escape => Some("closing and cancelling"),
        Key::Enter => Some("activating the focused control"),
        Key::ArrowUp | Key::ArrowDown | Key::ArrowLeft | Key::ArrowRight => {
            Some("arrow-key navigation and scrolling")
        }
        Key::Home | Key::End | Key::PageUp | Key::PageDown => Some("scrolling and list navigation"),
        Key::F5 => Some("the browser's reload"),
        Key::F6 => Some("moving focus between the page and the browser"),
        Key::Character(text) if text == " " => Some("activating the focused control"),
        _ => None,
    };
    if named.is_some() {
        return named;
    }

    let Key::Character(text) = key else {
        return None;
    };
    if !(modifiers.ctrl() || modifiers.meta()) || modifiers.alt() {
        return None;
    }
    match text.to_ascii_lowercase().as_str() {
        "a" | "c" | "v" | "x" | "z" | "y" => Some("the browser's editing shortcuts"),
        "f" | "g" => Some("the browser's find"),
        "l" => Some("the browser's address bar"),
        "r" => Some("the browser's reload"),
        "n" | "t" | "w" | "q" => Some("the browser's tab and window shortcuts"),
        _ => None,
    }
}

/// The dev-time warning for a global hotkey on a [reserved](reserved_chord)
/// chord, or `None` when the chord is free.
pub(crate) fn reserved_chord_warning(key: &Key, modifiers: Modifiers) -> Option<String> {
    reserved_chord(key, modifiers).map(|what| {
        let mut chord = String::new();
        // Ctrl and Cmd as one: a hotkey binds both.
        for (held, name) in [
            (modifiers.ctrl() || modifiers.meta(), "Ctrl/Cmd+"),
            (modifiers.alt(), "Alt+"),
            (modifiers.shift(), "Shift+"),
        ] {
            if held {
                chord.push_str(name);
            }
        }
        match key {
            Key::Character(text) if text == " " => chord.push_str("Space"),
            Key::Character(text) => chord.push_str(&text.to_ascii_uppercase()),
            other => chord.push_str(&other.to_string()),
        }
        format!(
            "global hotkey {chord} overrides {what}. Keyboard and screen-reader users \
             rely on it - pick another chord"
        )
    })
}

/// Says [`reserved_chord_warning`] once per chord: hotkeys re-subscribe often.
pub(crate) fn warn_reserved_chord(key: &Key, modifiers: Modifiers) {
    if !cfg!(debug_assertions) {
        return;
    }
    thread_local! {
        static WARNED: std::cell::RefCell<std::collections::HashSet<String>> =
            Default::default();
    }
    if let Some(message) = reserved_chord_warning(key, modifiers)
        && WARNED.with_borrow_mut(|warned| warned.insert(message.clone()))
    {
        crate::utils::warn(&message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_entry_is_typing_and_clicked_controls_are_not() {
        for kind in [
            None,
            Some("text"),
            Some("search"),
            Some("Email"),
            Some("number"),
            Some("date"),
            Some("bogus"),
        ] {
            assert!(takes_typing("INPUT", kind), "{kind:?}");
        }
        for kind in [
            "checkbox", "radio", "RADIO", "button", "submit", "range", "color", "file",
        ] {
            assert!(!takes_typing("INPUT", Some(kind)), "{kind}");
        }
        assert!(takes_typing("TEXTAREA", None));
        assert!(takes_typing("SELECT", None));
        assert!(!takes_typing("BUTTON", None));
        assert!(!takes_typing("DIV", None));
    }

    /// The two predicates are complements: no element is on both lists.
    #[test]
    fn stepping_controls_take_the_arrows_without_taking_typing() {
        for kind in ["range", "radio", "RANGE"] {
            assert!(takes_arrows("INPUT", Some(kind)), "{kind}");
            assert!(!takes_typing("INPUT", Some(kind)), "{kind}");
        }
        // The arrows do nothing here, so a component may act on them.
        for kind in ["checkbox", "button", "file", "color"] {
            assert!(!takes_arrows("INPUT", Some(kind)), "{kind}");
            assert!(!takes_typing("INPUT", Some(kind)), "{kind}");
        }
        // Text entry takes the arrows too, and it is `takes_typing`'s.
        for (tag, kind) in [("INPUT", None), ("TEXTAREA", None), ("SELECT", None)] {
            assert!(!takes_arrows(tag, kind), "{tag}");
            assert!(takes_typing(tag, kind), "{tag}");
        }
        assert!(!takes_arrows("DIV", None));
    }

    #[test]
    fn rtl_swaps_only_the_horizontal_arrows() {
        assert_eq!(logical_arrow(Key::ArrowLeft, true), Key::ArrowRight);
        assert_eq!(logical_arrow(Key::ArrowRight, true), Key::ArrowLeft);
        assert_eq!(logical_arrow(Key::ArrowLeft, false), Key::ArrowLeft);
        for key in [Key::ArrowUp, Key::ArrowDown, Key::Home, Key::End] {
            assert_eq!(logical_arrow(key.clone(), true), key);
        }
    }

    #[test]
    fn a11y_keys_are_reserved_with_any_modifier() {
        for key in [
            Key::Tab,
            Key::Escape,
            Key::Enter,
            Key::ArrowLeft,
            Key::Home,
            Key::PageDown,
        ] {
            assert!(reserved_chord(&key, Modifiers::empty()).is_some(), "{key}");
            assert!(reserved_chord(&key, Modifiers::SHIFT).is_some(), "{key}");
        }
        assert!(reserved_chord(&Key::Character(" ".into()), Modifiers::empty()).is_some());
    }

    #[test]
    fn browser_shortcuts_are_reserved_only_with_ctrl_or_cmd() {
        for letter in ["l", "T", "w", "r", "f", "c"] {
            let key = Key::Character(letter.into());
            assert!(
                reserved_chord(&key, Modifiers::CONTROL).is_some(),
                "{letter}"
            );
            assert!(reserved_chord(&key, Modifiers::META).is_some(), "{letter}");
            assert!(
                reserved_chord(&key, Modifiers::empty()).is_none(),
                "{letter}"
            );
        }
        for letter in ["k", "j", "p", "b"] {
            let key = Key::Character(letter.into());
            assert!(
                reserved_chord(&key, Modifiers::CONTROL).is_none(),
                "{letter}"
            );
        }
    }

    #[test]
    fn the_warning_names_the_chord() {
        let warning =
            reserved_chord_warning(&Key::Character("l".into()), Modifiers::CONTROL).unwrap();
        assert!(warning.contains("Ctrl/Cmd+L"), "{warning}");
        assert!(warning.contains("address bar"), "{warning}");
        assert!(reserved_chord_warning(&Key::Character("k".into()), Modifiers::CONTROL).is_none());
    }
}
