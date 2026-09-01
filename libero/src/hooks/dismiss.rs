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

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::provide_root_context;
use dioxus::core::{Attribute, AttributeValue};
use dioxus::html::{EventHandlerValue, PlatformEventData};
use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, FocusReturn, focus_return::use_focus_return},
    platform::{ElementApi, KeySubscription, keyboard, next_task},
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
/// key. A `Modal` hears Escape as a bubbled subtree `onkeydown` and a popover
/// hears it at the document through [`KeyboardApi`](crate::platform::KeyboardApi),
/// and the two transports never have to agree, because only the top layer acts.
///
/// The alternative - stopping propagation from the document-level listener -
/// was rejected: that listener is in capture, so stopping there kills every
/// element handler for the press in the whole document, and it hands the veto
/// to whichever subscriber ran first, which is not layer order.
///
/// **A layer may only be on this stack if its transport reaches every press it
/// would have to answer.** `Modal` qualifies with a bubbled subtree
/// `onkeydown`, because everything inside a modal is inside that subtree and a
/// modal has nothing outside itself to answer for. `use_dismiss` does not,
/// unless it has the document-level transport: a pointer-opened box leaves
/// focus outside itself, so its own handler would never fire and the press
/// would reach a `Modal` that then declined as not-top. See [`use_dismiss`].
#[derive(Clone, Copy)]
pub(crate) struct DismissLayer {
    id: LayerId,
    stack: LayerStack,
}

impl DismissLayer {
    /// Puts this layer on top, until the returned guard is dropped.
    ///
    /// **The guard is the only way off the stack**, and that is deliberate.
    /// Push and pop have to be symmetric on every exit path - a close, an
    /// unmount, `open` simply flipping false, a caller that stops rendering the
    /// consumer with no close path running at all - and one leaked layer pins
    /// the top of the stack and silently wedges Escape for every layer under
    /// it. Nothing errors, and no test we can write would see it: the same
    /// failure signature as a detached `focus()` returning `Ok(())`. So the pop
    /// lives in `Drop` rather than in a close handler, where it cannot be
    /// forgotten.
    ///
    /// `write`, not `try_write`. The tolerance in [`LayerGuard::drop`] is
    /// earned - it runs during the dom's own teardown - and there is no such
    /// path here: a push happens from an effect, in a live scope. A push that
    /// quietly did not happen would leave the layer never on top, so its own
    /// Escape does nothing *and* the `Modal` under it closes instead. Silent,
    /// unassertable, and the same failure class this unit exists to remove.
    #[must_use = "the layer is popped when the guard is dropped"]
    pub(crate) fn push(&self) -> LayerGuard {
        let mut open = self.stack.open;
        {
            let mut layers = open.write();
            layers.retain(|id| *id != self.id);
            layers.push(self.id);
        }
        LayerGuard {
            id: self.id,
            stack: self.stack,
        }
    }

    /// Whether an Escape press belongs to this layer. `peek`, never a read:
    /// this is asked from inside an event handler, which must subscribe to
    /// nothing.
    pub(crate) fn is_top(&self) -> bool {
        self.stack.open.peek().last() == Some(&self.id)
    }
}

/// On the stack until dropped.
pub(crate) struct LayerGuard {
    id: LayerId,
    stack: LayerStack,
}

impl Drop for LayerGuard {
    /// **By identity, not a pop.** A `Modal` opened from inside an open `Menu`
    /// makes the menu the older layer, and closing the menu first must not
    /// take the modal off the top.
    ///
    /// Tolerant of a dead signal because this also runs during the dom's own
    /// teardown, where the root scope may already be gone.
    fn drop(&mut self) {
        let mut open = self.stack.open;
        if let Ok(mut layers) = open.try_write() {
            layers.retain(|id| *id != self.id);
        }
    }
}

/// A layer id for this component. Push it with [`DismissLayer::push`] and keep
/// the guard for as long as the layer is open.
///
/// [`use_dismiss`] handles this for you. `Modal` calls it directly, because it
/// is only ever rendered while it is open and keeps its own Escape transport.
pub(crate) fn use_dismiss_layer() -> DismissLayer {
    let stack = use_hook(layer_stack);
    let id = use_hook(move || {
        let mut next = stack.next;
        let id = LayerId(*next.peek());
        next.set(id.0 + 1);
        id
    });

    DismissLayer { id, stack }
}

/// Why a box is closing, which is what decides whether focus goes back.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Dismissal {
    /// Escape, or the consumer telling us an item was chosen. Focus goes back
    /// to the trigger: the keyboard user is where they were, and the element
    /// they were on is about to disappear.
    Deliberate,
    /// Focus left the box on its own - a click elsewhere. Focus stays where it
    /// went. It was already moved somewhere deliberate, and pulling it back to
    /// the trigger fights the user.
    FocusMoved,
}

/// Which of the three dismissal behaviours a consumer wants, and what focus
/// should do. A `Default`ed struct rather than a builder: a fixed, small set of
/// independent *configuration* fields.
///
/// Nothing reactive lives here. `open` and `placed` are parameters of the hook
/// for that reason, and the "also inside" list is registered through
/// [`DismissHandle::register_inside`], because a submenu only exists after its
/// parent's hook has already run.
#[derive(Clone, PartialEq)]
pub(crate) struct DismissOptions {
    /// Escape closes the box, if it is the top layer.
    pub escape: bool,
    /// Focus leaving the box closes it.
    ///
    /// **Web-only in practice, and off the web it is wrong rather than inert.**
    /// The settle waits for `next_task()`, which is a no-op off the web, so the
    /// check runs before focus has landed; and on the mounted floor
    /// `is_focused()` always answers `false` and `query_selector` is
    /// `Unsupported`, so nothing ever counts as inside and *every* focusout
    /// closes the box - including focus moving from the trigger into the list.
    /// A consumer that has to work on those backends should pass `false` and
    /// close on its own signal.
    pub outside: bool,
    /// A deliberate close hands focus back to whatever opened the box.
    pub return_focus: bool,
    /// Focused once the box is open *and placed* - never on mount. `focus()`
    /// returns `Ok(())` on the `visibility: hidden` box a popover renders
    /// before it has been measured, and does nothing.
    pub initial_focus: Option<ElementHandle>,
}

impl Default for DismissOptions {
    fn default() -> Self {
        Self {
            escape: true,
            outside: true,
            return_focus: true,
            initial_focus: None,
        }
    }
}

/// Counts one more element as inside, until dropped. Handed out by
/// [`DismissHandle::register_inside`]; dropping it unregisters, the same contract
/// [`KeySubscription`] and
/// [`TimerSubscription`](crate::platform::TimerSubscription) state.
pub(crate) struct InsideGuard {
    id: u64,
    inside: Signal<Vec<(u64, ElementHandle)>>,
}

impl Drop for InsideGuard {
    fn drop(&mut self) {
        let mut inside = self.inside;
        if let Ok(mut elements) = inside.try_write() {
            elements.retain(|(id, _)| *id != self.id);
        }
    }
}

/// What a consumer spreads on its floating box, plus the handful of calls it
/// makes from its own handlers.
#[derive(Clone, Copy)]
pub(crate) struct DismissHandle {
    anchor: ElementHandle,
    floating: ElementHandle,
    onclose: Option<Callback<()>>,
    focus_return: FocusReturn,
    layer: DismissLayer,
    escape: bool,
    outside: bool,
    return_focus: bool,
    /// Whether this layer hears Escape at the document, which is the same
    /// question as whether it is allowed on the stack.
    global: bool,
    inside: Signal<Vec<(u64, ElementHandle)>>,
    inside_next: Signal<u64>,
}

impl DismissHandle {
    /// The focus-return contract for this box.
    ///
    /// Snapshot the trigger with `remember_active()` **synchronously inside the
    /// trigger's own handler**, where the active element still is the one the
    /// user acted on. Name a fallback with `fallback()` for the trigger the
    /// application may delete while the box is open - the row whose Delete
    /// button opened a confirm dialog is the canonical case
    /// ([[todos]] item 37).
    ///
    /// Handing the hook back rather than wrapping it: `FocusReturn` already
    /// states both halves of the contract, and a `DismissHandle::remember()`
    /// beside `FocusReturn::remember()` would be two names for two different
    /// things, in the one hook four later units copy from.
    pub(crate) fn focus_return(&self) -> FocusReturn {
        self.focus_return
    }

    /// Registers `element` as counting *inside* for the outside check, until
    /// the returned guard is dropped.
    ///
    /// A submenu is portaled to the document root, so it is not a descendant of
    /// the menu that owns it and focus moving into it reads as focus leaving.
    /// Registration is a call rather than a field because the submenu only
    /// exists after this hook has already run.
    #[must_use = "the element stops counting as inside when the guard is dropped"]
    pub(crate) fn register_inside(&self, element: ElementHandle) -> InsideGuard {
        // A counter rather than the element itself, because the same handle can
        // legitimately be registered twice - a submenu that closes and reopens
        // against a parent that never unmounted - and removing by handle on the
        // first `Drop` would unregister the live registration too.
        let mut next = self.inside_next;
        let id = *next.peek();
        next.set(id + 1);

        let mut inside = self.inside;
        if let Ok(mut elements) = inside.try_write() {
            elements.push((id, element));
        }
        InsideGuard {
            id,
            inside: self.inside,
        }
    }

    /// Closes deliberately, handing focus back - for a consumer whose item was
    /// chosen. Escape takes the same path.
    pub(crate) fn dismiss(&self) {
        self.close(Dismissal::Deliberate);
    }

    /// The attributes for the consumer's floating box: `onfocusout` for the
    /// outside check, and `onkeydown` for Escape **only where there is no
    /// document-level transport**. Where there is one, this box would be the
    /// second handler for the same press.
    pub(crate) fn floating_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();

        if self.escape && !self.global {
            let handle = *self;
            events.push(listener("onkeydown", move |event: Event<KeyboardData>| {
                // A held Escape is one intent, not a stream of them. Without
                // this it walks down the stack, closing the menu and then the
                // modal behind it inside one press. A held ArrowDown scrolling
                // a list still wants every repeat, so this is not a global
                // filter.
                if event.key() != Key::Escape || event.is_auto_repeating() {
                    return;
                }
                // No stack consultation here on purpose: a layer with only this
                // transport never pushed, so it would always find itself not on
                // top and never act.
                //
                // The press stops here. This is a *bubble-phase* stop on this
                // box's own handler, declining to let a press past the layer
                // that just consumed it, and it is not the thing the layer
                // stack rejected - that was a capture-phase stop at the
                // document, which kills every element handler for the press in
                // the whole document and hands the veto to whichever subscriber
                // ran first. `FileField` and `MultiSelect` already stop at
                // their own handlers for the same reason.
                //
                // Without it an enclosing `Modal` hears the same press and
                // closes too. The earlier version of this comment called that
                // "no worse than what shipped", which was true only because no
                // non-portaled consumer existed yet - and this hook's own doc
                // offers one two paragraphs up, a tooltip wanting dismissal
                // without placement, which has no portal to be a sibling
                // through.
                event.prevent_default();
                event.stop_propagation();
                handle.close(Dismissal::Deliberate);
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
                        handle.close(Dismissal::FocusMoved);
                    }
                });
            }));
        }

        events
    }

    /// Whether focus is still somewhere that counts as inside this box.
    ///
    /// Each handle is asked twice: `query_selector(":focus")` finds a focused
    /// *descendant*, and `is_focused()` catches the element itself - which is
    /// the common case for an anchor, since a trigger is usually the focusable
    /// element rather than a wrapper around one.
    fn holds_focus(&self) -> bool {
        let inside = |element: &ElementHandle| {
            element.is_focused() || element.query_selector(":focus").is_ok()
        };

        inside(&self.anchor)
            || inside(&self.floating)
            || self
                .inside
                .peek()
                .iter()
                .any(|(_, element)| inside(element))
    }

    /// Asks the consumer to close, and hands focus back if this was deliberate.
    ///
    /// **Does not touch the layer stack.** The effect keyed on `open` owns
    /// membership, through the guard. Popping here would take this layer off
    /// while the very press that closed it is still bubbling, and a `Modal`
    /// above would then find *itself* on top and close too: the exact
    /// double-close the stack exists to prevent, and the version anyone writing
    /// this fresh reaches for first.
    ///
    /// Deferred a task, the `Modal` rule: closing synchronously from an event
    /// still bubbling through the box being torn down re-enters the same
    /// `EventHandler` and panics. Focus is restored *after* `onclose` for the
    /// same reason `use_modal` restores after its closer - focus landed beside
    /// a box that is still up is taken straight back when it goes.
    fn close(&self, reason: Dismissal) {
        let handle = *self;
        spawn(async move {
            if let Some(onclose) = handle.onclose {
                onclose.call(());
            }
            if handle.return_focus && reason == Dismissal::Deliberate {
                handle.focus_return.restore();
            }
        });
    }
}

/// Closes `floating` when the user presses Escape or focus leaves it, and hands
/// focus back to whatever opened it.
///
/// Owns no open state - `open` is the consumer's, and `onclose` is how this
/// asks for it to change, exactly as [`use_popover`](super::use_popover) owns no
/// placement state of the consumer's.
///
/// `placed` is a parameter rather than a `DismissOptions` field because **the
/// dangerous value is the default one**. It gates
/// [`DismissOptions::initial_focus`](DismissOptions), and a defaulted `true`
/// would hand a consumer who simply forgot it the exact trap the input exists
/// to prevent: `focus()` on the pre-placement `visibility: hidden` box returns
/// `Ok(())` and moves nothing, with nothing to catch. Here it cannot be
/// forgotten. A consumer with no placement to wait for passes `true` on
/// purpose.
///
/// **This layer joins the Escape stack only where it can hear Escape from
/// outside its own subtree**, which means only where
/// [`keyboard()`](crate::platform::keyboard) answers. A pointer-opened box
/// leaves focus wherever it was - inside the enclosing `Modal`, say - so its
/// floating-box `onkeydown` never fires. If it took the top of the stack
/// anyway, the `Modal` that *did* hear the press would decline as not-top and
/// Escape would do nothing at all, which is worse than the behaviour it
/// replaced. So off the web this hook does not push, the `Modal` stays top, and
/// Escape closes the modal exactly as it does today.
///
/// ```ignore
/// let anchor = use_element();
/// let popover = use_popover(anchor, opened(), PopoverOptions::new(gap, padding));
/// let dismiss = use_dismiss(anchor, *popover.floating(), opened(), popover.placed(),
///     Some(close), DismissOptions { initial_focus: Some(first_item), ..Default::default() });
/// ```
pub(crate) fn use_dismiss(
    anchor: ElementHandle,
    floating: ElementHandle,
    open: bool,
    placed: bool,
    onclose: Option<Callback<()>>,
    options: DismissOptions,
) -> DismissHandle {
    let focus_return = use_focus_return();
    let layer = use_dismiss_layer();
    let inside = use_signal(Vec::<(u64, ElementHandle)>::new);
    let inside_next = use_signal(|| 0u64);

    // Decided once. Whether the document can be listened to is a property of
    // the running renderer, not of this render.
    //
    // This is the first place in the codebase to read a platform capability at
    // render time, so: [[codebase/platform-api]]'s "ask in an effect" rule is
    // about *reads that need a mounted element*, and this needs none. It is
    // also safe across hydration, which is the reason the rule looks like it
    // should apply - SSR serialises no listeners, so the server and the client
    // emit identical HTML whichever way this answers, and hydration walks nodes
    // rather than attributes. Moving it into an effect costs a render and buys
    // nothing anyone has been able to name.
    let global = use_hook(|| keyboard().is_some());

    let handle = DismissHandle {
        anchor,
        floating,
        onclose,
        focus_return,
        layer,
        escape: options.escape,
        outside: options.outside,
        return_focus: options.return_focus,
        global,
        inside,
        inside_next,
    };

    // The document subscription and the stack membership have exactly the same
    // lifetime, so they are one slot: taking it drops both, and dropping the
    // hook's own state on unmount drops it too. That is what makes a leaked
    // layer impossible rather than merely unlikely.
    type Listening = Rc<RefCell<Option<(Box<dyn KeySubscription>, LayerGuard)>>>;
    let listening: Listening = use_hook(|| Rc::new(RefCell::new(None)));

    // Bumped from the key callback, which runs outside every dioxus scope - so
    // the signal is owned by the root and dropped by hand, the same obligation
    // `use_popover`'s scroll callback has. The callback deliberately does not
    // close anything: it only records the press, and the effect below acts, in
    // the runtime.
    let escape_tick = use_hook(|| Signal::new_in_scope(0u64, ScopeId::ROOT));
    use_drop({
        let listening = listening.clone();
        move || {
            listening.borrow_mut().take();
            escape_tick.manually_drop();
        }
    });

    let slot = listening.clone();
    use_effect(use_reactive!(|(open,)| {
        if !open || !global || !options.escape {
            slot.borrow_mut().take();
            return;
        }
        if slot.borrow().is_some() {
            return;
        }
        let Some(api) = keyboard() else {
            return;
        };
        // Unfiltered: a dismissible surface has to hear Escape from inside its
        // own text field - a menu's filter box, a palette's search box.
        let subscription = api.on_key_unfiltered(Box::new(move |chord| {
            if chord.key != Key::Escape || chord.repeat || !layer.is_top() {
                // Declining must not prevent the default, or this layer eats
                // the press for the layer above it.
                return false;
            }
            let mut tick = escape_tick;
            let next = tick.peek().wrapping_add(1);
            tick.set(next);
            true
        }));
        *slot.borrow_mut() = Some((subscription, layer.push()));
    }));

    // Acts on what the key callback recorded. Comparing against what this scope
    // has already seen is what keeps a reopen from closing immediately on a
    // press from three opens ago.
    let mut seen = use_signal(|| 0u64);
    use_effect(move || {
        let tick = escape_tick();
        if tick == *seen.peek() {
            return;
        }
        seen.set(tick);
        handle.close(Dismissal::Deliberate);
    });

    // Armed on the opening edge and consumed once the focus lands, so reopening
    // focuses again and a re-placement on scroll does not.
    let mut entering = use_signal(|| false);
    use_effect(use_reactive!(|(open,)| entering.set(open)));

    let initial_focus = options.initial_focus;
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

    handle
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

/// The Escape arbitration and the layer lifetime, against a real `VirtualDom`
/// with a real dispatched key press. None of this is visible to SSR: it is
/// entirely about which of two handlers acts on one event.
///
/// These are unit tests rather than integration ones because the hook is
/// `pub(crate)`.
///
/// **What cannot be tested here**: the web arm. `platform::keyboard()` is
/// `None` off wasm, so `use_dismiss` never pushes a layer in this harness and
/// the document-level transport never runs. What is testable is that its
/// absence leaves `Modal` exactly as it was, which is the regression the
/// amended push rule exists to prevent.
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
        hooks::PopoverOptions,
        hooks::{ModalScope, use_element, use_modal, use_popover},
        use_theme,
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

    /// Just the marker line the app prints, so a failure message is readable -
    /// the rendered document is mostly the theme stylesheet.
    fn state(dom: &VirtualDom) -> String {
        let html = dioxus_ssr::render(dom);
        let at = html.rfind("state:").expect("the state marker");
        html[at..].to_string()
    }

    /// Drains tasks and re-renders until nothing is left: a close travels
    /// through a spawned task, then the consumer's signal, then the effect that
    /// takes the layer off the stack.
    fn settle(dom: &mut VirtualDom) {
        for _ in 0..4 {
            dom.process_events();
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
        }
    }

    fn press(dom: &mut VirtualDom, target: ElementId) {
        dom.runtime()
            .handle_event("keydown", Event::new(escape(), true), target);
        settle(dom);
    }

    /// How many `<div>`s are open at `at`, so two elements can be shown to be
    /// siblings rather than nested. Nesting of other tags does not affect it.
    fn div_depth_at(html: &str, at: usize) -> i32 {
        let mut depth = 0;
        let mut rest = &html[..at];
        while let Some(next) = rest.find("<div") {
            depth += 1;
            rest = &rest[next + 4..];
        }
        let mut closes = 0;
        let mut rest = &html[..at];
        while let Some(next) = rest.find("</div>") {
            closes += 1;
            rest = &rest[next + 6..];
        }
        depth - closes
    }

    /// Two modals, the inner opened while the outer was up. Both always push -
    /// a modal's bubbled handler covers its whole subtree - so this is the
    /// arbitration on every backend.
    #[component]
    fn TwoModals(outer: Signal<bool>, inner: Signal<bool>) -> Element {
        rsx! {
            if outer() {
                Modal { onclose: move |_| { let mut outer = outer; outer.set(false); },
                    "outer body"
                }
            }
            if inner() {
                Modal { onclose: move |_| { let mut inner = inner; inner.set(false); },
                    "inner body"
                }
            }
        }
    }

    fn two_modals() -> Element {
        let outer = use_signal(|| true);
        let inner = use_signal(|| true);

        rsx! {
            LiberoProvider { TwoModals { outer, inner } }
            "state: outer={outer} inner={inner}"
        }
    }

    #[test]
    fn escape_acts_on_the_newest_modal_and_no_other() {
        dioxus::html::set_event_converter(Box::new(EscapeConverter));
        let mut dom = VirtualDom::new(two_modals);
        let mut find = FindKeydownListeners::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);

        // Each modal registers one on its own root and one on its focus trap.
        assert_eq!(
            find.keydown.len(),
            4,
            "expected four keydown listeners, got {:?}",
            find.keydown
        );
        let outer_root = find.keydown[0];
        let inner_root = find.keydown[2];

        assert_eq!(state(&dom), "state: outer=true inner=true");

        // The older modal hears the press and declines: it is not the top.
        press(&mut dom, outer_root);
        assert_eq!(
            state(&dom),
            "state: outer=true inner=true",
            "a layer under the top one must not act"
        );

        press(&mut dom, inner_root);
        assert_eq!(state(&dom), "state: outer=true inner=false");

        // And with the newer one gone the older is top again, which is the
        // guard popping by identity rather than the stack being reset.
        press(&mut dom, outer_root);
        assert_eq!(state(&dom), "state: outer=false inner=false");
    }

    /// The discriminating test for popping by identity.
    ///
    /// The older modal goes away on its own - a route change, a caller that
    /// simply stops rendering it - so nothing runs a close handler and only the
    /// guard's `Drop` takes it off the stack. It has to take *its own* entry. A
    /// pop-the-last would take the newer modal's instead, and the newer modal
    /// would then silently stop answering Escape with nothing to show for it.
    #[test]
    fn dropping_an_older_layer_leaves_the_newer_one_on_top() {
        /// Outside both modals, so its own Escape bubbles to the app root and
        /// through no layer's handler. A key rather than a click because the
        /// test converter only speaks keyboard.
        #[component]
        fn Closer(outer: Signal<bool>) -> Element {
            let style = use_box().prepare();
            style.render(
                HtmlTag::Div,
                vec![listener("onkeydown", move |_: Event<KeyboardData>| {
                    let mut outer = outer;
                    outer.set(false);
                })],
                rsx! {},
            )
        }

        fn app() -> Element {
            let outer = use_signal(|| true);
            let inner = use_signal(|| true);

            rsx! {
                LiberoProvider {
                    Closer { outer }
                    TwoModals { outer, inner }
                }
                "state: outer={outer} inner={inner}"
            }
        }

        dioxus::html::set_event_converter(Box::new(EscapeConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindKeydownListeners::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);

        // The closer, then each modal's root and focus trap.
        assert_eq!(
            find.keydown.len(),
            5,
            "expected five keydown listeners, got {:?}",
            find.keydown
        );
        let closer = find.keydown[0];
        let inner_root = find.keydown[3];

        press(&mut dom, closer);
        assert_eq!(state(&dom), "state: outer=false inner=true");

        press(&mut dom, inner_root);
        assert_eq!(
            state(&dom),
            "state: outer=false inner=false",
            "the newer layer should still have been on top"
        );
    }

    /// A dismissible region drawn *inside* the modal rather than portaled, so
    /// an Escape on it bubbles through the modal's own handler. `outside` is
    /// off because the focusout settle needs a platform, and `return_focus`
    /// because there is no document to hand focus back to.
    #[component]
    fn Region(open: Signal<bool>) -> Element {
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
            true,
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
            .render(HtmlTag::Div, dismiss.floating_events(), rsx! { "region" })
    }

    fn region_in_modal() -> Element {
        let mut modal = use_signal(|| true);
        let region = use_signal(|| true);

        rsx! {
            LiberoProvider {
                if modal() {
                    Modal { onclose: move |_| modal.set(false),
                        Region { open: region }
                    }
                }
            }
            "state: modal={modal} region={region}"
        }
    }

    /// The layer consumes the press, so the `Modal` around it never hears it.
    ///
    /// This is the assertion that discriminates. Before the bubble-phase
    /// `stop_propagation`, both closed, and the doc comment called that "no
    /// worse than what shipped" - true only because no non-portaled consumer
    /// existed yet, which is a rationalisation rather than a reason. The old
    /// test asserted only that the modal closed, so it passed either way and
    /// made the defect look intentional.
    #[test]
    fn a_layers_own_handler_consumes_the_press_before_the_modal_hears_it() {
        dioxus::html::set_event_converter(Box::new(EscapeConverter));
        let mut dom = VirtualDom::new(region_in_modal);
        let mut find = FindKeydownListeners::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);

        let region = *find.keydown.last().expect("no keydown listener");
        assert_eq!(state(&dom), "state: modal=true region=true");

        press(&mut dom, region);
        assert_eq!(
            state(&dom),
            "state: modal=true region=false",
            "the press should have stopped at the layer that consumed it"
        );
    }

    /// The other side, and the control property the amended push rule exists
    /// for.
    ///
    /// The press lands outside the region - focus on the trigger, which is
    /// every `initial_focus: None` consumer - so the region's own handler never
    /// fires. `platform::keyboard()` is `None` here, so the region is not on
    /// the stack either, and the `Modal` is therefore top and closes. If the
    /// region had pushed without a transport that could hear this press, the
    /// modal would have declined as not-top and Escape would have done nothing
    /// at all, where today it closes the modal.
    #[test]
    fn a_layer_that_cannot_hear_escape_never_wedges_the_modal() {
        dioxus::html::set_event_converter(Box::new(EscapeConverter));
        let mut dom = VirtualDom::new(region_in_modal);
        let mut find = FindKeydownListeners::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);

        let modal_root = find.keydown[0];
        assert_eq!(state(&dom), "state: modal=true region=true");

        press(&mut dom, modal_root);
        assert_eq!(
            state(&dom),
            "state: modal=false region=true",
            "the modal must answer a press the region cannot hear"
        );
    }

    /// The other half: with no modal over it, the floating box's own handler is
    /// the transport and it closes the box.
    #[test]
    fn a_layer_with_only_its_own_handler_still_closes_on_escape() {
        fn app() -> Element {
            let region = use_signal(|| true);

            rsx! {
                LiberoProvider { Region { open: region } }
                "state: region={region}"
            }
        }

        dioxus::html::set_event_converter(Box::new(EscapeConverter));
        let mut dom = VirtualDom::new(app);
        let mut find = FindKeydownListeners::default();
        dom.rebuild(&mut find);
        dom.render_immediate(&mut find);

        let region = *find.keydown.last().expect("no keydown listener");
        assert_eq!(state(&dom), "state: region=true");

        press(&mut dom, region);
        assert_eq!(state(&dom), "state: region=false");
    }

    /// Two portaled modals are siblings at the outlet, not nested - which is
    /// why no Escape bubbles between them today and why the `is_top` guard
    /// changes nothing for the shipped case. Measured rather than reasoned.
    #[test]
    fn two_portaled_modals_are_siblings_at_the_outlet() {
        #[component]
        fn Opener() -> Element {
            let first = use_modal(|_: ModalScope<()>| rsx! { "first body" });
            let second = use_modal(|_: ModalScope<()>| rsx! { "second body" });
            use_hook(move || {
                first.open();
                second.open();
            });
            rsx! {}
        }

        fn app() -> Element {
            rsx! { LiberoProvider { Opener {} } }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);

        let first = html.find("first body").expect("the first modal");
        let second = html.find("second body").expect("the second modal");

        assert_eq!(
            div_depth_at(&html, first),
            div_depth_at(&html, second),
            "the two modals should be siblings, not nested"
        );
    }

    /// The wiring the docs page prints in its `use_dismiss` sample, which is
    /// not compiled anywhere else. Without this the sample can go stale and
    /// nothing says so.
    #[test]
    fn the_documented_wiring_builds_and_renders() {
        #[component]
        fn Documented(opened: Signal<bool>) -> Element {
            let theme = use_theme();
            let anchor = use_element();
            let first_item = use_element();
            let close = use_callback(move |()| {
                let mut opened = opened;
                opened.set(false);
            });

            let popover = use_popover(
                anchor,
                opened(),
                PopoverOptions::new(theme.popover.gap, theme.popover.padding),
            );
            let dismiss = use_dismiss(
                anchor,
                *popover.floating(),
                opened(),
                popover.placed(),
                Some(close),
                DismissOptions {
                    initial_focus: Some(first_item),
                    ..Default::default()
                },
            );

            let dropdown = use_box().style(popover.style()).prepare();
            popover.show(opened().then(|| {
                dropdown.clone().element(popover.floating()).render(
                    HtmlTag::Div,
                    dismiss.floating_events(),
                    rsx! { "dropdown body" },
                )
            }));

            let trigger = use_box().prepare();
            trigger
                .element(&anchor)
                .attr("type", "button")
                .event("onclick", move |_: Event<MouseData>| {
                    let mut opened = opened;
                    dismiss.focus_return().remember_active();
                    let next = !opened();
                    opened.set(next);
                })
                .render(HtmlTag::Button, vec![], rsx! { "Open" })
        }

        fn app() -> Element {
            let opened = use_signal(|| true);
            rsx! { LiberoProvider { Documented { opened } } }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);

        assert!(html.contains("dropdown body"), "the box should render");
        assert!(html.contains("Open"), "the trigger should render");
    }
}
