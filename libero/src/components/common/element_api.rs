use crate::components::common::PlatformError;

/// An element's rendered pixel size (`getBoundingClientRect`'s
/// `width`/`height`) - e.g. for converting a drag delta into a fraction of
/// the element it's dragging across.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
}

/// A single resolved, currently-valid element - obtained only via
/// [`DomApi::query_selector`](crate::components::common::DomApi::query_selector)
/// or another `ElementApi`'s own scoped queries, never held past the call
/// that produced it.
pub trait ElementApi {
    fn focus(&self) -> Result<(), PlatformError>;
    fn blur(&self) -> Result<(), PlatformError>;
    fn click(&self) -> Result<(), PlatformError>;

    /// Whether this is the currently focused element.
    fn is_focused(&self) -> bool;

    /// This element's rendered pixel size.
    fn dimensions(&self) -> Result<Dimensions, PlatformError>;

    /// This element's total scrollable content size (`scrollWidth`/
    /// `scrollHeight`) - e.g. for converting a scroll percent into a pixel
    /// offset.
    fn scroll_size(&self) -> Result<Dimensions, PlatformError>;

    /// This element's current scroll offset, in pixels (`scrollLeft`,
    /// `scrollTop`).
    fn scroll_offset(&self) -> Result<(f64, f64), PlatformError>;

    /// Sets this element's scroll offset, in pixels.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError>;

    /// Routes every further event from `pointer_id` to this element until
    /// the pointer is released - so a drag keeps tracking once the pointer
    /// leaves the element, and still delivers its `pointerup`. Released
    /// automatically on `pointerup`/`pointercancel`, hence no counterpart.
    fn set_pointer_capture(&self, pointer_id: i32) -> Result<(), PlatformError>;

    /// Finds the first descendant matching `selector`, scoped to this
    /// element's own subtree.
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError>;

    /// Finds every descendant matching `selector`, in DOM order, scoped to
    /// this element's own subtree.
    fn query_selector_all(&self, selector: &str)
    -> Result<Vec<Box<dyn ElementApi>>, PlatformError>;
}
