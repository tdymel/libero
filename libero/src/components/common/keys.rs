use crate::platform::meta_held;

/// Whether a key press is a shortcut rather than typing. Not Shift: it types capitals.
pub(crate) fn has_shortcut_modifier(event: &dioxus::prelude::KeyboardEvent) -> bool {
    use dioxus::prelude::ModifiersInteraction;

    let modifiers = event.data().modifiers();
    modifiers.ctrl() || modifiers.alt() || meta_held(modifiers)
}

/// What a navigation key under Ctrl, Alt or Meta means to a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NavigationChord {
    /// Alt+ArrowDown: a combobox opens its popup (APG).
    Open,
    /// Alt+ArrowUp: a combobox closes its popup (APG).
    Close,
    /// Any other: the text caret's or the browser's (Alt+ArrowLeft is Back).
    Browser,
}

/// The press's [`NavigationChord`]; `None` for a plain press or another key.
pub(crate) fn navigation_chord(event: &dioxus::prelude::KeyboardEvent) -> Option<NavigationChord> {
    use dioxus::prelude::{Key, ModifiersInteraction, NamedKey};

    let navigation = matches!(
        event.key(),
        Key::Named(NamedKey::ArrowDown)
            | Key::Named(NamedKey::ArrowUp)
            | Key::Named(NamedKey::ArrowLeft)
            | Key::Named(NamedKey::ArrowRight)
            | Key::Named(NamedKey::Home)
            | Key::Named(NamedKey::End)
            | Key::Named(NamedKey::PageUp)
            | Key::Named(NamedKey::PageDown)
    );
    if !navigation || !has_shortcut_modifier(event) {
        return None;
    }
    let modifiers = event.data().modifiers();
    let alt_only = modifiers.alt() && !modifiers.ctrl() && !meta_held(modifiers);
    Some(match event.key() {
        Key::Named(NamedKey::ArrowDown) if alt_only => NavigationChord::Open,
        Key::Named(NamedKey::ArrowUp) if alt_only => NavigationChord::Close,
        _ => NavigationChord::Browser,
    })
}
