use std::{future::Future, pin::Pin};

use super::PlatformError;

/// An element's rendered pixel size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
}

/// A read that has to reach the platform. Already resolved on the web and
/// under Blitz; a round-trip through the webview off them, which is why it is
/// a future rather than a plain `Result`.
///
/// **Start the read where the event handler is, and await it wherever.** Under
/// Blitz a read answers when it is *called*, and the document is locked for as
/// long as dioxus is draining tasks - so a read created inside a `spawn` fails
/// where the same read created just before it succeeds.
pub type Read<T> = Pin<Box<dyn Future<Output = Result<T, PlatformError>>>>;

/// One element, however this platform happens to address it. Reached through
/// [`use_element`](crate::hooks::use_element), or a scoped query off another
/// `ElementApi` - never stored.
///
/// Some renderers cannot answer all of it. A call the platform has no way to
/// serve fails with [`PlatformError::Unsupported`] rather than being absent
/// from the type: which parts work is a property of the running renderer, not
/// of the build.
pub trait ElementApi {
    // Commands stay synchronous: nothing reads a result back, so a renderer
    // that can only queue the work still honours them.
    fn focus(&self) -> Result<(), PlatformError>;
    fn blur(&self) -> Result<(), PlatformError>;
    fn click(&self) -> Result<(), PlatformError>;

    /// Whether this is the currently focused element.
    fn is_focused(&self) -> bool;

    /// This element's rendered pixel size.
    fn dimensions(&self) -> Read<Dimensions>;

    /// Top-left corner in client (viewport) coordinates.
    fn client_offset(&self) -> Read<(f64, f64)>;

    /// Total scrollable content size (`scrollWidth`/`scrollHeight`).
    fn scroll_size(&self) -> Read<Dimensions>;

    /// Current scroll offset in pixels (`scrollLeft`, `scrollTop`).
    fn scroll_offset(&self) -> Read<(f64, f64)>;

    /// Sets this element's scroll offset, in pixels.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError>;

    /// Routes further events from `pointer_id` here, so a drag keeps tracking
    /// once the pointer leaves and still gets its `pointerup`. Released
    /// automatically, hence no counterpart.
    fn set_pointer_capture(&self, pointer_id: i32) -> Result<(), PlatformError>;

    /// First descendant matching `selector`.
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError>;

    /// Every descendant matching `selector`, in DOM order.
    fn query_selector_all(&self, selector: &str)
    -> Result<Vec<Box<dyn ElementApi>>, PlatformError>;
}
