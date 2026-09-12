use super::{Dimensions, ElementApi, Read, backend};

/// The parts of a platform that no element owns. Deliberately tiny: after an
/// element handle can answer everything about its own subtree, "where is focus
/// right now" is the only question left that has no anchor to ask it from.
pub trait DocumentApi {
    /// Whatever currently has focus. Read synchronously from an event handler
    /// this is the element the user acted on, which is how an overlay learns
    /// where to put focus back.
    fn active_element(&self) -> Option<Box<dyn ElementApi>>;

    /// The visible viewport, in CSS pixels. No element owns it, and anything
    /// that has to stay on screen - a flipping popover - needs it.
    fn viewport(&self) -> Read<Dimensions>;

    /// Sets an attribute on the element `:root` matches - the one thing above
    /// the app's own tree that a stylesheet can select on. `None` removes it.
    ///
    /// `false` where the root is not reachable, which is how the theme switch
    /// learns it has to rebuild the sheet instead of flipping an attribute.
    fn set_root_attribute(&self, name: &str, value: Option<&str>) -> bool;

    /// The theme's colours changed wholesale. A renderer that baked colours
    /// into what it built redraws them; one that paints from live styles, the
    /// web, has nothing to do.
    fn colors_changed(&self) {}
}

/// `None` where the renderer exposes no document - a webview, where Rust holds
/// no handle to the DOM at all, and any headless build.
pub fn document() -> Option<&'static dyn DocumentApi> {
    backend::document()
}
