//! Dismissal for a floating box: Escape, focus leaving it, and handing focus
//! back - written once, instead of once per consumer.
//!
//! [`use_popover`](super::use_popover) places a box and owns no open state.
//! This is the other half: *when* a box should close. The two are separate
//! hooks because a tooltip wants dismissal without placement, and because
//! folding policy into the placement hook would contradict the promise in its
//! own doc comment.
//!
//! It renders nothing. [`DismissHandle::floating_events`] hands the consumer
//! two attributes to spread on its own box, and the consumer keeps whatever
//! role, theming and ARIA its popup type needs.

// The hook lands ahead of its first consumer: `Menu`, `Menubar` and
// `HoverCard` are what it exists for, and none of them is written yet, while
// `Modal` uses only the layer stack. Drop this once D2 lands.
#![allow(dead_code)]

use dioxus::core::provide_root_context;
use dioxus::core::{Attribute, AttributeValue};
use dioxus::html::{EventHandlerValue, PlatformEventData};
use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, FocusReturn, focus_return::use_focus_return},
    platform::{ElementApi, next_task},
};

/// One open dismissible layer, identified only by when it opened.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct LayerId(u64);

/// Every open dismissible layer in this document, in the order they opened.
///
/// One per `VirtualDom`, reached through the root context rather than a
/// `thread_local!`: two `VirtualDom`s share a thread routinely - every
/// integration test builds several - and a `thread_local` stack would outlive
/// the dom that filled it and answer for a document that is gone.
///
/// The signals themselves live in [`ScopeId::ROOT`] because a layer is pushed
/// and popped from handlers and drops that belong to no scope.
#[derive(Clone, Copy)]
struct LayerStack {
    open: Signal<Vec<LayerId>>,
    next: Signal<u64>,
}

fn layer_stack() -> LayerStack {
    try_consume_context::<LayerStack>().unwrap_or_else(|| {
        provide_root_context(LayerStack {
            open: Signal::new_in_scope(Vec::new(), ScopeId::ROOT),
            next: Signal::new_in_scope(0, ScopeId::ROOT),
        })
    })
}

/// This layer's place in the stack of open layers.
///
/// **Whoever hears an Escape asks this before acting**, wherever it heard the
/// key. That is the whole arbitration: a `Modal` hears Escape as a bubbled
/// subtree `onkeydown` and a popover hears it on its own floating box, and the
/// two transports never have to agree, because only the top layer acts. A
/// press that reaches two handlers closes one layer.
///
/// The alternative - stopping propagation from the document-level
/// [`KeyboardApi`](crate::platform::KeyboardApi) listener - was rejected: that
/// listener is in capture, so stopping there kills every element handler for
/// the press in the whole document, and it hands the veto to whichever
/// subscriber ran first, which is not layer order.
#[derive(Clone, Copy)]
pub(crate) struct DismissLayer {
    id: LayerId,
    stack: LayerStack,
}

impl DismissLayer {
    /// Puts this layer on top. Idempotent - an effect keyed on `open` may run
    /// again without the layer having closed in between.
    pub(crate) fn open(&self) {
        let mut open = self.stack.open;
        if open.peek().contains(&self.id) {
            return;
        }
        if let Ok(mut layers) = open.try_write() {
            layers.push(self.id);
        }
    }

    /// Takes this layer off, from wherever it sits - a layer under an open one
    /// can close on its own, so this is not a pop.
    ///
    /// Written tolerantly because it also runs from `use_drop` during the
    /// dom's own teardown, where the root scope may already be gone.
    pub(crate) fn close(&self) {
        let mut open = self.stack.open;
        if let Ok(mut layers) = open.try_write() {
            layers.retain(|id| *id != self.id);
        }
    }

    /// Whether an Escape press belongs to this layer. `peek`, never a read:
    /// this is asked from inside an event handler, which must subscribe to
    /// nothing.
    pub(crate) fn is_top(&self) -> bool {
        self.stack.open.peek().last() == Some(&self.id)
    }
}

/// A layer of this component's own, popped when the component unmounts.
///
/// [`use_dismiss`] calls it for you. `Modal` calls it directly, because it is
/// only ever rendered while it is open and keeps its own Escape transport.
pub(crate) fn use_dismiss_layer() -> DismissLayer {
    let stack = use_hook(layer_stack);
    let id = use_hook(move || {
        let mut next = stack.next;
        let id = LayerId(*next.peek());
        next.set(id.0 + 1);
        id
    });

    let layer = DismissLayer { id, stack };
    use_drop(move || layer.close());
    layer
}

/// Which of the three dismissal behaviours a consumer wants, and what focus
/// should do. A `Default`ed struct rather than a builder: a fixed, small set of
/// independent fields.
#[derive(Clone, PartialEq)]
pub(crate) struct DismissOptions {
    /// Escape closes the box, if it is the top layer.
    pub escape: bool,
    /// Focus leaving the box closes it.
    pub outside: bool,
    /// Closing hands focus back to whatever opened the box.
    pub return_focus: bool,
    /// Focused once the box is open *and placed* - never on mount. `focus()`
    /// returns `Ok(())` on the `visibility: hidden` box a popover renders
    /// before it has been measured, and does nothing.
    pub initial_focus: Option<ElementHandle>,
    /// Extra elements that count as *inside* for the outside check. A submenu
    /// is portaled to the root, so it is not a descendant of the menu that
    /// owns it, and focus moving into it must not read as focus leaving.
    pub also_inside: Vec<ElementHandle>,
    /// Whether the box has been measured - pass `popover.placed()`. Gates
    /// [`initial_focus`](Self::initial_focus) only. `true` for a consumer with
    /// no placement to wait for.
    pub placed: bool,
}

impl Default for DismissOptions {
    fn default() -> Self {
        Self {
            escape: true,
            outside: true,
            return_focus: true,
            initial_focus: None,
            also_inside: Vec::new(),
            placed: true,
        }
    }
}

/// What a consumer spreads on its floating box, plus the one call it makes
/// from its own trigger handler.
#[derive(Clone, Copy)]
pub(crate) struct DismissHandle {
    anchor: ElementHandle,
    floating: ElementHandle,
    open: bool,
    onclose: Option<Callback<()>>,
    focus_return: FocusReturn,
    layer: DismissLayer,
    escape: bool,
    outside: bool,
    return_focus: bool,
    /// Kept in a signal rather than on the handle so the handle stays `Copy`
    /// and the handlers below can capture it.
    also_inside: Signal<Vec<ElementHandle>>,
}

impl DismissHandle {
    /// Snapshots whatever holds focus now, so closing can hand it back.
    ///
    /// **Call it synchronously inside the trigger's own handler** - there the
    /// active element still is the one the user acted on.
    ///
    /// A no-op while the box is already open: a control *inside* an open box
    /// reopening it would otherwise replace a perfectly good trigger with an
    /// element that is about to be torn down ([[todos]] item 37).
    pub(crate) fn remember(&self) {
        if self.open {
            return;
        }
        self.focus_return.remember_active();
    }

    /// Names where focus should land if the trigger is gone by the time the
    /// box closes - see [`FocusReturn::fallback`].
    pub(crate) fn fallback(&self, element: ElementHandle) {
        self.focus_return.fallback(element);
    }

    /// The two attributes for the consumer's floating box: `onkeydown` for
    /// Escape and `onfocusout` for the outside check. Either is absent when
    /// its option is off, so a box that wants neither carries no listener.
    pub(crate) fn floating_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();

        if self.escape {
            let handle = *self;
            events.push(listener("onkeydown", move |event: Event<KeyboardData>| {
                if event.key() != Key::Escape || !handle.layer.is_top() {
                    return;
                }
                event.prevent_default();
                handle.close();
            }));
        }

        if self.outside {
            let handle = *self;
            events.push(listener("onfocusout", move |_: Event<FocusData>| {
                // `focusout` is dispatched *before* `focusin`, so a check here
                // sees focus nowhere at all. After the platform's next task it
                // has landed. Off the web `next_task()` is a no-op, which is
                // why this settle is web-only in practice.
                spawn(async move {
                    next_task().await;
                    if !handle.holds_focus() {
                        handle.close();
                    }
                });
            }));
        }

        events
    }

    /// Whether focus is still somewhere that counts as inside this popover.
    ///
    /// Each handle is asked twice: `query_selector(":focus")` finds a focused
    /// *descendant*, and `is_focused()` catches the element itself - which is
    /// the common case for an anchor, since a trigger is usually the focusable
    /// element rather than a wrapper around one.
    fn holds_focus(&self) -> bool {
        let inside = |element: &ElementHandle| {
            element.is_focused() || element.query_selector(":focus").is_ok()
        };

        inside(&self.anchor) || inside(&self.floating) || self.also_inside.peek().iter().any(inside)
    }

    /// Asks the consumer to close, then hands focus back.
    ///
    /// **Does not pop the layer** - the effect keyed on `open` does, once the
    /// consumer has actually closed. Popping here would take this layer off
    /// the stack while the very press that closed it is still bubbling, and
    /// the `Modal` above would then find *itself* on top and close too: the
    /// exact double-close the stack exists to prevent. It also keeps one owner
    /// of stack membership rather than two.
    ///
    /// Deferred a task, the `Modal` rule: closing synchronously from an event
    /// still bubbling through the box being torn down re-enters the same
    /// `EventHandler` and panics. Focus is restored *after* `onclose` for the
    /// same reason `use_modal` restores after its closer - focus landed beside
    /// a box that is still up is taken straight back when it goes.
    fn close(&self) {
        let handle = *self;
        spawn(async move {
            if let Some(onclose) = handle.onclose {
                onclose.call(());
            }
            if handle.return_focus {
                handle.focus_return.restore();
            }
        });
    }
}

/// Closes `floating` when the user presses Escape or focus leaves it, and
/// hands focus back to whatever opened it.
///
/// Owns no open state - `open` is the consumer's, and `onclose` is how this
/// asks for it to change, exactly as [`use_popover`](super::use_popover) owns
/// no placement state of the consumer's.
///
/// ```ignore
/// let anchor = use_element();
/// let popover = use_popover(anchor, opened(), PopoverOptions::new(gap, padding));
/// let dismiss = use_dismiss(anchor, *popover.floating(), opened(), Some(close),
///     DismissOptions { initial_focus: Some(first_item), placed: popover.placed(),
///                      ..Default::default() });
/// ```
pub(crate) fn use_dismiss(
    anchor: ElementHandle,
    floating: ElementHandle,
    open: bool,
    onclose: Option<Callback<()>>,
    options: DismissOptions,
) -> DismissHandle {
    let focus_return = use_focus_return();
    let layer = use_dismiss_layer();

    // Written during render, read only with `peek`: nothing subscribes to it,
    // so the write starts no second render, and the handlers see the current
    // list rather than last render's.
    let mut also_inside = use_signal(Vec::<ElementHandle>::new);
    if *also_inside.peek() != options.also_inside {
        also_inside.set(options.also_inside.clone());
    }

    // On the stack exactly while the box is open.
    use_effect(use_reactive!(|(open,)| {
        if open {
            layer.open();
        } else {
            layer.close();
        }
    }));

    // Armed on the opening edge and consumed once the focus lands, so
    // reopening focuses again and a re-placement on scroll does not.
    let mut entering = use_signal(|| false);
    use_effect(use_reactive!(|(open,)| entering.set(open)));

    let initial_focus = options.initial_focus;
    let placed = options.placed;
    use_effect(use_reactive!(|(open, placed)| {
        // Read first, branch second - this subscribes the effect to the box
        // mounting, and it re-runs on a *re*-mount, which is every reopen.
        let mounted = floating.mount_token().is_some();
        if !open || !placed || !mounted || !entering() {
            return;
        }
        entering.set(false);
        if let Some(target) = initial_focus {
            let _ = target.focus();
        }
    }));

    DismissHandle {
        anchor,
        floating,
        open,
        onclose,
        focus_return,
        layer,
        escape: options.escape,
        outside: options.outside,
        return_focus: options.return_focus,
        also_inside,
    }
}

/// One event attribute, built by hand - the one thing `rsx!` does that a plain
/// call cannot.
///
/// The handler has to go through `EventHandlerValue`: a renderer delivers a
/// `PlatformEventData`, and only that conversion turns it into `T`. A bare
/// `AttributeValue::listener::<T>` type-checks and panics on the first real
/// event. Same reason as [`BoxStyle::event`](crate::components::Box).
fn listener<T>(name: &'static str, handler: impl FnMut(Event<T>) + 'static) -> Attribute
where
    T: for<'a> From<&'a PlatformEventData> + 'static,
{
    Attribute::new(
        name,
        AttributeValue::Listener(handler.into_platform_listener().erase()),
        None,
        false,
    )
}

/// The Escape arbitration, against a real `VirtualDom` with a real dispatched
/// key press - the one thing SSR cannot show, because it is entirely about two
/// handlers hearing the same event.
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use dioxus::core::{AttributeValue, ElementId, WriteMutations};
    use dioxus::html::{HasKeyboardData, PlatformEventData};
    use dioxus::prelude::*;

    use super::*;
    use crate::{
        LiberoProvider,
        components::{HtmlTag, Modal, use_box},
        hooks::use_element,
    };

    /// Every element that registered a `keydown` listener, in the order the
    /// renderer saw them - which is creation order, so outermost first.
    #[derive(Default)]
    struct FindKeydownListeners {
        last: Option<ElementId>,
        keydown: Vec<ElementId>,
    }

    impl WriteMutations for FindKeydownListeners {
        fn push_id(&mut self, id: ElementId) {
            self.last = Some(id);
        }
        fn set_id(&mut self, id: ElementId) {
            self.last = Some(id);
        }
        fn add_event_listener(&mut self, name: &str) {
            if name == "keydown"
                && let Some(id) = self.last
            {
                self.keydown.push(id);
            }
        }
        fn child(&mut self, _index: usize) {}
        fn pop(&mut self) {}
        fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
        fn create_text(&mut self, _value: &str) {}
        fn clone(&mut self) {}
        fn append_children(&mut self, _m: usize) {}
        fn replace_with(&mut self, _m: usize) {}
        fn insert_after(&mut self, _m: usize) {}
        fn insert_before(&mut self, _m: usize) {}
        fn set_attribute(&mut self, _n: &str, _ns: Option<&str>, _v: &AttributeValue) {}
        fn set_text(&mut self, _value: &str) {}
        fn remove_event_listener(&mut self, _name: &str) {}
        fn remove(&mut self) {}
    }

    /// A stand-in for the renderer's key event; only Escape is ever pressed.
    #[derive(Clone, Copy)]
    struct FakeEscape;

    impl HasKeyboardData for FakeEscape {
        fn key(&self) -> Key {
            Key::Escape
        }
        fn code(&self) -> Code {
            Code::Escape
        }
        fn location(&self) -> dioxus::html::input_data::keyboard_types::Location {
            dioxus::html::input_data::keyboard_types::Location::Standard
        }
        fn is_auto_repeating(&self) -> bool {
            false
        }
        fn is_composing(&self) -> bool {
            false
        }
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    impl dioxus::html::point_interaction::ModifiersInteraction for FakeEscape {
        fn modifiers(&self) -> dioxus::html::keyboard_types::Modifiers {
            Default::default()
        }
    }

    struct EscapeConverter;

    impl dioxus::html::HtmlEventConverter for EscapeConverter {
        fn convert_keyboard_data(&self, _event: &PlatformEventData) -> dioxus::html::KeyboardData {
            dioxus::html::KeyboardData::new(FakeEscape)
        }
        fn convert_animation_data(&self, _e: &PlatformEventData) -> dioxus::html::AnimationData {
            unimplemented!()
        }
        fn convert_before_input_data(
            &self,
            _e: &PlatformEventData,
        ) -> dioxus::html::BeforeInputData {
            unimplemented!()
        }
        fn convert_cancel_data(&self, _e: &PlatformEventData) -> dioxus::html::CancelData {
            unimplemented!()
        }
        fn convert_clipboard_data(&self, _e: &PlatformEventData) -> dioxus::html::ClipboardData {
            unimplemented!()
        }
        fn convert_composition_data(
            &self,
            _e: &PlatformEventData,
        ) -> dioxus::html::CompositionData {
            unimplemented!()
        }
        fn convert_drag_data(&self, _e: &PlatformEventData) -> dioxus::html::DragData {
            unimplemented!()
        }
        fn convert_focus_data(&self, _e: &PlatformEventData) -> dioxus::html::FocusData {
            unimplemented!()
        }
        fn convert_form_data(&self, _e: &PlatformEventData) -> dioxus::html::FormData {
            unimplemented!()
        }
        fn convert_image_data(&self, _e: &PlatformEventData) -> dioxus::html::ImageData {
            unimplemented!()
        }
        fn convert_media_data(&self, _e: &PlatformEventData) -> dioxus::html::MediaData {
            unimplemented!()
        }
        fn convert_mounted_data(&self, _e: &PlatformEventData) -> dioxus::html::MountedData {
            unimplemented!()
        }
        fn convert_mouse_data(&self, _e: &PlatformEventData) -> dioxus::html::MouseData {
            unimplemented!()
        }
        fn convert_pointer_data(&self, _e: &PlatformEventData) -> dioxus::html::PointerData {
            unimplemented!()
        }
        fn convert_resize_data(&self, _e: &PlatformEventData) -> dioxus::html::ResizeData {
            unimplemented!()
        }
        fn convert_scroll_data(&self, _e: &PlatformEventData) -> dioxus::html::ScrollData {
            unimplemented!()
        }
        fn convert_selection_data(&self, _e: &PlatformEventData) -> dioxus::html::SelectionData {
            unimplemented!()
        }
        fn convert_toggle_data(&self, _e: &PlatformEventData) -> dioxus::html::ToggleData {
            unimplemented!()
        }
        fn convert_touch_data(&self, _e: &PlatformEventData) -> dioxus::html::TouchData {
            unimplemented!()
        }
        fn convert_transition_data(&self, _e: &PlatformEventData) -> dioxus::html::TransitionData {
            unimplemented!()
        }
        fn convert_visible_data(&self, _e: &PlatformEventData) -> dioxus::html::VisibleData {
            unimplemented!()
        }
        fn convert_wheel_data(&self, _e: &PlatformEventData) -> dioxus::html::WheelData {
            unimplemented!()
        }
    }

    fn escape() -> Rc<dyn std::any::Any> {
        Rc::new(PlatformEventData::new(Box::new(FakeEscape)))
    }

    /// A dismissible region rendered *inside* the modal rather than portaled,
    /// so an Escape on it bubbles through the modal's own handler - which is
    /// exactly the arbitration this is about. `outside` is off because the
    /// focusout settle needs a platform, and `return_focus` because there is
    /// no document to hand focus back to.
    #[component]
    fn Layer(open: Signal<bool>) -> Element {
        let anchor = use_element();
        let floating = use_element();
        let close = use_callback(move |()| {
            let mut open = open;
            open.set(false);
        });
        let dismiss = use_dismiss(
            anchor,
            floating,
            open(),
            Some(close),
            DismissOptions {
                outside: false,
                return_focus: false,
                ..Default::default()
            },
        );
        let style = use_box().prepare();

        if !open() {
            return rsx! {};
        }
        style
            .element(&floating)
            .render(HtmlTag::Div, dismiss.floating_events(), rsx! { "layer" })
    }

    fn app() -> Element {
        let mut modal = use_signal(|| true);
        let layer = use_signal(|| true);

        rsx! {
            LiberoProvider {
                if modal() {
                    Modal { onclose: move |_| modal.set(false),
                        Layer { open: layer }
                    }
                }
            }
            "modal={modal} layer={layer}"
        }
    }

    /// Just the marker line the app prints, so a failure message is readable -
    /// the rendered document is mostly the theme stylesheet.
    fn state(dom: &VirtualDom) -> String {
        let html = dioxus_ssr::render(dom);
        let at = html.rfind("modal=").expect("the state marker");
        html[at..].to_string()
    }

    /// Drains tasks and re-renders until nothing is left: a close travels
    /// through a spawned task, then the consumer's signal, then the effect
    /// that takes the layer off the stack.
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.process_events();
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
    }

    #[test]
    fn escape_closes_only_the_top_layer() {
        dioxus::html::set_event_converter(Box::new(EscapeConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindKeydownListeners::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);
        dom.process_events();
        dom.render_immediate(&mut find);

        // The modal's own root, the focus trap inside it, and the layer.
        // Creation order is outermost first, so the layer is the last.
        assert_eq!(
            find.keydown.len(),
            3,
            "expected three keydown listeners, got {:?}",
            find.keydown
        );
        let layer = *find.keydown.last().expect("no keydown listener");

        assert_eq!(state(&dom), "modal=true layer=true");

        // Bubbles, so the modal's handler hears this press too - and does
        // nothing, because the layer above it is the top of the stack.
        dom.runtime()
            .handle_event("keydown", Event::new(escape(), true), layer);
        settle(&mut dom);
        assert_eq!(
            state(&dom),
            "modal=true layer=false",
            "the modal should have stayed open"
        );

        // With the layer gone the modal is top, so the next press reaches it.
        let modal = find.keydown[0];
        dom.runtime()
            .handle_event("keydown", Event::new(escape(), true), modal);
        settle(&mut dom);
        assert_eq!(
            state(&dom),
            "modal=false layer=false",
            "the modal should have closed"
        );
    }
}
