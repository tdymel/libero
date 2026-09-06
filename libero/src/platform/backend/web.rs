use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use dioxus::prelude::{
    Event, Key, KeyboardData, Modifiers, MountedData, MouseData, TransitionData,
};
use wasm_bindgen::{JsCast, JsValue};

use wasm_bindgen::prelude::Closure;

use crate::platform::{
    Dimensions, DocumentApi, ElementApi, KeyChord, KeySubscription, KeyboardApi, PlatformError,
    Read, ScrollApi, ScrollSubscription, TimerApi, TimerSubscription,
    keyboard::{takes_typing, warn_reserved_chord},
};

/// dioxus-web backs a mounted element with the `web_sys::Element` itself, so
/// the whole trait is answerable - no id, no document lookup.
pub(super) fn element(mounted: &Rc<MountedData>) -> Option<Box<dyn ElementApi>> {
    let element = mounted.downcast::<web_sys::Element>()?.clone();
    Some(Box::new(WebElement { element }))
}

/// What a component-driven control held before a form reset.
enum Controlled {
    Input(String, bool),
    Text(String),
    Select(i32),
    Other,
}

/// dioxus-web's `HasTransitionData::as_any` hands back the `web_sys` event
/// itself rather than dioxus's wrapper around it, so that is what the
/// downcast names.
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

/// What HTML counts as interactive content, plus anything a caller made
/// focusable. A label does not forward a click on any of these.
const INTERACTIVE: &str = "a[href], button, input, select, textarea, summary, \
    [tabindex], [contenteditable]:not([contenteditable=\"false\"])";

/// The click's target, walked up to the nearest interactive element and the
/// nearest `boundary`: nested when the first sits strictly inside the second.
/// dioxus-web delegates from the root, so `currentTarget` is not the element
/// the handler sits on - the boundary selector stands in for it.
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

pub(super) fn prefers_reduced_motion() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media(crate::sx::REDUCED_MOTION).ok()?)
        .is_some_and(|query| query.matches())
}

pub(super) fn document() -> Option<&'static dyn DocumentApi> {
    Some(&DOCUMENT)
}

struct WebDocument;

static DOCUMENT: WebDocument = WebDocument;

impl DocumentApi for WebDocument {
    fn active_element(&self) -> Option<Box<dyn ElementApi>> {
        let element = web_sys::window()?.document()?.active_element()?;
        Some(Box::new(WebElement { element }))
    }

    /// `inner_width`/`inner_height`, so the answer includes whatever the
    /// scrollbars leave - which is the box a fixed element is laid out in.
    fn viewport(&self) -> Read<Dimensions> {
        let size = web_sys::window()
            .and_then(|window| {
                let width = window.inner_width().ok()?.as_f64()?;
                let height = window.inner_height().ok()?.as_f64()?;
                Some(Dimensions { width, height })
            })
            .ok_or(PlatformError::Unsupported);
        Box::pin(std::future::ready(size))
    }
}

pub(super) fn scroll() -> Option<&'static dyn ScrollApi> {
    Some(&SCROLL)
}

struct WebScroll;

static SCROLL: WebScroll = WebScroll;

impl ScrollApi for WebScroll {
    /// **In the capture phase, on the window.** A `scroll` event does not
    /// bubble, so a plain listener here would see the page scrolling and miss
    /// every element that scrolls - including a `ScrollArea`, which is what the
    /// docs shell scrolls and where this was first noticed. Capture sees every
    /// scroll in the document on its way down.
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
    /// `None` when there was no window to listen on, so `Drop` has nothing to
    /// undo - the subscription still exists, it just never fires.
    target: Option<web_sys::EventTarget>,
    /// Kept alive for exactly as long as the listener is registered: dropping a
    /// `Closure` frees the JS function the listener still points at.
    closure: Closure<dyn FnMut()>,
}

impl ScrollSubscription for WebScrollSubscription {}

impl Drop for WebScrollSubscription {
    fn drop(&mut self) {
        if let Some(target) = &self.target {
            // The same three arguments it was added with, capture included, or
            // the browser removes nothing.
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

    /// The native reset, then every control marked `data-controlled` gets its
    /// value back: its component still holds that value, and dioxus never
    /// sets a value again that did not change, so the control would show its
    /// default while the component's state says otherwise.
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

    /// The list is rebuilt through a `DataTransfer`, which is the only
    /// constructor a `FileList` has. Every file came from a picker or a drop,
    /// so its `FileData` still carries the `web_sys::File` it was made from.
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

    fn is_focused(&self) -> bool {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.active_element())
            .is_some_and(|active| JsValue::from(active) == JsValue::from(self.element.clone()))
    }

    /// `Node::isConnected`, which is what "still in the document" means on the
    /// web: it is false for a node the renderer has removed and for one that
    /// was built but never appended.
    fn is_connected(&self) -> bool {
        self.element.is_connected()
    }

    // The DOM answers all four immediately - they are futures only because a
    // webview-backed renderer cannot.
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

    fn scroll_to(&self, x: f64, y: f64) -> Result<(), PlatformError> {
        // One call, not `scrollLeft` then `scrollTop`: under
        // `scroll-behavior: smooth` the second write starts a new scroll from
        // the current position and replaces the first, so nothing moves.
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
        // Not rendered (`display: none` somewhere above): there is nothing to
        // show, and a zero rect would scroll an ancestor to its top.
        if rect.width() == 0.0 && rect.height() == 0.0 {
            return Ok(());
        }
        let mut ancestor = self.element.parent_element();
        let scroller = loop {
            let Some(element) = ancestor else {
                return Ok(());
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
        let view_top = scroller.get_bounding_client_rect().top() + scroller.client_top() as f64;
        let view_bottom = view_top + scroller.client_height() as f64;
        // `nearest`: the edge that is out of view, and the top one if the
        // element is taller than the view.
        let delta = if top < view_top || bottom - top > view_bottom - view_top {
            top - view_top
        } else if bottom > view_bottom {
            bottom - view_bottom
        } else {
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

/// `set_timeout` takes an `i32` of milliseconds. A delay past that is 24 days
/// out and a browser would not honour it anyway, so it saturates rather than
/// wrapping into a timer that fires at once.
fn millis(duration: Duration) -> i32 {
    i32::try_from(duration.as_millis()).unwrap_or(i32::MAX)
}

impl TimerApi for WebTimer {
    fn after(&self, delay: Duration, callback: Box<dyn FnOnce()>) -> Box<dyn TimerSubscription> {
        // The subscription and the callback share the handle so that a timer
        // which has already fired forgets its own: the spec only requires an
        // id to be unused, not unique forever, so a later `Drop` clearing a
        // stale id could cancel an unrelated timer that inherited it.
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
        // An interval never goes stale on its own - it runs until it is
        // cleared - so nothing has to forget this one.
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
    /// `None` when there was no window to schedule on, or when a one-shot has
    /// already fired and cleared it. Either way `Drop` has nothing to undo.
    handle: Rc<Cell<Option<i32>>>,
    /// `clear_timeout` and `clear_interval` are separate calls in web-sys even
    /// though the browser's handle space is shared, so the subscription has to
    /// remember which one made it.
    repeating: bool,
    /// Kept alive for exactly as long as the timer is pending: dropping a
    /// `Closure` frees the JS function the browser still holds.
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

/// Whether this event landed in something the user types into - the rule
/// itself is [`takes_typing`], plus `contenteditable`.
fn editable_target(event: &web_sys::KeyboardEvent) -> bool {
    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
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

impl WebKeyboard {
    /// **In the capture phase, on the window.** A key press does bubble, unlike
    /// a scroll, but a handler anywhere along the way can stop it - and a
    /// global shortcut that a dialog's own key handling can silently swallow is
    /// not global. Capture sees the press on the way down, before anything has
    /// the chance.
    ///
    /// `skip_text_entry` is per subscription rather than per capability, so one
    /// subscriber opting out cannot widen what another receives: each holds its
    /// own listener and its own closure.
    fn listen(
        &self,
        skip_text_entry: bool,
        callback: Box<dyn Fn(KeyChord) -> bool>,
    ) -> Box<dyn KeySubscription> {
        let closure = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::new(
            move |event: web_sys::KeyboardEvent| {
                // Ahead of the filter, so it guards the unfiltered path too -
                // which is the one that needs it. Escape mid-composition means
                // "cancel the composition", and every consumer of the opt-out
                // matches Escape and returns `true`, so without this the
                // `prevent_default()` below would eat the cancel and dismiss
                // the surface instead. A press carrying `isComposing` is never
                // a shortcut the user meant, so it costs the filtered path
                // nothing either.
                if event.is_composing() {
                    return;
                }

                if skip_text_entry && editable_target(&event) {
                    return;
                }

                // `Key` parses every named key it knows and falls back to the
                // character itself, which is what `KeyboardData` does too.
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
                });
                if handled {
                    event.prevent_default();
                    // A filtered subscription is a page's global hotkey. Said
                    // on the first press it takes, because the callback is
                    // opaque until then.
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
    /// `None` when there was no window to listen on - the subscription exists
    /// and never fires, so `Drop` has nothing to undo.
    target: Option<web_sys::EventTarget>,
    closure: Closure<dyn FnMut(web_sys::KeyboardEvent)>,
}

impl KeySubscription for WebKeySubscription {}

impl Drop for WebKeySubscription {
    fn drop(&mut self) {
        if let Some(target) = &self.target {
            // The same three arguments it was added with, capture included, or
            // the browser removes nothing.
            let _ = target.remove_event_listener_with_callback_and_bool(
                "keydown",
                self.closure.as_ref().unchecked_ref(),
                true,
            );
        }
    }
}
