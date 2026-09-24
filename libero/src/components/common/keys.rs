/// Whether a key press is a shortcut rather than typing. Not Shift: it types capitals.
pub(crate) fn has_shortcut_modifier(event: &dioxus::prelude::KeyboardEvent) -> bool {
    use dioxus::prelude::ModifiersInteraction;

    let modifiers = event.data().modifiers();
    modifiers.ctrl() || modifiers.alt() || modifiers.meta()
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
    use dioxus::prelude::{Key, ModifiersInteraction};

    let navigation = matches!(
        event.key(),
        Key::ArrowDown
            | Key::ArrowUp
            | Key::ArrowLeft
            | Key::ArrowRight
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
    );
    if !navigation || !has_shortcut_modifier(event) {
        return None;
    }
    let modifiers = event.data().modifiers();
    let alt_only = modifiers.alt() && !modifiers.ctrl() && !modifiers.meta();
    Some(match event.key() {
        Key::ArrowDown if alt_only => NavigationChord::Open,
        Key::ArrowUp if alt_only => NavigationChord::Close,
        _ => NavigationChord::Browser,
    })
}
