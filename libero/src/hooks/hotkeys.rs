use std::{cell::RefCell, rc::Rc, str::FromStr};

use dioxus::prelude::*;

use crate::hooks::{
    ElementHandle,
    popover::{OpenPopups, focus_in_popup_of, use_open_popups},
};
use crate::localization::ShortcutHelpLabels;
use crate::platform::{KeyChord, KeySubscription, keyboard, mod_is_meta, warn_reserved_chord};

/// One keyboard shortcut for [`use_hotkeys`]: a chord such as `"mod+k"` and what
/// to run when it is pressed.
///
/// A chord is modifiers and a key joined by `+`: `ctrl`, `alt`, `shift`, `meta`
/// (or `cmd`), `mod`, then the key, a character or a name like `f8`, `escape`,
/// `space`. `mod` is Cmd on Apple platforms and Ctrl elsewhere. Modifiers must
/// match exactly, apart from Shift on a symbol such as `?`.
pub struct Hotkey {
    chord: String,
    handler: Box<dyn FnMut()>,
    include_editable: bool,
    guard: Option<Rc<dyn Fn() -> bool>>,
    within: Vec<ElementHandle>,
}

impl Hotkey {
    pub fn new(chord: impl Into<String>, handler: impl FnMut() + 'static) -> Self {
        Self {
            chord: chord.into(),
            handler: Box::new(handler),
            include_editable: false,
            guard: None,
            within: Vec::new(),
        }
    }

    /// Lets the shortcut through only while focus is on `element` or inside it,
    /// or in a popup opened from inside it, such as a `Menu`, `Select` or `HoverCard`.
    /// Call it again to add an element, such as your own portaled box: focus in any
    /// of them counts. A press elsewhere keeps its default action.
    ///
    /// Spread `..element.attributes()` on the element, so a WebView finds it. There
    /// a `use_popover` box counts once its `anchor_events()` and `floating_events()` are spread.
    ///
    /// ```rust
    /// # use dioxus::prelude::*;
    /// # use libero::hooks::{Hotkey, use_element, use_hotkeys};
    /// # fn app() -> Element {
    /// let editor = use_element();
    /// let mut bold = use_signal(|| false);
    /// use_hotkeys([Hotkey::new("mod+b", move || bold.toggle())
    ///     .include_editable(true)
    ///     .within(editor)]);
    ///
    /// rsx! {
    ///     div { onmounted: editor.mount(), ..editor.attributes(),
    ///         textarea {}
    ///     }
    /// }
    /// # }
    /// ```
    pub fn within(mut self, element: ElementHandle) -> Self {
        self.within.push(element);
        self
    }

    /// Whether the shortcut also fires while typing in a text field, a
    /// `textarea`, a `select` or editable content. Off by default.
    pub fn include_editable(mut self, include: bool) -> Self {
        self.include_editable = include;
        self
    }

    /// Lets the shortcut through only while `guard` answers `true`, checked at
    /// each press. A shortcut the guard turns away keeps its default action.
    pub fn when(mut self, guard: impl Fn() -> bool + 'static) -> Self {
        self.guard = Some(Rc::new(guard));
        self
    }
}

/// A parsed chord.
#[derive(Clone, Debug, PartialEq)]
struct Chord {
    key: Key,
    ctrl: bool,
    meta: bool,
    alt: bool,
    shift: bool,
}

impl Chord {
    /// `None` for a chord with no key or a name no key has. `apple` is where
    /// `mod` means Cmd.
    fn parse(text: &str, apple: bool) -> Option<Self> {
        let text = text.trim();
        let (modifiers, key) = match text {
            "+" => ("", "+"),
            _ if text.ends_with("++") => (&text[..text.len() - 2], "+"),
            _ => text.rsplit_once('+').unwrap_or(("", text)),
        };
        let mut chord = Chord {
            key: parse_key(key.trim())?,
            ctrl: false,
            meta: false,
            alt: false,
            shift: false,
        };
        for name in modifiers
            .split('+')
            .map(str::trim)
            .filter(|n| !n.is_empty())
        {
            match name.to_ascii_lowercase().as_str() {
                "ctrl" | "control" => chord.ctrl = true,
                "meta" | "cmd" | "command" => chord.meta = true,
                "alt" | "option" => chord.alt = true,
                "shift" => chord.shift = true,
                "mod" if apple => chord.meta = true,
                "mod" => chord.ctrl = true,
                _ => return None,
            }
        }
        Some(chord)
    }

    fn matches(&self, pressed: &KeyChord) -> bool {
        let held = pressed.modifiers;
        let key_matches = match (&self.key, &pressed.key) {
            (Key::Character(want), Key::Character(got)) => want.eq_ignore_ascii_case(got),
            (want, got) => want == got,
        };
        // A symbol's shift is already in the key: `?` is Shift+/ on most layouts.
        let symbol = matches!(&self.key, Key::Character(text)
            if !text.chars().any(char::is_alphanumeric));
        key_matches
            && self.ctrl == held.ctrl()
            && self.meta == held.meta()
            && self.alt == held.alt()
            && (symbol || self.shift == held.shift())
    }
}

fn parse_key(text: &str) -> Option<Key> {
    let lower = text.to_ascii_lowercase();
    match lower.as_str() {
        "" => return None,
        "space" => return Some(Key::Character(" ".into())),
        "esc" => return Some(Key::Escape),
        _ => {}
    }
    if text.chars().count() == 1 {
        return Some(Key::Character(lower));
    }
    let mut capitalised = lower.clone();
    if let Some(first) = capitalised.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    [text.to_owned(), capitalised, text.to_ascii_uppercase()]
        .iter()
        .find_map(|name| Key::from_str(name).ok())
        .filter(|key| !matches!(key, Key::Unidentified | Key::Character(_)))
}

struct Entry {
    chord: Option<Chord>,
    handler: Box<dyn FnMut()>,
    include_editable: bool,
    guard: Option<Rc<dyn Fn() -> bool>>,
    within: Vec<ElementHandle>,
}

impl Entry {
    /// No scope, or focus in one of its mounted elements or a popup opened from one.
    fn in_scope(&self, pressed: &KeyChord, popups: OpenPopups) -> bool {
        self.within.is_empty()
            || self.within.iter().any(|element| {
                element.try_mounted().is_some_and(|mounted| {
                    pressed.within(&mounted, element.tag()) || focus_in_popup_of(popups, &mounted)
                })
            })
    }
}

/// Runs each [`Hotkey`]'s handler when its chord is pressed anywhere in the
/// document, focus on `<body>` or in a portal included, or only [within an
/// element](Hotkey::within). Presses that start in
/// text entry are ignored unless the hotkey [includes editable
/// targets](Hotkey::include_editable); a press it takes has its default action
/// prevented, held-key repeats included, while the handler runs once per
/// press. Stops listening when the component unmounts.
///
/// Bindings are read again every render, so handlers see current state. A
/// debug build warns once about a chord that would shadow a browser or
/// accessibility shortcut such as Tab, Escape or Ctrl+L. Where there is no
/// keyboard, as in a server render, nothing is bound.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::{Hotkey, use_hotkeys};
/// # fn app() -> Element {
/// let mut searching = use_signal(|| false);
/// use_hotkeys([
///     Hotkey::new("mod+k", move || searching.toggle()),
///     Hotkey::new("shift+?", move || searching.set(true)),
/// ]);
///
/// rsx! {
///     if searching() { p { "Search" } }
/// }
/// # }
/// ```
pub fn use_hotkeys(bindings: impl IntoIterator<Item = Hotkey>) {
    let apple = mod_is_meta();
    let entries: Vec<Entry> = bindings
        .into_iter()
        .map(|binding| Entry {
            chord: Chord::parse(&binding.chord, apple),
            handler: binding.handler,
            include_editable: binding.include_editable,
            guard: binding.guard,
            within: binding.within,
        })
        .collect();
    let bound = !entries.is_empty();
    let chords: Vec<Chord> = entries.iter().filter_map(|e| e.chord.clone()).collect();

    let shared: Rc<RefCell<Vec<Entry>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    *shared.borrow_mut() = entries;

    // The key callback runs with no runtime on the web: it records, an effect runs the handler.
    let tick = use_signal(|| 0u64);
    let pending: Rc<RefCell<Vec<usize>>> = use_hook(|| Rc::new(RefCell::new(Vec::new())));
    let listening: Rc<RefCell<Vec<Box<dyn KeySubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(Vec::new())));
    use_drop({
        let listening = listening.clone();
        move || listening.borrow_mut().clear()
    });

    use_effect(use_reactive!(|chords| {
        for chord in &chords {
            warn_reserved_chord(&chord.key, held_modifiers(chord));
        }
    }));

    let popups = use_open_popups();
    let subscriptions = listening.clone();
    let owned = shared.clone();
    let queue = pending.clone();
    // One listener, so a press is matched once: with a listener per kind, the
    // handler's effect could run between the two and the second would see its result.
    use_effect(use_reactive!(|bound| {
        subscriptions.borrow_mut().clear();
        let Some(api) = keyboard().filter(|_| bound) else {
            return;
        };
        let entries = owned.clone();
        let queue = queue.clone();
        let callback = Box::new(move |pressed: KeyChord| {
            let hit = entries.borrow().iter().position(|entry| {
                (entry.include_editable || !pressed.text_entry)
                    && entry
                        .chord
                        .as_ref()
                        .is_some_and(|chord| chord.matches(&pressed))
                    && entry.in_scope(&pressed, popups)
                    && entry.guard.as_ref().is_none_or(|guard| guard())
            });
            let Some(index) = hit else {
                return false;
            };
            if !pressed.repeat {
                queue.borrow_mut().push(index);
                let mut tick = tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
            true
        });
        subscriptions
            .borrow_mut()
            .push(api.on_key_unfiltered(callback));
    }));

    let mut seen = use_signal(|| 0u64);
    use_effect(move || {
        let pressed = tick();
        if pressed == *seen.peek() {
            return;
        }
        seen.set(pressed);
        let fired: Vec<usize> = pending.borrow_mut().drain(..).collect();
        for index in fired {
            if let Some(entry) = shared.borrow_mut().get_mut(index) {
                (entry.handler)();
            }
        }
    });
}

/// The key names `chord` shows, modifiers first in the platform's order; `None`
/// for a chord [`use_hotkeys`] would not bind.
pub(crate) fn chord_keys(
    chord: &str,
    apple: bool,
    words: &ShortcutHelpLabels,
) -> Option<Vec<String>> {
    let chord = Chord::parse(chord, apple)?;
    let (alt, meta) = match apple {
        true => (words.option, words.meta),
        false => (words.alt, "Meta"),
    };
    let mut keys: Vec<String> = [
        (chord.ctrl, words.ctrl),
        (chord.alt, alt),
        (chord.shift, words.shift),
        (chord.meta, meta),
    ]
    .into_iter()
    .filter(|(held, _)| *held)
    .map(|(_, name)| name.to_string())
    .collect();
    keys.push(match chord.key {
        Key::Character(text) if text == " " => "Space".to_string(),
        Key::Character(text) => text.to_uppercase(),
        other => other.to_string(),
    });
    Some(keys)
}

/// `chord` as `aria-keyshortcuts` writes it (`Control+Shift+B`); `None` for a chord
/// [`use_hotkeys`] would not bind.
pub(crate) fn aria_keyshortcuts(chord: &str, apple: bool) -> Option<String> {
    let chord = Chord::parse(chord, apple)?;
    let mut keys: Vec<String> = [
        (chord.ctrl, "Control"),
        (chord.alt, "Alt"),
        (chord.shift, "Shift"),
        (chord.meta, "Meta"),
    ]
    .into_iter()
    .filter(|(held, _)| *held)
    .map(|(_, name)| name.to_string())
    .collect();
    keys.push(match chord.key {
        Key::Character(text) if text == " " => "Space".to_string(),
        Key::Character(text) => text.to_uppercase(),
        other => other.to_string(),
    });
    Some(keys.join("+"))
}

fn held_modifiers(chord: &Chord) -> Modifiers {
    let mut modifiers = Modifiers::empty();
    modifiers.set(Modifiers::CONTROL, chord.ctrl);
    modifiers.set(Modifiers::META, chord.meta);
    modifiers.set(Modifiers::ALT, chord.alt);
    modifiers.set(Modifiers::SHIFT, chord.shift);
    modifiers
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pressed(key: Key, modifiers: Modifiers) -> KeyChord {
        KeyChord {
            key,
            modifiers,
            repeat: false,
            text_entry: false,
            scopes: Vec::new(),
        }
    }

    fn character(text: &str) -> Key {
        Key::Character(text.into())
    }

    #[test]
    fn a_chord_shows_the_platforms_key_names() {
        let words = ShortcutHelpLabels::ENGLISH;
        let keys = |chord, apple| chord_keys(chord, apple, &words);
        assert_eq!(keys("shift+mod+k", false).unwrap(), ["Ctrl", "Shift", "K"]);
        assert_eq!(keys("shift+mod+k", true).unwrap(), ["Shift", "Cmd", "K"]);
        assert_eq!(keys("alt+f10", true).unwrap(), ["Option", "F10"]);
        assert_eq!(keys("space", false).unwrap(), ["Space"]);
        assert_eq!(keys("hyper+k", false), None);
    }

    #[test]
    fn mod_is_ctrl_off_apple_and_cmd_on_it() {
        let linux = Chord::parse("mod+k", false).unwrap();
        assert!(linux.ctrl && !linux.meta);
        let mac = Chord::parse("mod+k", true).unwrap();
        assert!(mac.meta && !mac.ctrl);
    }

    #[test]
    fn modifiers_and_named_keys_parse_in_any_case() {
        let chord = Chord::parse("Ctrl+Shift+F8", false).unwrap();
        assert_eq!(chord.key, Key::F8);
        assert!(chord.ctrl && chord.shift && !chord.alt);
        assert_eq!(Chord::parse("esc", false).unwrap().key, Key::Escape);
        assert_eq!(Chord::parse("ArrowUp", false).unwrap().key, Key::ArrowUp);
        assert_eq!(
            Chord::parse("alt+space", false).unwrap().key,
            character(" ")
        );
    }

    #[test]
    fn plus_is_a_key_when_it_ends_the_chord() {
        assert_eq!(Chord::parse("+", false).unwrap().key, character("+"));
        let chord = Chord::parse("mod++", false).unwrap();
        assert_eq!((chord.key, chord.ctrl), (character("+"), true));
    }

    #[test]
    fn nonsense_does_not_parse() {
        for text in ["", "mod+", "hyper+k", "nosuchkey", "ctrl+nosuchkey"] {
            assert_eq!(Chord::parse(text, false), None, "{text:?}");
        }
    }

    #[test]
    fn a_multi_char_key_starting_non_ascii_does_not_parse() {
        for text in ["éa", "ß+ab", "ctrl+ñx"] {
            assert_eq!(Chord::parse(text, false), None, "{text:?}");
            assert_eq!(aria_keyshortcuts(text, false), None, "{text:?}");
        }
        assert_eq!(Chord::parse("é", false).unwrap().key, character("é"));
    }

    #[test]
    fn a_chord_matches_its_key_with_exactly_its_modifiers() {
        let chord = Chord::parse("mod+k", false).unwrap();
        assert!(chord.matches(&pressed(character("k"), Modifiers::CONTROL)));
        assert!(chord.matches(&pressed(character("K"), Modifiers::CONTROL)));
        assert!(!chord.matches(&pressed(character("k"), Modifiers::empty())));
        assert!(!chord.matches(&pressed(character("k"), Modifiers::META)));
        assert!(!chord.matches(&pressed(
            character("k"),
            Modifiers::CONTROL | Modifiers::SHIFT
        )));
        assert!(!chord.matches(&pressed(
            character("k"),
            Modifiers::CONTROL | Modifiers::ALT
        )));
        assert!(!chord.matches(&pressed(character("j"), Modifiers::CONTROL)));
    }

    #[test]
    fn a_bare_named_key_rejects_ctrl_alt_and_meta() {
        let chord = Chord::parse("f8", false).unwrap();
        assert!(chord.matches(&pressed(Key::F8, Modifiers::empty())));
        for held in [Modifiers::CONTROL, Modifiers::ALT, Modifiers::META] {
            assert!(!chord.matches(&pressed(Key::F8, held)));
        }
    }

    fn entry(within: Vec<ElementHandle>) -> Entry {
        Entry {
            chord: Chord::parse("mod+b", false),
            handler: Box::new(|| {}),
            include_editable: false,
            guard: None,
            within,
        }
    }

    #[test]
    fn a_scoped_hotkey_needs_focus_in_a_mounted_element() {
        let mut dom = VirtualDom::new(|| rsx! {});
        dom.rebuild_in_place();
        // `ElementHandle::new` needs a scope.
        dom.in_scope(ScopeId::ROOT, || {
            let press = pressed(character("b"), Modifiers::CONTROL);
            let popups = crate::hooks::popover::open_popups();
            assert!(entry(Vec::new()).in_scope(&press, popups));
            let unmounted = ElementHandle::new();
            assert!(!entry(vec![unmounted]).in_scope(&press, popups));
            assert!(!entry(vec![unmounted, ElementHandle::new()]).in_scope(&press, popups));
        });
    }

    #[test]
    fn a_symbol_ignores_the_shift_that_types_it() {
        let chord = Chord::parse("?", false).unwrap();
        assert!(chord.matches(&pressed(character("?"), Modifiers::SHIFT)));
        assert!(chord.matches(&pressed(character("?"), Modifiers::empty())));
        assert!(!chord.matches(&pressed(character("?"), Modifiers::CONTROL)));
    }
}
