//! One module per renderer, plus [`mounted`], the element floor all share, and
//! [`thread`], the non-wasm timer.
//!
//! [`element`] downcasts `MountedData` to the renderer's own handle (a
//! `web_sys::Element`, a Blitz `NodeHandle`) and falls back to the floor.

use std::rc::Rc;

use dioxus::prelude::{
    Callback, Element, Event, FocusData, KeyboardData, MountedData, MouseData, PointerData,
    ResizeData, TransitionData,
};

use super::{
    A11yMediaApi, ColorSchemeApi, ContentSubscription, DocumentApi, ElementApi, KeyboardApi,
    MediaQueryApi, PressApi, ScrollApi, SilentFocusApi, TimerApi,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod blitz;
mod mounted;
#[cfg(not(target_arch = "wasm32"))]
mod origin;
#[cfg(all(target_os = "linux", feature = "native"))]
mod portal;
#[cfg(not(target_arch = "wasm32"))]
mod thread;
#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
mod webview;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::capture as webview_capture;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::clipboard as webview_clipboard;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::file_dialog as webview_file_dialog;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::geolocation as webview_geolocation;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::image_crop as webview_image_crop;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::media as webview_media;
#[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
pub(crate) use webview::permission as webview_permission;

/// What HTML counts as interactive content, plus anything a caller made
/// focusable. A label does not forward a click on any of these.
#[cfg(any(target_arch = "wasm32", feature = "native"))]
const INTERACTIVE: &str = "a[href], button, input, select, textarea, summary, \
    [tabindex], [contenteditable]:not([contenteditable=\"false\"])";

/// The richest [`ElementApi`] this renderer can back `mounted` with.
pub(crate) fn element(mounted: &Rc<MountedData>) -> Box<dyn ElementApi> {
    #[cfg(target_arch = "wasm32")]
    if let Some(element) = web::element(mounted) {
        return element;
    }

    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    if let Some(element) = blitz::element(mounted) {
        return element;
    }

    Box::new(mounted::MountedElement(mounted.clone()))
}

/// The web has a `MutationObserver`; Blitz watches the subtree's scroll size.
pub(crate) fn on_content_change(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    #[cfg(target_arch = "wasm32")]
    return web::on_content_change(mounted, callback);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::on_content_change(mounted, callback);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = (mounted, callback);
        None
    }
}

/// Only a WebView - see [`observes_by_tag`](crate::platform::observes_by_tag).
pub(crate) fn observes_by_tag() -> bool {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::runs_scripts();
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return false;
}

/// Only a WebView finds an element by its tag; see [`observes_by_tag`].
pub(crate) fn computed_px_by_tag(tag: u64, property: &str) -> super::Read<Option<f64>> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::computed_px(tag, property);
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    {
        let _ = (tag, property);
        Box::pin(std::future::ready(Err(super::PlatformError::Unsupported)))
    }
}

/// The web and a WebView have an `IntersectionObserver`; Blitz has none.
pub(crate) fn on_intersection(
    target: &Rc<MountedData>,
    root: Option<&Rc<MountedData>>,
    tags: (u64, Option<u64>),
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = tags;
        web::on_intersection(target, root, root_margin, thresholds, callback)
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = (target, root);
        webview::on_intersection(tags, root_margin, thresholds, callback)
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        let _ = (target, root, tags, root_margin, thresholds, callback);
        None
    }
}

/// Only Blitz: every other renderer fires `onresize` itself.
pub(crate) fn on_resize(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn(Event<ResizeData>)>,
) -> Option<Box<dyn ContentSubscription>> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::on_resize(mounted, callback);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    {
        let _ = (mounted, callback);
        None
    }
}

/// Only the web: Blitz fires no native `submit` or `reset` for a raw `<form>`,
/// so an owner found there would promise what nothing delivers.
pub(crate) fn on_form_reset(
    mounted: &Rc<MountedData>,
    on_reset: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (mounted, on_reset);
    #[cfg(target_arch = "wasm32")]
    return web::on_form_reset(mounted, on_reset);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// The web and Blitz compute `direction`; the WebView floor answers LTR.
pub(crate) fn is_rtl(mounted: &Rc<MountedData>) -> bool {
    #[cfg(not(any(target_arch = "wasm32", feature = "native")))]
    let _ = mounted;
    #[cfg(target_arch = "wasm32")]
    return web::is_rtl(mounted);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::is_rtl(mounted);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// A WebView answers `false`: it finds the focused element by its tags instead.
pub(crate) fn focus_is_in(mounted: &Rc<MountedData>) -> bool {
    #[cfg(not(any(target_arch = "wasm32", feature = "native")))]
    let _ = mounted;
    #[cfg(target_arch = "wasm32")]
    return web::focus_is_in(mounted);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::focus_is_in(mounted);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// A WebView answers `false`: it has no synchronous DOM.
pub(crate) fn element_contains(outer: &Rc<MountedData>, inner: &Rc<MountedData>) -> bool {
    #[cfg(not(any(target_arch = "wasm32", feature = "native")))]
    let _ = (outer, inner);
    #[cfg(target_arch = "wasm32")]
    return web::element_contains(outer, inner);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::element_contains(outer, inner);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return false;
}

/// What the renderer needs mounted at the root, once, by `LiberoProvider`:
/// Blitz's `blitz::Outlet`; a WebView starts its focus mirror before any press
/// and its typed-value guard before any keystroke.
#[allow(non_snake_case)]
pub(crate) fn Outlet() -> Element {
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::Outlet {} };
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    use_hook(|| {
        webview::watch_focus();
        webview::guard_typed_values();
        webview::guard_defaults();
    });
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return rsx! {};
}

/// Rendered next to the `<style>`s libero registers, `version` bumped with
/// each change. Only Blitz needs to know (see `blitz::SheetWatch`).
#[allow(non_snake_case)]
pub(crate) fn SheetWatch(version: u64) -> Element {
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::SheetWatch { version } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return {
        let _ = version;
        rsx! {}
    };
}

/// Wraps the app on a renderer that has to watch input from above it; only
/// Blitz does (see `blitz::Listener`). Everywhere else this is `children`.
#[allow(non_snake_case)]
pub(crate) fn Listener(children: Element) -> Element {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::Listener { {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return children;
}

/// The portal outlet's box; only Blitz places it (see `blitz::PortalRoot`).
#[allow(non_snake_case)]
pub(crate) fn PortalRoot(children: Element) -> Element {
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::PortalRoot { {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return rsx! { div { {children} } };
}

/// Wraps one portaled entry; only Blitz needs a box (see `blitz::PortalEntry`).
/// `idle`: mounted but drawing nothing.
#[allow(non_snake_case)]
pub(crate) fn PortalEntry(children: Element, idle: bool) -> Element {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    use dioxus::prelude::*;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return rsx! { blitz::PortalEntry { idle, {children} } };
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    {
        let _ = idle;
        children
    }
}

/// Blitz opens no picker for a `<select>`; see [`opens_select_picker`](crate::platform::opens_select_picker).
pub(crate) const OPENS_SELECT_PICKER: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz skips a `<caption>`; see [`lays_out_captions`](crate::platform::lays_out_captions).
pub(crate) const LAYS_OUT_CAPTIONS: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz holds a sized table at its width; see [`widens_sized_tables`](crate::platform::widens_sized_tables).
pub(crate) const WIDENS_SIZED_TABLES: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// No Blitz backend draws a `backdrop-filter`; see [`draws_backdrop_filter`](crate::platform::draws_backdrop_filter).
pub(crate) const DRAWS_BACKDROP_FILTER: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz clips `background-clip: text` to the box; see [`clips_background_to_text`](crate::platform::clips_background_to_text).
pub(crate) const CLIPS_BACKGROUND_TO_TEXT: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz draws every SVG `<img>` `contain`; see [`fits_svg_images`](crate::platform::fits_svg_images).
pub(crate) const FITS_SVG_IMAGES: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz's `NetHandler` has no failure path; see [`fires_image_errors`](crate::platform::fires_image_errors).
pub(crate) const FIRES_IMAGE_ERRORS: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz draws no `placeholder`; see [`draws_placeholders`](crate::platform::draws_placeholders).
pub(crate) const DRAWS_PLACEHOLDERS: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Only Blitz marks libero's own placeholders - see
/// [`placeholder_drawn`](crate::platform::placeholder_drawn).
pub(crate) fn placeholder_drawn() {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::placeholder_drawn();
}

/// Blitz paints only a text run's innermost element's background; see [`paints_outer_inline_backgrounds`](crate::platform::paints_outer_inline_backgrounds).
pub(crate) const PAINTS_OUTER_INLINE_BACKGROUNDS: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz aligns `start` left under `rtl`; see [`aligns_logical_text`](crate::platform::aligns_logical_text).
pub(crate) const ALIGNS_LOGICAL_TEXT: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz lays a legend out as a flex item; see [`lifts_legends`](crate::platform::lifts_legends).
pub(crate) const LIFTS_LEGENDS: bool = !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// Blitz paints raw form controls white; see [`colors_form_controls`](crate::platform::colors_form_controls).
pub(crate) const COLORS_FORM_CONTROLS: bool =
    !cfg!(all(not(target_arch = "wasm32"), feature = "native"));

pub(crate) fn document() -> Option<&'static dyn DocumentApi> {
    #[cfg(target_arch = "wasm32")]
    return web::document();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::document();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::document();
}

/// See [`color_scheme`](crate::platform::color_scheme).
pub(crate) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    #[cfg(target_arch = "wasm32")]
    return web::color_scheme();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::color_scheme();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::color_scheme();
}

/// The web and a WebView answer media queries; Blitz and a server get `None`.
pub(crate) fn media_query() -> Option<&'static dyn MediaQueryApi> {
    #[cfg(target_arch = "wasm32")]
    return web::media_query();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::media_query();
}

/// Cmd on Apple platforms, read from the browser's platform on the web.
pub(crate) fn mod_is_meta() -> bool {
    #[cfg(target_arch = "wasm32")]
    return web::mod_is_meta();
    #[cfg(not(target_arch = "wasm32"))]
    return cfg!(any(target_os = "macos", target_os = "ios"));
}

/// The web, Blitz and a WebView report a scroll; a server gets `None`.
pub(crate) fn scroll() -> Option<&'static dyn ScrollApi> {
    #[cfg(target_arch = "wasm32")]
    return web::scroll();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::scroll();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::scroll();
}

/// `scrollIntoView`'s `nearest`, vertically: how far a view spanning
/// `view_top..view_bottom` has to scroll to show `top..bottom`, `None` if it
/// already shows it. A box taller than the view aligns its top.
#[cfg_attr(not(any(target_arch = "wasm32", feature = "native")), allow(dead_code))]
fn nearest_scroll(top: f64, bottom: f64, view_top: f64, view_bottom: f64) -> Option<f64> {
    if top < view_top || bottom - top > view_bottom - view_top {
        Some(top - view_top)
    } else if bottom > view_bottom {
        Some(bottom - view_bottom)
    } else {
        None
    }
}

/// `setTimeout` on the web, [`thread`]'s sleeping thread on every other renderer.
pub(crate) fn timer() -> Option<&'static dyn TimerApi> {
    #[cfg(target_arch = "wasm32")]
    return web::timer();
    #[cfg(not(target_arch = "wasm32"))]
    return thread::timer();
}

/// The web listens on the window; Blitz hears a press bubble out of the app to
/// `blitz::Listener`, a WebView at the window over eval. A server has none.
pub(crate) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    #[cfg(target_arch = "wasm32")]
    return web::keyboard();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::keyboard();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::keyboard();
}

/// Only Blitz moves focus without a focus event - see
/// [`silent_focus`](crate::platform::silent_focus).
pub(crate) fn silent_focus() -> Option<&'static dyn SilentFocusApi> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::silent_focus();
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return None;
}

/// Only a WebView - see [`press`](crate::platform::press).
pub(crate) fn press() -> Option<&'static dyn PressApi> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::press();
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return None;
}

/// Only Blitz blurs for a press that cancelled its `mousedown` - see
/// [`blur_counts`](crate::platform::blur_counts).
pub(crate) fn press_kept_focus() -> bool {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::press_kept_focus();
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return false;
}

/// Only Blitz rewrites them - see [`focus_selectors`](crate::platform::focus_selectors).
pub(crate) fn focus_selectors(css: &str) -> std::borrow::Cow<'_, str> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::focus_selectors(css);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return std::borrow::Cow::Borrowed(css);
}

/// The web reads the `web_sys` event, a WebView its IPC payload; Blitz and a
/// server answer `None`. See [`transition_property`](crate::platform::transition_property).
pub(crate) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return web::transition_property(event);
    #[cfg(not(target_arch = "wasm32"))]
    return {
        use dioxus::html::{HasTransitionData, SerializedTransitionData};
        event
            .downcast::<SerializedTransitionData>()
            .map(HasTransitionData::property_name)
    };
}

/// Only the web can say - see [`focus_visible`](crate::platform::focus_visible).
pub(crate) fn focus_visible(event: &Event<FocusData>) -> Option<bool> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::focus_visible(event);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web has a document key listener that can take a press ahead of
/// the element handlers - see [`key_taken`](crate::platform::key_taken).
pub(crate) fn key_taken(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::key_taken(event);
    #[cfg(not(target_arch = "wasm32"))]
    return false;
}

/// The web reads the event's target; Blitz the focused node, where it sends a
/// key press, and a WebView its mirror of it - see [`typing_target`](crate::platform::typing_target).
pub(crate) fn typing_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::typing_target(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::typing_target();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::typing_target();
}

/// As [`typing_target`] - see [`arrow_target`](crate::platform::arrow_target).
pub(crate) fn arrow_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::arrow_target(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::arrow_target();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::arrow_target();
}

/// As [`typing_target`] - see [`caret_edges`](crate::platform::caret_edges).
pub(crate) fn caret_edges(event: &Event<KeyboardData>) -> Option<(bool, bool)> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::caret_edges(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::caret_edges();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::caret_edges();
}

/// As [`typing_target`] - see [`rtl_target`](crate::platform::rtl_target).
pub(crate) fn rtl_target(event: &Event<KeyboardData>) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::rtl_target(event);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::rtl_target();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::rtl_target();
}

/// Only a WebView - see [`set_value_by_id`](crate::platform::set_value_by_id).
pub(crate) fn set_value_by_id(id: &str, value: &str) -> Result<(), super::PlatformError> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::set_value_by_id(id, value);
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return {
        let _ = (id, value);
        Err(super::PlatformError::Unsupported)
    };
}

/// Only a WebView - see [`focus_selector`](crate::platform::focus_selector).
pub(crate) fn focus_selector(selector: &str) -> Result<(), super::PlatformError> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::focus_selector(selector);
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return {
        let _ = selector;
        Err(super::PlatformError::Unsupported)
    };
}

/// Only a WebView - see [`focus_among`](crate::platform::focus_among).
pub(crate) fn focus_among(
    attr: &str,
    values: &[String],
    to: super::FocusStep,
) -> Result<(), super::PlatformError> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::focus_among(attr, values, to);
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return {
        let _ = (attr, values, to);
        Err(super::PlatformError::Unsupported)
    };
}

/// Only a WebView - see [`focused_attribute`](crate::platform::focused_attribute).
pub(crate) fn focused_attribute(attr: &str) -> super::Read<Option<String>> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::focused_attribute(attr);
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return {
        let _ = attr;
        Box::pin(std::future::ready(Err(super::PlatformError::Unsupported)))
    };
}

/// Only a WebView - see [`keep_focused`](crate::platform::keep_focused).
pub(crate) fn keep_focused() -> Option<u64> {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::keep_focused();
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    return None;
}

/// Only a WebView - see [`focus_kept`](crate::platform::focus_kept).
pub(crate) fn focus_kept(token: u64) {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    webview::focus_kept(token);
    #[cfg(any(target_arch = "wasm32", feature = "native"))]
    let _ = token;
}

/// Only a WebView on a phone - see [`soft_keyboard_app`](crate::platform::soft_keyboard_app).
pub(crate) fn soft_keyboard_app() -> bool {
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(feature = "native"),
        any(target_os = "android", target_os = "ios")
    ))]
    return webview::runs_scripts();
    #[cfg(not(all(
        not(target_arch = "wasm32"),
        not(feature = "native"),
        any(target_os = "android", target_os = "ios")
    )))]
    return false;
}

/// The web reads the click's target, Blitz the press `blitz::Listener` hit -
/// see [`nested_interactive`](crate::platform::nested_interactive).
pub(crate) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = event;
    #[cfg(target_arch = "wasm32")]
    return web::nested_interactive(event, boundary);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::nested_interactive(boundary);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return {
        let _ = boundary;
        false
    };
}

/// Only Blitz needs it: its editor ignores `maxlength` - see
/// [`fit_max_length`](crate::platform::fit_max_length).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
pub(crate) fn fit_pasted(value: String) -> String {
    blitz::fit_pasted(value)
}

/// Blitz, which has no pointer capture, and a WebView, whose handles reach no
/// element - see [`follow_pointer`](crate::platform::follow_pointer).
pub(crate) fn follow_pointer(
    event: &Event<PointerData>,
    capture: &Rc<MountedData>,
    onmove: Callback<Event<PointerData>>,
    onup: Callback<Event<PointerData>>,
) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::follow_pointer(event, capture, onmove, onup);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = (capture, onmove, onup);
        webview::follow_pointer(event);
    }
    #[cfg(target_arch = "wasm32")]
    let _ = (event, capture, onmove, onup);
}

/// Only the web can see a click's target today - see
/// [`padding_press`](crate::platform::padding_press).
pub(crate) fn padding_press(
    event: &Event<MouseData>,
    boundary: &str,
) -> Option<Box<dyn ElementApi>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (event, boundary);
    #[cfg(target_arch = "wasm32")]
    return web::padding_press(event, boundary);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// Only the web can see a focus event's `relatedTarget` - see
/// [`focus_entered_from`](crate::platform::focus_entered_from).
pub(crate) fn focus_entered_from(
    event: &Event<FocusData>,
    boundary: &str,
) -> Option<Option<Box<dyn ElementApi>>> {
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (event, boundary);
    #[cfg(target_arch = "wasm32")]
    return web::focus_entered_from(event, boundary);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}

/// The web reads a press's target, Blitz hit-tests it - see
/// [`focus_pressed`](crate::platform::focus_pressed).
pub(crate) fn focus_pressed(event: &Event<PointerData>, within: &Rc<MountedData>) {
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    let _ = (event, within);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::focus_pressed(event, within);
    #[cfg(target_arch = "wasm32")]
    web::focus_pressed(event, within);
}

/// Only Blitz fires no `submit` - see
/// [`emulates_submit`](crate::platform::emulates_submit).
pub(crate) fn emulates_submit() -> bool {
    cfg!(all(not(target_arch = "wasm32"), feature = "native"))
}

pub(crate) fn activated_submitter(form: &Rc<MountedData>) -> bool {
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = form;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::activated_submitter(form);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return false;
}

pub(crate) fn implicit_submission(form: &Rc<MountedData>) -> bool {
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = form;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::implicit_submission(form);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return false;
}

pub(crate) fn form_values(form: &Rc<MountedData>) -> Vec<(String, dioxus::html::FormValue)> {
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    let _ = form;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return blitz::form_values(form);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    return Vec::new();
}

/// Only Blitz renders with its document borrowed - see
/// [`when_free`](crate::platform::when_free).
pub(crate) fn when_free(run: Box<dyn FnOnce()>) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::when_free(run);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    run();
}

/// Only Blitz lays out after effects run - see
/// [`when_laid_out`](crate::platform::when_laid_out).
pub(crate) fn when_laid_out(run: Box<dyn FnOnce()>) {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    blitz::when_laid_out(run);
    #[cfg(not(all(not(target_arch = "wasm32"), feature = "native")))]
    run();
}

/// Only Blitz's stylo lacks the accessibility media features - see
/// [`answers_a11y_media`](crate::platform::answers_a11y_media).
pub(crate) const ANSWERS_A11Y_MEDIA: bool =
    cfg!(all(not(target_arch = "wasm32"), feature = "native"));

/// The web's and a WebView's `matchMedia`, and the desktop portal for Blitz on Linux.
pub(crate) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    #[cfg(target_arch = "wasm32")]
    return web::a11y_media();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::a11y_media();
    #[cfg(all(target_os = "linux", feature = "native"))]
    return portal::a11y_media();
    #[cfg(all(not(target_os = "linux"), feature = "native"))]
    return None;
}

/// The web and a WebView read the media query - see
/// [`prefers_reduced_motion`](crate::platform::prefers_reduced_motion).
pub(crate) fn prefers_reduced_motion() -> bool {
    #[cfg(target_arch = "wasm32")]
    return web::prefers_reduced_motion();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return webview::prefers_reduced_motion();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return false;
}

#[cfg(test)]
mod tests {
    use super::nearest_scroll;

    #[test]
    fn nearest_scroll_moves_the_out_of_view_edge_only() {
        assert_eq!(nearest_scroll(20.0, 40.0, 0.0, 100.0), None);
        assert_eq!(nearest_scroll(120.0, 140.0, 0.0, 100.0), Some(40.0));
        assert_eq!(nearest_scroll(-30.0, -10.0, 0.0, 100.0), Some(-30.0));
        // Taller than the view: its top.
        assert_eq!(nearest_scroll(50.0, 250.0, 0.0, 100.0), Some(50.0));
    }
}
