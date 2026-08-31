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
    /// **A press the user is typing never arrives.** The capability drops
    /// anything targeting an `input`, `textarea`, `select` or a
    /// `contenteditable` - Mantine's `tagsToIgnore`, enforced here rather than
    /// left to each consumer. So a shortcut cannot eat a character out of a
    /// text field, and equally cannot fire while focus is in one: a palette
    /// opens from the page, not from a search box. Focus on `<body>` or inside
    /// a portal is unaffected, which is the case an element's own `onkeydown`
    /// could not reach.
    ///
    /// Where that is wrong, use [`on_key_unfiltered`](Self::on_key_unfiltered).
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

/// `None` where the renderer cannot report a document-level key press -
/// everything but the web today, the same hole [`scroll`](super::scroll) has and
/// for the same reason.
pub fn keyboard() -> Option<Box<dyn KeyboardApi>> {
    backend::keyboard()
}
