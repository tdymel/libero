use crate::components::common::DomApiError;

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
    fn focus(&self) -> Result<(), DomApiError>;
    fn blur(&self) -> Result<(), DomApiError>;
    fn click(&self) -> Result<(), DomApiError>;

    /// Whether this is the currently focused element.
    fn is_focused(&self) -> bool;

    /// This element's rendered pixel size.
    fn dimensions(&self) -> Result<Dimensions, DomApiError>;

    /// Finds the first descendant matching `selector`, scoped to this
    /// element's own subtree.
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, DomApiError>;

    /// Finds every descendant matching `selector`, in DOM order, scoped to
    /// this element's own subtree.
    fn query_selector_all(&self, selector: &str) -> Result<Vec<Box<dyn ElementApi>>, DomApiError>;
}
