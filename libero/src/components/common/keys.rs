use dioxus::prelude::Modifiers;

use crate::platform::meta_held;

/// AltGr, which types a character: reported as itself, or on Windows as Ctrl+Alt without Meta.
/// For typing paths only; an explicit Ctrl+Alt hotkey chord still fires.
pub(crate) fn is_alt_graph(modifiers: Modifiers) -> bool {
    modifiers.contains(Modifiers::ALT_GRAPH)
        || (modifiers.ctrl() && modifiers.alt() && !meta_held(modifiers))
}

/// Whether a key press is a shortcut rather than typing. Not Shift: it types capitals,
/// nor a character under AltGr (todo 3848).
pub(crate) fn has_shortcut_modifier(event: &dioxus::prelude::KeyboardEvent) -> bool {
    use dioxus::prelude::{Key, ModifiersInteraction};

    shortcut_modifier(
        event.data().modifiers(),
        matches!(event.key(), Key::Character(_)),
    )
}

fn shortcut_modifier(modifiers: Modifiers, character: bool) -> bool {
    let typing = character && is_alt_graph(modifiers);
    !typing && (modifiers.ctrl() || modifiers.alt() || meta_held(modifiers))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_character_under_alt_graph_is_typing_not_a_shortcut() {
        let alt_graph = Modifiers::CONTROL | Modifiers::ALT;
        assert!(!shortcut_modifier(alt_graph, true));
        assert!(shortcut_modifier(alt_graph, false));
        assert!(shortcut_modifier(Modifiers::CONTROL, true));
        assert!(shortcut_modifier(Modifiers::ALT, true));
        assert!(shortcut_modifier(Modifiers::META | alt_graph, true));
        assert!(!shortcut_modifier(Modifiers::SHIFT, true));
        assert!(is_alt_graph(alt_graph) && !is_alt_graph(Modifiers::CONTROL));
        assert!(is_alt_graph(Modifiers::ALT_GRAPH));
        assert!(!shortcut_modifier(
            Modifiers::ALT_GRAPH | Modifiers::CONTROL,
            true
        ));
    }
}
