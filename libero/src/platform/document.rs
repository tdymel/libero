use super::{ElementApi, backend};

/// The parts of a platform that no element owns. Deliberately tiny: after an
/// element handle can answer everything about its own subtree, "where is focus
/// right now" is the only question left that has no anchor to ask it from.
pub trait DocumentApi {
    /// Whatever currently has focus. Read synchronously from an event handler
    /// this is the element the user acted on, which is how an overlay learns
    /// where to put focus back.
    fn active_element(&self) -> Option<Box<dyn ElementApi>>;
}

/// `None` where the renderer exposes no document - a webview, where Rust holds
/// no handle to the DOM at all, and any headless build.
pub fn document() -> Option<Box<dyn DocumentApi>> {
    backend::document()
}
