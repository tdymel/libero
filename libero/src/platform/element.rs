use std::{future::Future, pin::Pin, rc::Rc};

use dioxus::prelude::MountedData;

use super::{PlatformError, backend};

/// Dropping it stops the callbacks: a content watch, a form's `reset`.
pub(crate) trait ContentSubscription {}

/// Calls `callback` after `mounted`'s subtree changed (nodes, text, focusability).
/// Blitz sees only scroll-size changes, within 0.5 s; elsewhere off the web `None`.
pub(crate) fn on_content_change(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    backend::on_content_change(mounted, callback)
}

/// Calls `on_reset` on each `reset` of the `<form>` owning `mounted`. `None` when
/// no form owns it, and off the web.
pub(crate) fn on_form_reset(
    mounted: &Rc<MountedData>,
    on_reset: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    backend::on_form_reset(mounted, on_reset)
}

/// Sets the `value` of the element with `id`, for a WebView, whose handles cannot
/// set one (1030). `Unsupported` elsewhere: an [`ElementApi`] there can.
pub(crate) fn set_value_by_id(id: &str, value: &str) -> Result<(), PlatformError> {
    backend::set_value_by_id(id, value)
}

/// Focuses the page's first match of `selector`, for a WebView, where no handle
/// can query (1191). `Unsupported` elsewhere: `query_selector` there can.
pub(crate) fn focus_selector(selector: &str) -> Result<(), PlatformError> {
    backend::focus_selector(selector)
}

/// Where [`focus_among`] moves focus.
#[derive(Clone, Copy)]
#[cfg_attr(any(target_arch = "wasm32", feature = "native"), allow(dead_code))]
pub(crate) enum FocusStep {
    /// `by` places from the focused one; `wrap` past the ends, else it stops there.
    By {
        by: isize,
        wrap: bool,
    },
    First,
    Last,
}

/// Moves focus among the page's `[attr]` elements valued in `values`, in DOM
/// order, for a WebView, where no handle can query (1191). `Unsupported` elsewhere.
pub(crate) fn focus_among(
    attr: &str,
    values: &[String],
    to: FocusStep,
) -> Result<(), PlatformError> {
    backend::focus_among(attr, values, to)
}

/// The focused element's own or nearest ancestor's `attr`, for a WebView, which
/// reads no attribute in a handler. `Unsupported` elsewhere.
pub(crate) fn focused_attribute(attr: &str) -> super::Read<Option<String>> {
    backend::focused_attribute(attr)
}

/// Keeps the focused element page-side, for a WebView, which holds no active
/// element (no `DocumentApi`). `None` elsewhere. [`focus_kept`] focuses it.
pub(crate) fn keep_focused() -> Option<u64> {
    backend::keep_focused()
}

/// Focuses what [`keep_focused`] kept under `token`, once.
pub(crate) fn focus_kept(token: u64) {
    backend::focus_kept(token);
}

/// Whether `mounted` lays out right to left: its computed `direction`, which
/// `dir="rtl"` on it or an ancestor sets. `false` where the renderer cannot say.
pub(crate) fn is_rtl(mounted: &Rc<MountedData>) -> bool {
    backend::is_rtl(mounted)
}

/// Blitz's `scroll_into_view` reads this (one `px`/`rem` length) since stylo
/// does not parse `scroll-margin`. Set it beside `scroll-margin`.
pub(crate) const SCROLL_MARGIN_VAR: &str = "--lsx-scroll-margin";

/// An element's rendered pixel size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimensions {
    pub width: f64,
    pub height: f64,
}

/// A read that has to reach the platform: resolved at once on the web and Blitz,
/// a round-trip through a WebView.
///
/// Start the read in the event handler, await it anywhere: under Blitz a read
/// made inside a `spawn` fails, the document is locked while tasks drain.
pub type Read<T> = Pin<Box<dyn Future<Output = Result<T, PlatformError>>>>;

/// One element, reached through [`use_element`](crate::hooks::use_element) or a
/// scoped query, never stored. What a renderer can't serve is `Unsupported`.
///
/// ```
/// # use libero::platform::{ElementApi, PlatformError};
/// # async fn f(element: &dyn ElementApi) -> Result<(), PlatformError> {
/// let size = element.dimensions().await?;
/// element.focus()?;
/// # let _ = size;
/// # Ok(())
/// # }
/// ```
pub trait ElementApi {
    // Commands stay synchronous, so a renderer that only queues still honours them.
    fn focus(&self) -> Result<(), PlatformError>;
    fn blur(&self) -> Result<(), PlatformError>;
    fn click(&self) -> Result<(), PlatformError>;

    /// A `<form>`'s reset: every control back to its default, except one marked
    /// `data-controlled`, which keeps what it showed.
    fn reset(&self) -> Result<(), PlatformError>;

    /// Submits a `<form>` as its submit button would, so `onsubmit` fires with
    /// a real event. Web only.
    fn request_submit(&self) -> Result<(), PlatformError>;

    /// Whether this is the currently focused element.
    fn is_focused(&self) -> bool;

    /// Whether this node is still in the document (`isConnected`). Synchronous,
    /// for a closing handler; a renderer that cannot tell answers `true`.
    fn is_connected(&self) -> bool;

    /// This element's rendered pixel size.
    fn dimensions(&self) -> Read<Dimensions>;

    /// Top-left corner in client (viewport) coordinates.
    fn client_offset(&self) -> Read<(f64, f64)>;

    /// Total scrollable content size (`scrollWidth`/`scrollHeight`).
    fn scroll_size(&self) -> Read<Dimensions>;

    /// Current scroll offset in pixels (`scrollLeft`, `scrollTop`).
    fn scroll_offset(&self) -> Read<(f64, f64)>;

    /// An `<img>`'s intrinsic pixel size. [`PlatformError::NotFound`] until it
    /// decoded, and for a non-image.
    fn natural_size(&self) -> Read<Dimensions>;

    /// A CSS property's computed value in pixels, `None` when it is not a
    /// length (`auto`, `none`, a percentage the browser keeps as one).
    fn computed_px(&self, property: &str) -> Read<Option<f64>> {
        let _ = property;
        Box::pin(std::future::ready(Err(PlatformError::Unsupported)))
    }

    /// Sets this element's scroll offset, in pixels.
    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError>;

    /// Scrolls the nearest scroller vertically just enough to show this element,
    /// like `scrollIntoView` `nearest`: that call moves Chromium's Tab start.
    fn scroll_into_view(&self, smooth: bool) -> Result<(), PlatformError>;

    /// Replaces this `<input type="file">`'s file list, so a form posts what the
    /// caller holds. Web only, through a `DataTransfer`.
    fn set_files(&self, files: &[dioxus::html::FileData]) -> Result<(), PlatformError>;

    /// Sets a checkbox `<input>`'s `indeterminate` property, the only mixed
    /// state a browser exposes for it: HTML-AAM ignores `aria-checked` there.
    fn set_indeterminate(&self, indeterminate: bool) -> Result<(), PlatformError> {
        let _ = indeterminate;
        Err(PlatformError::Unsupported)
    }

    /// Writes a text control's current value, as typing would leave it. For a
    /// control whose value prop did not change, which dioxus never writes again.
    fn set_value(&self, value: &str) -> Result<(), PlatformError> {
        let _ = value;
        Err(PlatformError::Unsupported)
    }

    /// An attribute as the markup has it (not live state), `None` when absent.
    /// Synchronous: a key handler decides before it returns.
    fn attribute(&self, name: &str) -> Result<Option<String>, PlatformError> {
        let _ = name;
        Err(PlatformError::Unsupported)
    }

    /// Where a text control's selection starts (the caret), in UTF-16 units.
    /// `None` without a text selection or where the renderer cannot tell.
    fn selection_start(&self) -> Option<u32> {
        None
    }

    /// The last match of `selector` before this one in document order, subtree
    /// excluded: where Shift+Tab would land without it. `Ok(None)` for none.
    fn previous_focusable(
        &self,
        selector: &str,
    ) -> Result<Option<Box<dyn ElementApi>>, PlatformError> {
        let _ = selector;
        Err(PlatformError::Unsupported)
    }

    /// Routes further events from `pointer_id` here, so a drag keeps its
    /// `pointerup`. Released automatically.
    fn set_pointer_capture(&self, pointer_id: i32) -> Result<(), PlatformError>;

    /// First descendant matching `selector`.
    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError>;

    /// Every descendant matching `selector`, in DOM order.
    fn query_selector_all(&self, selector: &str)
    -> Result<Vec<Box<dyn ElementApi>>, PlatformError>;
}
