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

    /// A `<form>`'s own reset: every control back to its default value, except
    /// one marked `data-controlled`, whose component sets its value - that one
    /// keeps what it showed.
    fn reset(&self) -> Result<(), PlatformError>;

    /// Submits a `<form>` as its submit button would, so `onsubmit` fires with
    /// a real event. Only the web has one to fire.
    fn request_submit(&self) -> Result<(), PlatformError>;

    /// Whether this is the currently focused element.
    fn is_focused(&self) -> bool;

    /// Whether this node is still in the document - the DOM's `isConnected`.
    ///
    /// A plain `bool` rather than a [`Read`], like
    /// [`is_focused`](Self::is_focused): every renderer that can answer it at
    /// all can answer it synchronously, and the one caller that matters -
    /// handing focus back to a trigger the application has since deleted -
    /// has to decide *inside* the closing handler, before a `spawn`.
    ///
    /// **A renderer that cannot tell answers `true`.** `focus()` on a detached
    /// node already returns `Ok(())` everywhere and does nothing, so the
    /// optimistic answer is the one that leaves behaviour where it was; `false`
    /// would claim every element on that renderer had been removed.
    fn is_connected(&self) -> bool;

    /// This element's rendered pixel size.
    fn dimensions(&self) -> Read<Dimensions>;

    /// Top-left corner in client (viewport) coordinates.
    fn client_offset(&self) -> Read<(f64, f64)>;

    /// Total scrollable content size (`scrollWidth`/`scrollHeight`).
    fn scroll_size(&self) -> Read<Dimensions>;

    /// Current scroll offset in pixels (`scrollLeft`, `scrollTop`).
    fn scroll_offset(&self) -> Read<(f64, f64)>;

    /// An `<img>`'s intrinsic pixel size (`naturalWidth`/`naturalHeight`).
    ///
    /// [`PlatformError::NotFound`] until the picture has decoded, and for any
    /// element that is not an image: there is no size to report yet, which is
    /// different from a renderer that cannot say.
    fn natural_size(&self) -> Read<Dimensions>;

    /// Sets this element's scroll offset, in pixels.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError>;

    /// Scrolls the nearest scrollable ancestor, vertically, just far enough to
    /// show this element and its `scroll-margin` - `scrollIntoView`'s
    /// `nearest`, without calling it. Chromium moves its sequential focus
    /// starting point to whatever `scrollIntoView` shows, so on page load the
    /// next Tab would start from there rather than from the top of the page.
    fn scroll_into_view(&self, smooth: bool) -> Result<(), PlatformError>;

    /// Replaces this `<input type="file">`'s own file list.
    ///
    /// A `FileList` is the only thing a form posts, and it cannot be edited:
    /// removing one file of three, or clearing a field, happens in Rust and
    /// would otherwise leave the input still holding what the picker produced.
    /// This writes the list back, so what posts is what the caller holds.
    ///
    /// Only the web can serve it (through a `DataTransfer`); elsewhere it is
    /// [`PlatformError::Unsupported`], and a native form post is not a thing
    /// there either.
    fn set_files(&self, files: &[dioxus::html::FileData]) -> Result<(), PlatformError>;

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
