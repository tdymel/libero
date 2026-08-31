use dioxus::prelude::{Key, Modifiers};

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
pub struct KeyChord {
    pub key: Key,
    pub modifiers: Modifiers,
    /// Whether the press landed in something the user is typing into: an
    /// `input`, `textarea` or `select`, or anything `contenteditable`.
    ///
    /// **A hotkey has to check this itself.** Mantine keeps the same rule in
    /// `tagsToIgnore`, and it is a policy, not a fact about the platform: a
    /// palette's Ctrl+K should still open while the user is typing in a search
    /// box, while a bare `/` should not. The capability reports where the press
    /// landed and lets each consumer decide.
    pub editable_target: bool,
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
    fn on_key(&self, callback: Box<dyn Fn(KeyChord) -> bool>) -> Box<dyn KeySubscription>;
}

/// `None` where the renderer cannot report a document-level key press -
/// everything but the web today, the same hole [`scroll`](super::scroll) has and
/// for the same reason.
pub fn keyboard() -> Option<Box<dyn KeyboardApi>> {
    backend::keyboard()
}
