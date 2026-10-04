use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::{
    Event, FocusData, Key, KeyboardData, Modifiers, MountedData, MouseData, PointerData,
    TransitionData,
};
use wasm_bindgen::{JsCast, JsValue};

use wasm_bindgen::prelude::Closure;

use super::INTERACTIVE;
use crate::platform::a11y_media::{
    A11yMediaSubscription, REDUCED_MOTION_STORAGE_KEY, kept_reduced_motion, parse_reduced_motion,
};
use crate::platform::{
    A11yMediaApi, ColorSchemeApi, ColorSchemeSubscription, ContentSubscription, Dimensions,
    DocumentApi, ElementApi, KeyChord, KeySubscription, KeyboardApi, MediaQueryApi,
    MediaQuerySubscription, PlatformError, Read, ScrollApi, ScrollSubscription, TimerApi,
    TimerSubscription,
    keyboard::{takes_arrows, takes_typing, warn_reserved_chord},
};
use crate::tokens::{
    AccessibilityPreferences, COLOR_SCHEME_STORAGE_KEY, ColorScheme, ColorSchemeSetting, Contrast,
};

/// dioxus-web backs a mounted element with the `web_sys::Element` itself, so
/// the whole trait is answerable - no id, no document lookup.
pub(super) fn element(mounted: &Rc<MountedData>) -> Option<Box<dyn ElementApi>> {
    let element = mounted.downcast::<web_sys::Element>()?.clone();
    Some(Box::new(WebElement { element }))
}

pub(super) fn is_rtl(mounted: &Rc<MountedData>) -> bool {
    mounted
        .downcast::<web_sys::Element>()
        .is_some_and(element_is_rtl)
}

/// Whether `document.activeElement` is `mounted` or inside it.
pub(super) fn focus_is_in(mounted: &Rc<MountedData>) -> bool {
    let active = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element());
    mounted
        .downcast::<web_sys::Element>()
        .is_some_and(|element| active.is_some_and(|active| element.contains(Some(&active))))
}

pub(super) fn element_contains(outer: &Rc<MountedData>, inner: &Rc<MountedData>) -> bool {
    let (Some(outer), Some(inner)) = (
        outer.downcast::<web_sys::Element>(),
        inner.downcast::<web_sys::Element>(),
    ) else {
        return false;
    };
    outer.contains(Some(inner))
}

/// The computed `direction`, for any node: an event's target as well.
pub(super) fn element_is_rtl(element: &web_sys::Element) -> bool {
    web_sys::window()
        .and_then(|window| window.get_computed_style(element).ok()?)
        .and_then(|style| style.get_property_value("direction").ok())
        .is_some_and(|direction| direction == "rtl")
}

/// What a component-driven control held before a form reset.
enum Controlled {
    Input(String, bool),
    Text(String),
    Select(i32),
    Other,
}

/// dioxus-web's `as_any` is the `web_sys` event itself, not a wrapper.
pub(super) fn transition_property(event: &Event<TransitionData>) -> Option<String> {
    Some(
        event
            .downcast::<web_sys::TransitionEvent>()?
            .property_name(),
    )
}

/// The native event outlives its dispatch, so `defaultPrevented` still reads
/// what the capture-phase `KeyboardApi` listener did to it.
pub(super) fn key_taken(event: &Event<KeyboardData>) -> bool {
    event
        .downcast::<web_sys::KeyboardEvent>()
        .is_some_and(|event| event.default_prevented())
}

pub(super) fn typing_target(event: &Event<KeyboardData>) -> bool {
    event
        .downcast::<web_sys::KeyboardEvent>()
        .is_some_and(editable_target)
}

pub(super) fn arrow_target(event: &Event<KeyboardData>) -> bool {
    event
        .downcast::<web_sys::KeyboardEvent>()
        .is_some_and(stepping_target)
}

pub(super) fn caret_edges(event: &Event<KeyboardData>) -> Option<(bool, bool)> {
    let input = event
        .downcast::<web_sys::KeyboardEvent>()
        .and_then(key_target)?
        .dyn_into::<web_sys::HtmlInputElement>()
        .ok()?;
    // `None` on a type without a selection, such as `number`.
    let start = input.selection_start().ok()??;
    let end = input.selection_end().ok()??;
    let length = u32::try_from(input.value().encode_utf16().count()).ok()?;
    Some((start == end && start == 0, start == end && end == length))
}

pub(super) fn rtl_target(event: &Event<KeyboardData>) -> bool {
    event
        .downcast::<web_sys::KeyboardEvent>()
        .and_then(key_target)
        .is_some_and(|target| element_is_rtl(&target))
}

/// Nested when the target's nearest interactive element sits inside `boundary`.
/// dioxus-web delegates from the root, so a selector stands in for `currentTarget`.
pub(super) fn nested_interactive(event: &Event<MouseData>, boundary: &str) -> bool {
    let nested = || -> Option<bool> {
        let target = event
            .downcast::<web_sys::MouseEvent>()?
            .target()?
            .dyn_into::<web_sys::Element>()
            .ok()?;
        let boundary = target.closest(boundary).ok()??;
        let hit = target.closest(INTERACTIVE).ok()??;
        Some(hit != boundary && boundary.contains(Some(&hit)))
    };
    nested().unwrap_or(false)
}

pub(super) fn padding_press(
    event: &Event<MouseData>,
    boundary: &str,
) -> Option<Box<dyn ElementApi>> {
    let target = event
        .downcast::<web_sys::MouseEvent>()?
        .target()?
        .dyn_into::<web_sys::Element>()
        .ok()?;
    let frame = target.closest(boundary).ok()??;
    if let Some(hit) = target.closest(INTERACTIVE).ok()?
        && hit != frame
        && frame.contains(Some(&hit))
    {
        return None;
    }
    let is_control = |child: &web_sys::Element| {
        child
            .get_attribute("data-slot")
            .is_none_or(|slot| slot == "control")
            && !child.has_attribute("data-ring")
    };
    // The frame's own child the press is in, if any: the control's is its own.
    let mut at = target;
    while at != frame {
        let parent = at.parent_element()?;
        if parent == frame && is_control(&at) {
            return None;
        }
        at = parent;
    }
    let control = std::iter::successors(frame.first_element_child(), |child| {
        child.next_element_sibling()
    })
    .find(is_control)?;
    let stops = control.query_selector_all(INTERACTIVE).ok()?;
    let tab_stop = |element: web_sys::Element| {
        element
            .dyn_into::<web_sys::HtmlElement>()
            .ok()
            .filter(|element| element.tab_index() >= 0)
    };
    let element = tab_stop(control.clone()).or_else(|| {
        (0..stops.length())
            .filter_map(|index| stops.item(index)?.dyn_into::<web_sys::Element>().ok())
            .find_map(tab_stop)
    })?;
    Some(Box::new(WebElement {
        element: element.into(),
    }))
}

pub(super) fn focus_visible(event: &Event<FocusData>) -> Option<bool> {
    let target = event.downcast::<web_sys::FocusEvent>()?.target()?;
    let element = target.dyn_into::<web_sys::Element>().ok()?;
    element.matches(":focus-visible").ok()
}

pub(super) fn focus_entered_from(
    event: &Event<FocusData>,
    boundary: &str,
) -> Option<Option<Box<dyn ElementApi>>> {
    let from = event.downcast::<web_sys::FocusEvent>()?.related_target();
    let Some(element) = from.and_then(|from| from.dyn_into::<web_sys::Element>().ok()) else {
        return Some(None);
    };
    if element.closest(boundary).ok().flatten().is_some() {
        return None;
    }
    Some(Some(Box::new(WebElement { element })))
}

/// A `focusout` to nothing (`<body>`) left it too.
pub(super) fn focus_left(event: &Event<FocusData>, element: &Rc<MountedData>) -> Option<bool> {
    let element = element.downcast::<web_sys::Element>()?;
    let to = event.downcast::<web_sys::FocusEvent>()?.related_target();
    let to = to.and_then(|to| to.dyn_into::<web_sys::Node>().ok());
    Some(!to.is_some_and(|to| element.contains(Some(&to))))
}

/// Walks up from the target because dioxus-web delegates from the root, so
/// `currentTarget` is not the handle.
pub(super) fn focus_pressed(event: &Event<PointerData>, within: &Rc<MountedData>) {
    let focus = || -> Option<()> {
        let within = within.downcast::<web_sys::Element>()?;
        let target = match event.downcast::<web_sys::PointerEvent>() {
            Some(event) => event.target(),
            None => event.downcast::<web_sys::MouseEvent>()?.target(),
        };
        let mut at = target?.dyn_into::<web_sys::Element>().ok()?;
        let mut handle = None;
        loop {
            if let Some(element) = at.dyn_ref::<web_sys::HtmlElement>()
                && element.tab_index() >= 0
            {
                handle = Some(element.clone());
            }
            if at == *within {
                break;
            }
            at = at.parent_element()?;
        }
        let handle = handle?;
        let active = web_sys::window()?.document()?.active_element();
        if active.is_some_and(|active| handle.contains(Some(&active))) {
            return None;
        }
        let options = web_sys::FocusOptions::new();
        options.set_prevent_scroll(true);
        handle.focus_with_options(&options).ok()
    };
    let _ = focus();
}

pub(super) fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media(crate::sx::REDUCED_MOTION).ok()?)
        .is_some_and(|query| query.matches())
}

pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    Some(&DOCUMENT)
}

pub(super) fn root_padding_right() -> Option<f64> {
    let window = web_sys::window()?;
    let root = window.document()?.document_element()?;
    let value = window
        .get_computed_style(&root)
        .ok()??
        .get_property_value("padding-right")
        .ok()?;
    value.strip_suffix("px")?.trim().parse().ok()
}

pub(super) fn a11y_media() -> Option<&'static dyn A11yMediaApi> {
    Some(&A11Y_MEDIA)
}

/// The browser's own answers, read back for `use_accessibility`.
struct WebA11yMedia;

static A11Y_MEDIA: WebA11yMedia = WebA11yMedia;

const A11Y_QUERIES: [&str; 5] = [
    crate::sx::REDUCED_MOTION,
    crate::sx::FORCED_COLORS,
    "(prefers-contrast: more)",
    "(prefers-contrast: less)",
    "(prefers-reduced-transparency: reduce)",
];

fn a11y_queries() -> Vec<Option<web_sys::MediaQueryList>> {
    let window = web_sys::window();
    A11Y_QUERIES
        .iter()
        .map(|query| window.as_ref()?.match_media(query).ok()?)
        .collect()
}

impl A11yMediaApi for WebA11yMedia {
    fn system(&self) -> AccessibilityPreferences {
        let matches: Vec<bool> = a11y_queries()
            .iter()
            .map(|query| query.as_ref().is_some_and(|query| query.matches()))
            .collect();
        AccessibilityPreferences {
            reduced_motion: matches[0],
            forced_colors: matches[1],
            contrast: match (matches[2], matches[3]) {
                (true, _) => Contrast::More,
                (_, true) => Contrast::Less,
                _ => Contrast::NoPreference,
            },
            reduced_transparency: matches[4],
        }
    }

    fn on_change(
        &self,
        callback: Box<dyn Fn(AccessibilityPreferences)>,
    ) -> Box<dyn A11yMediaSubscription> {
        let closure = Closure::<dyn FnMut()>::new(move || callback(A11Y_MEDIA.system()));
        let queries: Vec<_> = a11y_queries()
            .into_iter()
            .flatten()
            .filter(|query| {
                query
                    .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                    .is_ok()
            })
            .collect();
        Box::new(WebA11yMediaSubscription { queries, closure })
    }

    fn stored_reduced_motion(&self) -> Option<bool> {
        parse_reduced_motion(
            &local_storage()?
                .get_item(REDUCED_MOTION_STORAGE_KEY)
                .ok()??,
        )
    }

    fn store_reduced_motion(&self, reduced: Option<bool>) {
        let Some(storage) = local_storage() else {
            return;
        };
        let _ = match reduced {
            Some(reduced) => {
                storage.set_item(REDUCED_MOTION_STORAGE_KEY, kept_reduced_motion(reduced))
            }
            None => storage.remove_item(REDUCED_MOTION_STORAGE_KEY),
        };
    }
}

struct WebA11yMediaSubscription {
    queries: Vec<web_sys::MediaQueryList>,
    closure: Closure<dyn FnMut()>,
}

impl A11yMediaSubscription for WebA11yMediaSubscription {}

impl Drop for WebA11yMediaSubscription {
    fn drop(&mut self) {
        for query in &self.queries {
            let _ = query.remove_event_listener_with_callback(
                "change",
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

pub(super) fn color_scheme() -> Option<&'static dyn ColorSchemeApi> {
    Some(&COLOR_SCHEME)
}

struct WebColorScheme;

static COLOR_SCHEME: WebColorScheme = WebColorScheme;

fn dark_query() -> Option<web_sys::MediaQueryList> {
    web_sys::window()
        .and_then(|window| window.match_media(crate::theme::DARK_SCHEME_QUERY).ok())
        .flatten()
}

impl ColorSchemeApi for WebColorScheme {
    fn system(&self) -> ColorScheme {
        match dark_query().is_some_and(|query| query.matches()) {
            true => ColorScheme::Dark,
            false => ColorScheme::Light,
        }
    }

    fn on_change(&self, callback: Box<dyn Fn(ColorScheme)>) -> Box<dyn ColorSchemeSubscription> {
        // Read off the query, so it cannot disagree with `system()`.
        let closure = Closure::<dyn FnMut()>::new(move || {
            let dark = dark_query().is_some_and(|query| query.matches());
            callback(match dark {
                true => ColorScheme::Dark,
                false => ColorScheme::Light,
            });
        });

        let query = dark_query().filter(|query| {
            query
                .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .is_ok()
        });

        Box::new(WebColorSchemeSubscription { query, closure })
    }

    fn stored(&self) -> Option<ColorSchemeSetting> {
        let stored = local_storage()?.get_item(COLOR_SCHEME_STORAGE_KEY).ok()??;
        Some(ColorSchemeSetting::parse(&stored))
    }

    fn store(&self, setting: ColorSchemeSetting) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(COLOR_SCHEME_STORAGE_KEY, setting.as_str());
        }
    }
}

/// `None` where the browser refuses the store: blocked site data throws.
fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

struct WebColorSchemeSubscription {
    /// `None` without a query: the subscription never fires.
    query: Option<web_sys::MediaQueryList>,
    /// Kept alive as long as the listener is registered.
    closure: Closure<dyn FnMut()>,
}

impl ColorSchemeSubscription for WebColorSchemeSubscription {}

impl Drop for WebColorSchemeSubscription {
    fn drop(&mut self) {
        if let Some(query) = &self.query {
            let _ = query.remove_event_listener_with_callback(
                "change",
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

pub(super) fn media_query() -> Option<&'static dyn MediaQueryApi> {
    Some(&MEDIA_QUERY)
}

struct WebMediaQuery;

static MEDIA_QUERY: WebMediaQuery = WebMediaQuery;

impl MediaQueryApi for WebMediaQuery {
    fn watch(&self, query: &str, callback: Box<dyn Fn(bool)>) -> Box<dyn MediaQuerySubscription> {
        let list = web_sys::window()
            .and_then(|window| window.match_media(query).ok())
            .flatten();
        let callback = Rc::new(callback);
        let source = list.clone();
        let heard = callback.clone();
        // Read off the list, so it cannot disagree with the first answer.
        let closure = Closure::<dyn FnMut()>::new(move || {
            if let Some(list) = &source {
                heard(list.matches());
            }
        });
        let list = list.filter(|list| {
            list.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .is_ok()
        });
        if let Some(list) = &list {
            callback(list.matches());
        }
        Box::new(WebMediaQuerySubscription { list, closure })
    }
}

struct WebMediaQuerySubscription {
    /// `None` without a list: the subscription never fires.
    list: Option<web_sys::MediaQueryList>,
    /// Kept alive as long as the listener is registered.
    closure: Closure<dyn FnMut()>,
}

impl MediaQuerySubscription for WebMediaQuerySubscription {}

impl Drop for WebMediaQuerySubscription {
    fn drop(&mut self) {
        if let Some(list) = &self.list {
            let _ = list.remove_event_listener_with_callback(
                "change",
                self.closure.as_ref().unchecked_ref(),
            );
        }
    }
}

/// Cmd on a Mac, an iPhone and an iPad, read from `navigator.platform`.
pub(super) fn mod_is_meta() -> bool {
    let platform = web_sys::window()
        .and_then(|window| window.navigator().platform().ok())
        .unwrap_or_default();
    ["Mac", "iPhone", "iPad", "iPod"]
        .iter()
        .any(|apple| platform.starts_with(apple))
}

struct WebDocument;

static DOCUMENT: WebDocument = WebDocument;

impl DocumentApi for WebDocument {
    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        let element = web_sys::window()?.document()?.active_element()?;
        Some(Box::new(WebElement { element }))
    }

    /// `inner_width`/`inner_height`: scrollbars included, the box a fixed
    /// element is laid out in. The height stops at the visual viewport's bottom.
    fn viewport(&self) -> Read<Dimensions> {
        let size = web_sys::window()
            .and_then(|window| {
                let width = window.inner_width().ok()?.as_f64()?;
                let height = window.inner_height().ok()?.as_f64()?;
                let height = visible_bottom(&window).map_or(height, |bottom| height.min(bottom));
                Some(Dimensions { width, height })
            })
            .ok_or(PlatformError::Unsupported);
        Box::pin(std::future::ready(size))
    }

    fn set_root_attribute(&self, name: &str, value: Option<&str>) -> bool {
        let Some(root) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
        else {
            return false;
        };

        match value {
            Some(value) => root.set_attribute(name, value).is_ok(),
            None => root.remove_attribute(name).is_ok(),
        }
    }
}

pub(super) fn scroll() -> Option<&'static dyn ScrollApi> {
    Some(&SCROLL)
}

struct WebScroll;

static SCROLL: WebScroll = WebScroll;

impl ScrollApi for WebScroll {
    /// In the capture phase, on the window: `scroll` does not bubble, so only
    /// capture sees an element (a `ScrollArea`) scroll.
    fn on_scroll(&self, callback: Box<dyn Fn()>) -> Box<dyn ScrollSubscription> {
        let closure = Closure::<dyn FnMut()>::new(callback);
        let target = web_sys::window().and_then(|window| {
            let target: web_sys::EventTarget = window.into();
            target
                .add_event_listener_with_callback_and_bool(
                    "scroll",
                    closure.as_ref().unchecked_ref(),
                    true,
                )
                .ok()?;
            Some(target)
        });

        Box::new(WebScrollSubscription { target, closure })
    }
}

struct WebScrollSubscription {
    /// `None` without a window: the subscription never fires.
    target: Option<web_sys::EventTarget>,
    /// Kept alive as long as the listener is registered: dropping it frees the
    /// JS function the listener points at.
    closure: Closure<dyn FnMut()>,
}

fn visual_viewport(window: &web_sys::Window) -> Option<JsValue> {
    js_sys::Reflect::get(window, &JsValue::from_str("visualViewport"))
        .ok()
        .filter(|viewport| viewport.is_object())
}

fn visual_read(window: &web_sys::Window, name: &str) -> Option<f64> {
    js_sys::Reflect::get(&visual_viewport(window)?, &JsValue::from_str(name))
        .ok()?
        .as_f64()
}

/// The visual viewport's bottom in layout coordinates: a keyboard in a browser tab
/// shrinks only the visual viewport.
fn visible_bottom(window: &web_sys::Window) -> Option<f64> {
    Some(visual_read(window, "offsetTop")? + visual_read(window, "height")?)
}

/// The visual viewport's top in layout coordinates: mobile Chrome pans it down to keep a
/// focused field visible.
pub(super) fn visible_top() -> f64 {
    web_sys::window()
        .and_then(|window| visual_read(&window, "offsetTop"))
        .map_or(0.0, |top| top.max(0.0))
}

pub(super) fn on_viewport_resize(callback: Box<dyn Fn()>) -> Option<Box<dyn ScrollSubscription>> {
    let window = web_sys::window()?;
    let closure = Closure::<dyn FnMut()>::new(callback);
    let mut listened = Vec::new();
    let visual: Option<web_sys::EventTarget> =
        visual_viewport(&window).and_then(|viewport| viewport.dyn_into().ok());
    let targets = [
        (Some(window.into()), "resize"),
        (visual.clone(), "resize"),
        (visual, "scroll"),
    ];
    for (target, event) in targets {
        if let Some(target) = target
            && target
                .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref())
                .is_ok()
        {
            listened.push((target, event));
        }
    }
    Some(Box::new(WebResizeSubscription { listened, closure }))
}

struct WebResizeSubscription {
    listened: Vec<(web_sys::EventTarget, &'static str)>,
    closure: Closure<dyn FnMut()>,
}

impl ScrollSubscription for WebResizeSubscription {}

impl Drop for WebResizeSubscription {
    fn drop(&mut self) {
        for (target, event) in &self.listened {
            let _ = target
                .remove_event_listener_with_callback(event, self.closure.as_ref().unchecked_ref());
        }
    }
}

pub(super) fn hold_edge_pan(
    band: crate::platform::scroll::EdgeBand,
    swiped: crate::platform::scroll::LostSwipe,
) -> Option<Box<dyn ScrollSubscription>> {
    let install =
        js_sys::Function::new_no_args(&format!("return {};", crate::platform::scroll::EDGE_PAN_JS))
            .call0(&JsValue::NULL)
            .ok()?
            .dyn_into::<js_sys::Function>()
            .ok()?;
    let closure = Closure::<dyn Fn(f64, f64, f64, f64)>::new(move |x, y, dx, dy| {
        swiped([x, y, dx, dy]);
    });
    let args = js_sys::Array::of5(
        &JsValue::from_f64(band.inset),
        &JsValue::from_f64(band.width),
        &JsValue::from_bool(band.left),
        &JsValue::from_f64(band.distance),
        closure.as_ref(),
    );
    let remove = install
        .apply(&JsValue::NULL, &args)
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    Some(Box::new(WebEdgePan {
        remove,
        _closure: closure,
    }))
}

struct WebEdgePan {
    remove: js_sys::Function,
    _closure: Closure<dyn Fn(f64, f64, f64, f64)>,
}

impl ScrollSubscription for WebEdgePan {}

impl Drop for WebEdgePan {
    fn drop(&mut self) {
        let _ = self.remove.call0(&JsValue::NULL);
    }
}

/// Attributes that make a node focusable or not, or reshape it through a
/// class. Not `style`: a `Virtualize` rewrites the observed wrapper's on scroll.
const CONTENT_ATTRIBUTES: [&str; 7] = [
    "tabindex",
    "disabled",
    "href",
    "contenteditable",
    "hidden",
    "open",
    "class",
];

/// The observer batches a render's records into one callback.
pub(super) fn on_content_change(
    mounted: &Rc<MountedData>,
    callback: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    let element = mounted.downcast::<web_sys::Element>()?;
    let closure = Closure::<dyn FnMut()>::new(callback);
    let observer = web_sys::MutationObserver::new(closure.as_ref().unchecked_ref()).ok()?;
    let options = web_sys::MutationObserverInit::new();
    options.set_child_list(true);
    options.set_subtree(true);
    options.set_character_data(true);
    let filter: js_sys::Array = CONTENT_ATTRIBUTES
        .iter()
        .copied()
        .map(JsValue::from)
        .collect();
    options.set_attribute_filter(&filter);
    observer.observe_with_options(element, &options).ok()?;
    Some(Box::new(WebContentSubscription {
        observer,
        _closure: closure,
    }))
}

struct WebContentSubscription {
    observer: web_sys::MutationObserver,
    /// Kept alive for as long as the observer can call it.
    _closure: Closure<dyn FnMut()>,
}

/// The newest of a batch's entries is the element's current state.
pub(super) fn on_intersection(
    target: &Rc<MountedData>,
    root: Option<&Rc<MountedData>>,
    root_margin: &str,
    thresholds: &[f64],
    callback: Box<dyn Fn(bool, f64)>,
) -> Option<Box<dyn ContentSubscription>> {
    let element = target.downcast::<web_sys::Element>()?;
    let closure = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
        let latest = entries.pop();
        if let Ok(entry) = latest.dyn_into::<web_sys::IntersectionObserverEntry>() {
            callback(entry.is_intersecting(), entry.intersection_ratio());
        }
    });
    let options = web_sys::IntersectionObserverInit::new();
    if let Some(root) = root {
        options.set_root(Some(root.downcast::<web_sys::Element>()?));
    }
    options.set_root_margin(root_margin);
    let thresholds: js_sys::Array = thresholds.iter().copied().map(JsValue::from).collect();
    options.set_threshold(&thresholds);
    // Throws where the browser has no `IntersectionObserver`.
    let observer =
        web_sys::IntersectionObserver::new_with_options(closure.as_ref().unchecked_ref(), &options)
            .ok()?;
    observer.observe(element);
    Some(Box::new(WebIntersectionSubscription {
        observer,
        _closure: closure,
    }))
}

struct WebIntersectionSubscription {
    observer: web_sys::IntersectionObserver,
    /// Kept alive for as long as the observer can call it.
    _closure: Closure<dyn FnMut(js_sys::Array)>,
}

impl ContentSubscription for WebIntersectionSubscription {}

impl Drop for WebIntersectionSubscription {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

pub(super) fn on_form_reset(
    mounted: &Rc<MountedData>,
    on_reset: Box<dyn Fn()>,
) -> Option<Box<dyn ContentSubscription>> {
    let form = form_owner(mounted.downcast::<web_sys::Element>()?)?;
    let closure = Closure::<dyn FnMut()>::new(on_reset);
    form.add_event_listener_with_callback("reset", closure.as_ref().unchecked_ref())
        .ok()?;
    Some(Box::new(WebResetSubscription { form, closure }))
}

/// A control's `form`, which honours its `form` attribute; for anything else
/// the nearest `<form>` around it.
fn form_owner(element: &web_sys::Element) -> Option<web_sys::HtmlFormElement> {
    if let Some(input) = element.dyn_ref::<web_sys::HtmlInputElement>() {
        return input.form();
    }
    if let Some(textarea) = element.dyn_ref::<web_sys::HtmlTextAreaElement>() {
        return textarea.form();
    }
    element.closest("form").ok()??.dyn_into().ok()
}

struct WebResetSubscription {
    form: web_sys::HtmlFormElement,
    /// Kept alive for as long as the listener is registered.
    closure: Closure<dyn FnMut()>,
}

impl ContentSubscription for WebResetSubscription {}

impl Drop for WebResetSubscription {
    fn drop(&mut self) {
        let _ = self
            .form
            .remove_event_listener_with_callback("reset", self.closure.as_ref().unchecked_ref());
    }
}

impl ContentSubscription for WebContentSubscription {}

impl Drop for WebContentSubscription {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

impl ScrollSubscription for WebScrollSubscription {}

impl Drop for WebScrollSubscription {
    fn drop(&mut self) {
        if let Some(target) = &self.target {
            // The same three arguments as added, capture included.
            let _ = target.remove_event_listener_with_callback_and_bool(
                "scroll",
                self.closure.as_ref().unchecked_ref(),
                true,
            );
        }
    }
}

pub(super) struct WebElement {
    element: web_sys::Element,
}

impl WebElement {
    /// Focus and clicks are `HtmlElement`'s, not every element's - an `<svg>`
    /// matched by a selector is an `Element` and has neither.
    fn html(&self) -> Result<&web_sys::HtmlElement, PlatformError> {
        self.element
            .dyn_ref::<web_sys::HtmlElement>()
            .ok_or(PlatformError::NotFound)
    }

    fn form(&self) -> Result<&web_sys::HtmlFormElement, PlatformError> {
        self.element
            .dyn_ref::<web_sys::HtmlFormElement>()
            .ok_or(PlatformError::NotFound)
    }
}

fn resolved<T: 'static>(value: T) -> Read<T> {
    Box::pin(std::future::ready(Ok(value)))
}

impl ElementApi for WebElement {
    fn focus(&self) -> Result<(), PlatformError> {
        self.html()?.focus().map_err(|_| PlatformError::NotFound)
    }

    fn blur(&self) -> Result<(), PlatformError> {
        self.html()?.blur().map_err(|_| PlatformError::NotFound)
    }

    fn click(&self) -> Result<(), PlatformError> {
        self.html()?.click();
        Ok(())
    }

    /// The native reset, then each `data-controlled` control gets its value back:
    /// dioxus never rewrites an unchanged value, so it would show its default.
    fn reset(&self) -> Result<(), PlatformError> {
        let form = self.form()?;
        let marked = form
            .query_selector_all("[data-controlled]")
            .map_err(|_| PlatformError::NotFound)?;
        let saved: Vec<_> = (0..marked.length())
            .filter_map(|index| marked.item(index))
            .map(|node| {
                let state = if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                    Controlled::Input(input.value(), input.checked())
                } else if let Some(text) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                    Controlled::Text(text.value())
                } else if let Some(select) = node.dyn_ref::<web_sys::HtmlSelectElement>() {
                    Controlled::Select(select.selected_index())
                } else {
                    Controlled::Other
                };
                (node, state)
            })
            .collect();
        form.reset();
        for (node, state) in saved {
            match state {
                Controlled::Input(value, checked) => {
                    if let Some(input) = node.dyn_ref::<web_sys::HtmlInputElement>() {
                        input.set_value(&value);
                        input.set_checked(checked);
                    }
                }
                Controlled::Text(value) => {
                    if let Some(text) = node.dyn_ref::<web_sys::HtmlTextAreaElement>() {
                        text.set_value(&value);
                    }
                }
                Controlled::Select(index) => {
                    if let Some(select) = node.dyn_ref::<web_sys::HtmlSelectElement>() {
                        select.set_selected_index(index);
                    }
                }
                Controlled::Other => {}
            }
        }
        Ok(())
    }

    fn request_submit(&self) -> Result<(), PlatformError> {
        self.form()?
            .request_submit()
            .map_err(|_| PlatformError::NotFound)
    }

    /// Rebuilt through a `DataTransfer`, a `FileList`'s only constructor. Each
    /// `FileData` still carries its `web_sys::File`.
    fn set_files(&self, files: &[dioxus::html::FileData]) -> Result<(), PlatformError> {
        let input = self
            .element
            .dyn_ref::<web_sys::HtmlInputElement>()
            .ok_or(PlatformError::NotFound)?;
        let transfer = web_sys::DataTransfer::new().map_err(|_| PlatformError::Unsupported)?;
        for file in files {
            let file = file
                .inner()
                .downcast_ref::<web_sys::File>()
                .ok_or(PlatformError::Unsupported)?;
            transfer
                .items()
                .add_with_file(file)
                .map_err(|_| PlatformError::Unsupported)?;
        }
        input.set_files(Some(&transfer.files().ok_or(PlatformError::Unsupported)?));
        Ok(())
    }

    fn set_indeterminate(&self, indeterminate: bool) -> Result<(), PlatformError> {
        self.element
            .dyn_ref::<web_sys::HtmlInputElement>()
            .ok_or(PlatformError::NotFound)?
            .set_indeterminate(indeterminate);
        Ok(())
    }

    fn set_value(&self, value: &str) -> Result<(), PlatformError> {
        if let Some(input) = self.element.dyn_ref::<web_sys::HtmlInputElement>() {
            input.set_value(value);
        } else if let Some(text) = self.element.dyn_ref::<web_sys::HtmlTextAreaElement>() {
            text.set_value(value);
        } else {
            return Err(PlatformError::NotFound);
        }
        Ok(())
    }

    fn attribute(&self, name: &str) -> Result<Option<String>, PlatformError> {
        Ok(self.element.get_attribute(name))
    }

    fn selection_start(&self) -> Option<u32> {
        if let Some(input) = self.element.dyn_ref::<web_sys::HtmlInputElement>() {
            input.selection_start().ok().flatten()
        } else {
            let text = self.element.dyn_ref::<web_sys::HtmlTextAreaElement>()?;
            text.selection_start().ok().flatten()
        }
    }

    /// The document's matches, last first, down to one this element follows:
    /// `FOLLOWING` holds for an ancestor too, and not for a descendant.
    fn previous_focusable(
        &self,
        selector: &str,
    ) -> Result<Option<Box<dyn ElementApi>>, PlatformError> {
        let nodes = web_sys::window()
            .and_then(|window| window.document())
            .ok_or(PlatformError::NotFound)?
            .query_selector_all(selector)
            .map_err(|_| PlatformError::NotFound)?;
        let element = (0..nodes.length())
            .rev()
            .filter_map(|index| nodes.get(index)?.dyn_into::<web_sys::Element>().ok())
            .find(|node| {
                node.compare_document_position(&self.element)
                    & web_sys::Node::DOCUMENT_POSITION_FOLLOWING
                    != 0
            });
        Ok(element.map(|element| Box::new(WebElement { element }) as Box<dyn ElementApi>))
    }

    fn is_focused(&self) -> bool {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .is_some_and(|active| JsValue::from(active) == JsValue::from(self.element.clone()))
    }

    fn is_connected(&self) -> bool {
        self.element.is_connected()
    }

    // The DOM answers all four at once; futures only for the WebView.
    fn dimensions(&self) -> Read<Dimensions> {
        let rect = self.element.get_bounding_client_rect();
        resolved(Dimensions {
            width: rect.width(),
            height: rect.height(),
        })
    }

    fn client_offset(&self) -> Read<(f64, f64)> {
        let rect = self.element.get_bounding_client_rect();
        resolved((rect.left(), rect.top()))
    }

    fn scroll_size(&self) -> Read<Dimensions> {
        resolved(Dimensions {
            width: self.element.scroll_width() as f64,
            height: self.element.scroll_height() as f64,
        })
    }

    fn scroll_offset(&self) -> Read<(f64, f64)> {
        resolved((
            self.element.scroll_left() as f64,
            self.element.scroll_top() as f64,
        ))
    }

    /// Zero until the picture has decoded, which is reported as not there.
    fn natural_size(&self) -> Read<Dimensions> {
        let size = self
            .element
            .dyn_ref::<web_sys::HtmlImageElement>()
            .map(|image| Dimensions {
                width: image.natural_width() as f64,
                height: image.natural_height() as f64,
            })
            .filter(|size| size.width > 0.0 && size.height > 0.0);
        Box::pin(std::future::ready(size.ok_or(PlatformError::NotFound)))
    }

    /// Viewport units and `rem` compute to pixels; `auto`, `none` and a
    /// percentage do not, and answer `None`.
    fn computed_px(&self, property: &str) -> Read<Option<f64>> {
        let Some(style) = web_sys::window()
            .and_then(|window| window.get_computed_style(&self.element).ok().flatten())
        else {
            return Box::pin(std::future::ready(Err(PlatformError::NotFound)));
        };
        let value = style.get_property_value(property).unwrap_or_default();
        resolved(
            value
                .strip_suffix("px")
                .and_then(|number| number.trim().parse::<f64>().ok()),
        )
    }

    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        // One call: under `scroll-behavior: smooth` a second write replaces
        // the first scroll, so nothing moves.
        self.element.scroll_to_with_x_and_y(x, y);
        Ok(())
    }

    fn scroll_into_view(&self, smooth: bool) -> Result<(), PlatformError> {
        let window = web_sys::window().ok_or(PlatformError::NotFound)?;
        let style = |element: &web_sys::Element, property: &str| {
            window
                .get_computed_style(element)
                .ok()
                .flatten()
                .and_then(|style| style.get_property_value(property).ok())
                .unwrap_or_default()
        };
        let rect = self.element.get_bounding_client_rect();
        // Not rendered: a zero rect would scroll an ancestor to its top.
        if rect.width() == 0.0 && rect.height() == 0.0 {
            return Ok(());
        }
        let page = window
            .document()
            .and_then(|document| document.scrolling_element());
        let mut ancestor = self.element.parent_element();
        let scroller = loop {
            // No inner scroller: the page itself, as the browser's `scrollIntoView` does.
            let Some(element) = ancestor else {
                match page.clone() {
                    Some(page) => break page,
                    None => return Ok(()),
                }
            };
            if element.scroll_height() > element.client_height()
                && matches!(style(&element, "overflow-y").as_str(), "auto" | "scroll")
            {
                break element;
            }
            ancestor = element.parent_element();
        };
        let margin = |side: &str| {
            style(&self.element, &format!("scroll-margin-{side}"))
                .trim_end_matches("px")
                .parse::<f64>()
                .unwrap_or(0.0)
        };
        let top = rect.top() - margin("top");
        let bottom = rect.bottom() + margin("bottom");
        // The page's own rect scrolls with it; its view is the viewport.
        let view_top = match page.as_ref() == Some(&scroller) {
            true => 0.0,
            false => scroller.get_bounding_client_rect().top() + scroller.client_top() as f64,
        };
        let view_bottom = view_top + scroller.client_height() as f64;
        let Some(delta) = super::nearest_scroll(top, bottom, view_top, view_bottom) else {
            return Ok(());
        };
        let options = web_sys::ScrollToOptions::new();
        options.set_top(scroller.scroll_top() as f64 + delta);
        options.set_behavior(if smooth {
            web_sys::ScrollBehavior::Smooth
        } else {
            web_sys::ScrollBehavior::Instant
        });
        scroller.scroll_to_with_scroll_to_options(&options);
        Ok(())
    }

    fn set_pointer_capture(&self, pointer_id: i32) -> Result<(), PlatformError> {
        self.element
            .set_pointer_capture(pointer_id)
            .map_err(|_| PlatformError::NotFound)
    }

    fn query_selector(&self, selector: &str) -> Result<Box<dyn ElementApi>, PlatformError> {
        let element = self
            .element
            .query_selector(selector)
            .ok()
            .flatten()
            .ok_or(PlatformError::NotFound)?;

        Ok(Box::new(WebElement { element }))
    }

    fn query_selector_all(
        &self,
        selector: &str,
    ) -> Result<Vec<Box<dyn ElementApi>>, PlatformError> {
        let nodes = self
            .element
            .query_selector_all(selector)
            .map_err(|_| PlatformError::NotFound)?;

        let items = (0..nodes.length())
            .filter_map(|index| nodes.get(index))
            .filter_map(|node| node.dyn_into::<web_sys::Element>().ok())
            .map(|element| Box::new(WebElement { element }) as Box<dyn ElementApi>)
            .collect();

        Ok(items)
    }
}

pub(super) fn timer() -> Option<&'static dyn TimerApi> {
    Some(&TIMER)
}

struct WebTimer;

static TIMER: WebTimer = WebTimer;

/// `set_timeout` takes `i32` milliseconds: saturate past 24 days rather than
/// wrap into a timer that fires at once.
fn millis(duration: Duration) -> i32 {
    i32::try_from(duration.as_millis()).unwrap_or(i32::MAX)
}

impl TimerApi for WebTimer {
    fn after(&self, delay: Duration, callback: Box<dyn FnOnce()>) -> Box<dyn TimerSubscription> {
        // A fired timer forgets its id: ids are reused, so a stale `Drop` could
        // cancel an unrelated timer.
        let handle: Rc<Cell<Option<i32>>> = Rc::new(Cell::new(None));
        let fired = handle.clone();

        let closure = Closure::once(Box::new(move || {
            fired.set(None);
            callback();
        }) as Box<dyn FnOnce()>);

        handle.set(web_sys::window().and_then(|window| {
            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    millis(delay),
                )
                .ok()
        }));

        Box::new(WebTimerSubscription {
            handle,
            repeating: false,
            _closure: closure,
        })
    }

    fn every(&self, interval: Duration, callback: Box<dyn Fn()>) -> Box<dyn TimerSubscription> {
        let closure = Closure::<dyn FnMut()>::new(callback);
        // An interval runs until cleared, so its id never goes stale.
        let handle = Rc::new(Cell::new(web_sys::window().and_then(|window| {
            window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    millis(interval),
                )
                .ok()
        })));

        Box::new(WebTimerSubscription {
            handle,
            repeating: true,
            _closure: closure,
        })
    }
}

struct WebTimerSubscription {
    /// `None` without a window, or once a one-shot fired.
    handle: Rc<Cell<Option<i32>>>,
    /// Picks `clear_interval` over `clear_timeout`.
    repeating: bool,
    /// Kept alive while the timer is pending: the browser holds the JS function.
    _closure: Closure<dyn FnMut()>,
}

impl TimerSubscription for WebTimerSubscription {}

impl Drop for WebTimerSubscription {
    fn drop(&mut self) {
        let Some(handle) = self.handle.get() else {
            return;
        };
        if let Some(window) = web_sys::window() {
            if self.repeating {
                window.clear_interval_with_handle(handle);
            } else {
                window.clear_timeout_with_handle(handle);
            }
        }
    }
}

pub(super) fn keyboard() -> Option<&'static dyn KeyboardApi> {
    Some(&KEYBOARD)
}

struct WebKeyboard;

static KEYBOARD: WebKeyboard = WebKeyboard;

/// The element a key press landed on, or `None` where the target is not one.
fn key_target(event: &web_sys::KeyboardEvent) -> Option<web_sys::Element> {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
}

/// [`takes_typing`] for the event's target, plus `contenteditable`.
fn editable_target(event: &web_sys::KeyboardEvent) -> bool {
    let Some(target) = key_target(event) else {
        return false;
    };

    if takes_typing(
        target.tag_name().as_str(),
        target.get_attribute("type").as_deref(),
    ) {
        return true;
    }

    target
        .dyn_ref::<web_sys::HtmlElement>()
        .is_some_and(|html| html.is_content_editable())
}

/// [`takes_arrows`] for the event's target.
fn stepping_target(event: &web_sys::KeyboardEvent) -> bool {
    key_target(event).is_some_and(|target| {
        takes_arrows(
            target.tag_name().as_str(),
            target.get_attribute("type").as_deref(),
        )
    })
}

impl WebKeyboard {
    /// In the capture phase, on the window, so no handler can swallow the press
    /// first. Each subscription holds its own listener and filter.
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |event: web_sys::KeyboardEvent| {
                // Escape mid-composition cancels the IME: an unfiltered Escape
                // handler must not eat it and dismiss the surface.
                if event.is_composing() {
                    return;
                }

                let text_entry = editable_target(&event);
                if skip_text_entry && text_entry {
                    return;
                }

                // As `KeyboardData` parses it.
                let key = event.key().parse().unwrap_or(Key::Unidentified);
                let mut modifiers = Modifiers::empty();
                modifiers.set(Modifiers::CONTROL, event.ctrl_key());
                modifiers.set(Modifiers::SHIFT, event.shift_key());
                modifiers.set(Modifiers::ALT, event.alt_key());
                modifiers.set(Modifiers::META, event.meta_key());

                let chord_key = key.clone();
                let handled = callback(KeyChord {
                    key,
                    modifiers,
                    repeat: event.repeat(),
                    text_entry,
                    scopes: Vec::new(),
                });
                if handled {
                    event.prevent_default();
                    // A filtered subscription is a global hotkey; the callback
                    // is opaque until its first press.
                    if skip_text_entry {
                        warn_reserved_chord(&chord_key, modifiers);
                    }
                }
            },
        );

        let target = web_sys::window().and_then(|window| {
            let target: web_sys::EventTarget = window.into();
            target
                .add_event_listener_with_callback_and_bool(
                    "keydown",
                    closure.as_ref().unchecked_ref(),
                    true,
                )
                .ok()?;
            Some(target)
        });

        Box::new(WebKeySubscription { target, closure })
    }
}

impl KeyboardApi for WebKeyboard {
    fn on_key(&self, callback: Box<dyn Fn(KeyChord) -> bool>) -> Box<dyn KeySubscription> {
        self.listen(true, callback)
    }

    fn on_key_unfiltered(
        &self,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        self.listen(false, callback)
    }
}

struct WebKeySubscription {
    /// `None` without a window: the subscription never fires.
    target: Option<web_sys::EventTarget>,
    closure: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

impl KeySubscription for WebKeySubscription {}

impl Drop for WebKeySubscription {
    fn drop(&mut self) {
        if let Some(target) = &self.target {
            // The same three arguments as added, capture included.
            let _ = target.remove_event_listener_with_callback_and_bool(
                "keydown",
                self.closure.as_ref().unchecked_ref(),
                true,
            );
        }
    }
}
