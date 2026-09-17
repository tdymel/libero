use dioxus::prelude::{Event, Key, KeyboardData, Modifiers};

use super::backend;

/// A live key subscription. **Dropping it unsubscribes** - the same contract as
/// [`ScrollSubscription`](super::ScrollSubscription) and
/// [`TimerSubscription`](super::TimerSubscription), and the same empty trait
/// saying so.
pub trait KeySubscription {}

/// One key press, seen at the document.
///
/// The same `Key` and `Modifiers` a `KeyboardData` handler gets, so a component
/// that already matches on `event.key()` matches on this unchanged.
///
/// `#[non_exhaustive]`: a chord is something the platform hands you, never
/// something you build, and the next field to be needed should not be a
/// breaking change.
#[non_exhaustive]
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
    /// Whether the platform is repeating a key the user is still holding.
    ///
    /// **Held keys are not rare and not always deliberate.** A motor
    /// impairment, Sticky Keys, Slow Keys or a switch device all turn one
    /// intended press into a stream of them, and a toggle that fires per press
    /// flaps - Ctrl+K opening and closing a palette over and over.
    ///
    /// Reported rather than debounced, because unlike the text-entry filter
    /// this is a fact about the event and not a policy: a toggle ignores a
    /// repeat, while a held ArrowDown scrolling a list wants every one.
    pub repeat: bool,
}

/// Hearing a key press anywhere in the document, not just inside one subtree.
///
/// Which is the only way a global shortcut can work: an `onkeydown` on an app's
/// root element misses every press while focus is on `<body>` or inside a
/// portal, and both are ordinary states rather than corner cases.
pub trait KeyboardApi {
    /// Calls `callback` for every key press in the document until the returned
    /// subscription is dropped. **Returning `true` prevents the default
    /// action** - what an `event.prevent_default()` would do from a handler,
    /// which is how Ctrl+K opens a palette instead of the browser's search bar.
    ///
    /// **Never return `true` for Tab or Shift+Tab.** This listens in capture on
    /// the window, so preventing the default there stops focus moving anywhere
    /// in the document, and a keyboard or screen-reader user has no way back -
    /// the one mistake on this surface with no recovery. Answer the one chord
    /// the subscription is for and return `false` to everything else.
    ///
    /// **A press the user is typing never arrives.** The capability drops
    /// anything targeting a text-like `input`, a `textarea`, a `select` or a
    /// `contenteditable` - Mantine's `tagsToIgnore`, enforced here rather than
    /// left to each consumer. A checkbox, radio, button or range input is not
    /// text entry, so a hotkey still works after a click on one. So a shortcut cannot eat a character out of a
    /// text field, and equally cannot fire while focus is in one: a palette
    /// opens from the page, not from a search box. Focus on `<body>` or inside
    /// a portal is unaffected, which is the case an element's own `onkeydown`
    /// could not reach.
    ///
    /// Where that is wrong, use [`on_key_unfiltered`](Self::on_key_unfiltered).
    ///
    /// In a debug build, the first press a subscription here takes on a key
    /// the user or the browser relies on - Tab, Escape, Enter, Space, the
    /// arrows, Home/End, PageUp/PageDown, or Ctrl+L, T, W, R, F and the like -
    /// logs a warning.
    fn on_key(&self, callback: Box<dyn Fn(KeyChord) -> bool>) -> Box<dyn KeySubscription>;

    /// [`on_key`](Self::on_key) without the text-entry filter: every press,
    /// wherever it landed.
    ///
    /// **Escape is the reason this exists.** A dismissible surface has to hear
    /// it from inside its own text field - a pointer-opened HoverCard leaves
    /// focus wherever it was, a Menu has a filter field, a palette has a search
    /// box that a second Ctrl+K should still close. Filtering those is not
    /// convenience, it is a surface the keyboard cannot escape from.
    ///
    /// **Per subscription, and never the default.** Opting out changes what
    /// *this* callback receives and nothing else, and a consumer that does not
    /// think about it gets the filtered stream. The cost of choosing this is
    /// that the callback can now swallow a keystroke the user meant for a text
    /// field, so a subscription here answers a specific chord and returns
    /// `false` to everything else.
    fn on_key_unfiltered(
        &self,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription>;
}

/// `None` where the renderer cannot report a document-level key press: the
/// WebView floor and a server.
///
/// Natively (Blitz) a press is heard as it bubbles out of `LiberoProvider`, so
/// a handler that stops its propagation hides it, and nothing is heard while
/// focus sits on `<html>` or outside the provider.
pub fn keyboard() -> Option<&'static dyn KeyboardApi> {
    backend::keyboard()
}

/// Whether something nearer the press already took it. Taking a key is marked
/// the same way everywhere: by preventing its default. An enclosing overlay's
/// bubbled handler asks this before it closes, so one Escape closes one layer.
///
/// Two places can have marked it. An element handler below this one - a field
/// dropdown closing its list - sets the flag on the event itself, which every
/// handler later in the same bubble shares, on every backend. On the web the
/// capture-phase [`KeyboardApi`] listener, which runs ahead of every element
/// handler, marks the native event instead.
pub(crate) fn key_taken(event: &Event<KeyboardData>) -> bool {
    !event.default_action_enabled() || backend::key_taken(event)
}

/// Whether this press landed in something the user types into - a text-like
/// `input`, a `textarea`, a `select` or a `contenteditable`.
///
/// **The component-facing half of the filter [`KeyboardApi::on_key`] applies
/// to a document listener.** A root key handler that acts on the arrows,
/// Home or End has the same obligation and no way to meet it: the press it
/// receives has bubbled out of whatever is focused, so acting on it - and
/// worse, `prevent_default()`ing it - takes the caret out of a text field a
/// caller put inside the component. Ask this first and return.
///
/// The web reads the event's target and Blitz the focused node; every other
/// renderer answers `false`.
pub(crate) fn typing_target(event: &Event<KeyboardData>) -> bool {
    backend::typing_target(event)
}

/// Whether this press landed on a control the browser steps with the arrow
/// keys although it is not text entry - the rule is [`takes_arrows`].
///
/// **Ask it beside [`typing_target`], never instead of it.** The two are
/// complements, so a component filtering the arrows asks both and a component
/// filtering typing asks only the first. Neither name covers the other's
/// elements.
///
/// Like [`typing_target`] it answers about the **element**, not about the key
/// in hand: a `range` inside the component is left alone for Home and End too,
/// which is right, since Home steps a range to its minimum.
///
/// Answered where [`typing_target`] is, `false` elsewhere.
pub(crate) fn arrow_target(event: &Event<KeyboardData>) -> bool {
    backend::arrow_target(event)
}

/// Whether this press landed in right-to-left content: the computed
/// `direction` of the event's target on the web, of the focused node natively
/// (the read behind `ElementHandle::is_rtl`). `false` elsewhere.
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

/// Whether an element takes typing, so a hotkey must not fire from it - the
/// text-entry filter behind [`KeyboardApi::on_key`], kept apart from any one
/// renderer so the next backend applies the same list. `tag` is upper case,
/// the way `tagName` reports it; `input_type` is the raw `type` attribute.
///
/// An `input` counts unless its type is a control the user clicks or ticks. A
/// radio, a checkbox, a button or a range takes no characters, and treating it
/// as text left Ctrl+K dead after a click on a switch or a segmented control.
/// A missing or unknown type is a text box, as it is to the browser. `select`
/// counts: a key press there drives the native option search.
#[cfg_attr(not(any(target_arch = "wasm32", feature = "native")), allow(dead_code))]
pub(crate) fn takes_typing(tag: &str, input_type: Option<&str>) -> bool {
    match tag {
        "TEXTAREA" | "SELECT" => true,
        "INPUT" => !input_type.is_some_and(|kind| {
            matches!(
                kind.trim().to_ascii_lowercase().as_str(),
                "checkbox"
                    | "radio"
                    | "button"
                    | "submit"
                    | "reset"
                    | "image"
                    | "file"
                    | "range"
                    | "color"
                    | "hidden"
            )
        }),
        _ => false,
    }
}

/// Whether an element steps on an arrow key although the user types nothing
/// into it - **the complement of [`takes_typing`], not a superset of it**.
/// `tag` and `input_type` are read the same way.
///
/// A `range` and a `radio` are the whole list: the browser moves a range's
/// value and a radio group's selection on an arrow, and nothing prevents that
/// default, so a component acting on the arrows has no other way to know the
/// press was spoken for. A `textarea`, a `select` and a text `input` also take
/// the arrows, and they are [`takes_typing`]'s, which is why a component
/// filtering the arrows asks both and neither name lies about its own list.
///
/// A checkbox, a button and a file input are on neither list: the arrows do
/// nothing there, so a component may act on them.
///
/// **Our own controls do not need this.** `Slider`, `RadioGroup` and
/// `SegmentedControl` each `prevent_default()` on the arrows, which
/// [`key_taken`] already reports. This is the backstop for raw HTML a caller
/// wrote.
#[cfg_attr(not(any(target_arch = "wasm32", feature = "native")), allow(dead_code))]
pub(crate) fn takes_arrows(tag: &str, input_type: Option<&str>) -> bool {
    tag == "INPUT"
        && input_type.is_some_and(|kind| {
            matches!(kind.trim().to_ascii_lowercase().as_str(), "range" | "radio")
        })
}

/// What a global hotkey on this chord would take away from the user, or `None`
/// if nothing. A development aid: [`reserved_chord_warning`] turns it into the
/// dev-time warning.
///
/// Two lists. The keys that move and act without a mouse - Tab, Escape, Enter,
/// Space, the arrows, Home/End, PageUp/PageDown, F6 - reserved with any
/// modifier, since Shift+Tab or Alt+ArrowLeft is as much the browser's as the
/// bare key. And the browser's own Ctrl (Cmd) shortcuts that a page can take:
/// editing, find, the address bar, reload, and the tab and window ones, which
/// some browsers do not even let a page see.
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
        // Ctrl and Cmd as one: a hotkey binds both, and the warning should
        // read the same whichever a press happened to carry.
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

/// Says [`reserved_chord_warning`] once per chord for the page's lifetime -
/// a hotkey is re-subscribed on every open and close, and the second telling
/// adds nothing.
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

    /// The two predicates are complements: a component filtering the arrows
    /// asks both, so an element on both lists would be asked about twice and
    /// one on neither is a press the component may act on.
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
