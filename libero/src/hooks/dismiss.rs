//! Dismissal for a floating box: Escape, focus leaving it, and handing focus
//! back. [`use_popover`](super::use_popover)'s other half: *when* it closes.
//!
//! Without [`keyboard()`] (the WebView floor) Escape reaches only the focused
//! element: a consumer that can leave focus on its trigger spreads `anchor_events()`.

use std::cell::RefCell;
use std::rc::Rc;

use dioxus::core::provide_root_context;
use dioxus::core::{Attribute, AttributeValue};
use dioxus::html::{EventHandlerValue, PlatformEventData};
use dioxus::prelude::*;

use crate::{
    hooks::{
        ElementHandle, FocusChange, FocusReturn, FocusWithin, focus_return::use_focus_return,
        use_focus_within,
    },
    platform::{
        ElementApi, KeySubscription, PRESS_MARKER_ATTR, PlatformError, PressSubscription,
        key_taken, keyboard, next_task, press,
    },
};

/// Tells a press inside a box from one outside where focus cannot (the
/// WebView): its elements carry it, see [`press`]. Inert elsewhere.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct PressMarker {
    id: u64,
    live: bool,
}

impl PressMarker {
    /// Spread on every element a press inside counts for. `None` where unused.
    pub(crate) fn attribute(&self) -> Option<Attribute> {
        self.live.then(|| {
            Attribute::new(
                PRESS_MARKER_ATTR,
                AttributeValue::Text(self.id.to_string()),
                None,
                false,
            )
        })
    }
}

/// A marker for a box, made where the trigger renders when that is not the
/// box's scope (`Menu`'s wrapper). Pass it in [`DismissOptions::marker`].
pub(crate) fn use_press_marker() -> PressMarker {
    let stack = use_hook(layer_stack);
    use_hook(move || {
        let mut next = stack.next;
        let id = *next.peek();
        next.set(id + 1);
        PressMarker {
            id,
            live: press().is_some(),
        }
    })
}

/// One open dismissible layer, identified only by when it opened.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct LayerId(u64);

/// Every open dismissible layer in this document, in the order they opened.
/// Root context, not a `thread_local!`: `VirtualDom`s share a thread in tests.
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

/// This layer's place in the stack of open layers. Whoever hears an Escape asks
/// it first, so only the top layer acts, whatever the transport.
///
/// A layer may push only if it hears Escape at least as widely as any layer it
/// may sit above: `use_dismiss` off the web does not (see [`use_dismiss`]).
#[derive(Clone, Copy)]
pub(crate) struct DismissLayer {
    id: LayerId,
    stack: LayerStack,
}

impl DismissLayer {
    /// Puts this layer on top until the guard drops: the only way off, so no
    /// exit path can leak a layer and silently wedge Escape. `write`: a live scope.
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

    /// Whether an Escape press belongs to this layer. `peek`: asked from an
    /// event handler, which must subscribe to nothing.
    pub(crate) fn is_top(&self) -> bool {
        self.stack.open.peek().last() == Some(&self.id)
    }

    /// Whether any dismissible layer is open (`Spotlight` asks before its
    /// hotkey). `peek`, as [`is_top`](Self::is_top).
    pub(crate) fn any_open(&self) -> bool {
        !self.stack.open.peek().is_empty()
    }
}

/// Android's Back runs `onback` while `open` and this is the newest open layer
/// (1275). Nothing elsewhere: the WebView on a desktop has no Back.
pub(crate) fn use_back(open: bool, onback: Callback<()>) {
    #[cfg(target_os = "android")]
    back::use_back(open, onback);
    #[cfg(not(target_os = "android"))]
    let _ = (open, onback);
}

/// On the stack until dropped.
pub(crate) struct LayerGuard {
    id: LayerId,
    stack: LayerStack,
}

impl Drop for LayerGuard {
    /// By identity, not a pop: an older layer may close first. Tolerates a dead
    /// signal, since it also runs in the dom's teardown.
    fn drop(&mut self) {
        let mut open = self.stack.open;
        if let Ok(mut layers) = open.try_write() {
            layers.retain(|id| *id != self.id);
        }
    }
}

/// Whether an element-level Escape closes the overlay: not a repeat, not
/// mid-composition, not [`key_taken`] inside (the document transport uses [`use_field_list_layer`]).
pub(crate) fn escape_closes(event: &Event<KeyboardData>) -> bool {
    event.key() == Key::Escape
        && !event.is_auto_repeating()
        && !event.is_composing()
        && !key_taken(event)
}

/// A layer id for this component: push it with [`DismissLayer::push`] while open.
/// [`use_dismiss`] does it for you; `Modal` calls it directly.
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

/// Puts a field's open list on the Escape stack where [`keyboard()`] answers,
/// so one Escape closes the list, not the card around it too (todo 348).
///
/// The field must close its list when focus leaves, or the list wedges Escape
/// on top. Every field does; the public `Combobox` leaves it to its caller.
/// Android's Back runs `onback`, which closes the list as Escape does.
pub(crate) fn use_field_list_layer(open: bool, onback: Callback<()>) {
    use_back(open, onback);
    let layer = use_dismiss_layer();
    // Decided once, as in `use_dismiss`: a property of the renderer.
    let global = use_hook(|| keyboard().is_some());
    let guard: Rc<RefCell<Option<LayerGuard>>> = use_hook(|| Rc::new(RefCell::new(None)));

    let slot = guard.clone();
    use_effect(use_reactive!(|(open,)| {
        if !open || !global {
            slot.borrow_mut().take();
        } else if slot.borrow().is_none() {
            *slot.borrow_mut() = Some(layer.push());
        }
    }));
    use_drop(move || {
        guard.borrow_mut().take();
    });
}

/// Why a box is closing, which is what decides whether focus goes back.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Dismissal {
    /// Escape heard by our own element handler (focus is inside), or an item
    /// chosen (APG: back to the trigger). Focus goes back either way.
    FromInside,
    /// Escape heard at the document, through [`KeyboardApi`](crate::platform::KeyboardApi).
    /// Focus goes back only if it was inside.
    FromDocument,
    /// Focus left the box on its own (a click elsewhere), and stays where it went.
    FocusMoved,
}

/// Which dismissal behaviours a consumer wants, and what focus should do.
/// Nothing reactive: `open` and `placed` are hook parameters.
#[derive(Clone, PartialEq)]
pub(crate) struct DismissOptions {
    /// Escape closes the box, if it is the top layer.
    pub escape: bool,
    /// Focus leaving the box closes it. Inert on the mounted floor, which cannot
    /// tell focus is outside (todo 46); Blitz's Tab goes via [`use_focus_within`].
    pub outside: bool,
    /// A deliberate close hands focus back to whatever opened the box.
    pub return_focus: bool,
    /// Focused once the box is open *and placed*: `focus()` on the unplaced,
    /// hidden box returns `Ok(())` and does nothing.
    pub initial_focus: Option<ElementHandle>,
    /// Called instead of `onclose` for [`Dismissal::FocusMoved`] (a submenu
    /// decides by where focus went). `None` calls `onclose`.
    pub onfocusmoved: Option<Callback<()>>,
    /// With `outside`, a press outside closes it where focus cannot tell
    /// ([`press`]). Off for a box inside another that already listens (a submenu).
    pub press: bool,
    /// The trigger's marker, when it renders outside this scope. `None` makes one.
    pub marker: Option<PressMarker>,
}

impl Default for DismissOptions {
    fn default() -> Self {
        Self {
            escape: true,
            outside: true,
            return_focus: true,
            initial_focus: None,
            onfocusmoved: None,
            press: true,
            marker: None,
        }
    }
}

/// Counts one more element as inside until dropped, from
/// [`DismissHandle::register_inside_box`].
pub(crate) struct InsideGuard {
    id: u64,
    inside: Signal<Vec<Inside>>,
}

/// One element counted inside: its registration id, and its box's marker.
type Inside = (u64, ElementHandle, u64);

impl Drop for InsideGuard {
    fn drop(&mut self) {
        let mut inside = self.inside;
        if let Ok(mut elements) = inside.try_write() {
            elements.retain(|(id, ..)| *id != self.id);
        }
    }
}

/// What a consumer spreads on its floating box, plus the handful of calls it
/// makes from its own handlers.
#[derive(Clone, Copy)]
pub(crate) struct DismissHandle {
    anchor: ElementHandle,
    floating: ElementHandle,
    /// For [`anchor_events`](DismissHandle::anchor_events): the trigger
    /// outlives the box.
    open: bool,
    onclose: Option<Callback<()>>,
    focus_return: FocusReturn,
    escape: bool,
    outside: bool,
    return_focus: bool,
    /// Whether this layer hears Escape at the document, which is the same
    /// question as whether it is allowed on the stack.
    global: bool,
    inside: Signal<Vec<Inside>>,
    inside_next: Signal<u64>,
    onfocusmoved: Option<Callback<()>>,
    focus: FocusWithin,
    /// Bumped when a silent focus move, or a press, left the box.
    left_tick: Signal<u64>,
    marker: PressMarker,
}

impl DismissHandle {
    /// The focus-return contract for this box: `remember_active()` in the
    /// trigger's handler, `fallback()` for a trigger that may be deleted (todo 37).
    pub(crate) fn focus_return(&self) -> FocusReturn {
        self.focus_return
    }

    /// Counts `child`'s box *inside*, its marker too, until the guard drops: a
    /// portaled submenu is no descendant of its menu.
    #[must_use = "the box stops counting as inside when the guard is dropped"]
    pub(crate) fn register_inside_box(&self, child: &DismissHandle) -> InsideGuard {
        // A counter: one handle may be registered twice (a reopened submenu).
        // Nothing subscribes: this writes during a render, a subscriber would loop.
        let mut next = self.inside_next;
        let id = *next.peek();
        next.set(id + 1);

        let mut inside = self.inside;
        if let Ok(mut elements) = inside.try_write() {
            elements.push((id, child.floating, child.marker.id));
        }
        InsideGuard {
            id,
            inside: self.inside,
        }
    }

    /// Whether a press carrying `markers` landed in this box or one inside it.
    fn pressed_inside(&self, markers: &[u64]) -> bool {
        let inside = self.inside.peek();
        let mut own =
            std::iter::once(self.marker.id).chain(inside.iter().map(|(_, _, marker)| *marker));
        own.any(|marker| markers.contains(&marker))
    }

    /// Closes deliberately, handing focus back, for a chosen item. See
    /// [`Dismissal::FromInside`].
    pub(crate) fn dismiss(&self) {
        self.close(Dismissal::FromInside);
    }

    /// The Escape handler for the **trigger** where there is no document
    /// transport; without it focus there cannot dismiss the box (SC 1.4.13).
    ///
    /// Empty while closed, or its `stop_propagation` would swallow Escape for
    /// whatever *is* open (a `Modal` around it).
    pub(crate) fn anchor_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();
        if self.open && self.escape && !self.global {
            events.push(self.escape_listener());
        }
        // A press on the trigger is its own toggle, not one outside.
        if self.open && self.outside {
            events.extend(self.marker.attribute());
        }
        events
    }

    /// For the floating box: `onfocusout` for the outside check, and `onkeydown`
    /// for Escape only where there is no document transport.
    ///
    /// Ungated because a box renders only while open. A box kept mounted on close
    /// (an exit via [`use_presence`](super::use_presence)) must stop spreading it.
    pub(crate) fn floating_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();

        if self.escape && !self.global {
            events.push(self.escape_listener());
        }

        if self.outside {
            events.push(self.focusout_listener());
            events.extend(self.marker.attribute());
        }

        events
    }

    /// Closes once focus has left the anchor, the box and every registered
    /// element. `use_popover` also puts it on the anchor.
    pub(crate) fn focusout_listener(&self) -> Attribute {
        listener("onfocusout", self.focus.focusout(0))
    }

    /// Focus left an element counted inside: closes if it left them all.
    fn focus_changed(&self, change: FocusChange) {
        match change.in_group {
            _ if change.within => {}
            Some(true) => {}
            // Landed already, heard at `Outlet`'s flush: recorded for the
            // effect in `use_dismiss`, as Escape is.
            Some(false) => {
                let mut tick = self.left_tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
            None => {
                // The web lands focus after `focusout`: ask a task later. Blitz
                // moves focus first and cannot answer from a task: ask here.
                let early = self.focus_inside();
                let handle = *self;
                spawn(async move {
                    next_task().await;
                    if handle.focus_inside().or(early) == Some(false) {
                        handle.close(Dismissal::FocusMoved);
                    }
                });
            }
        }
    }

    /// The Escape listener both surfaces share.
    fn escape_listener(&self) -> Attribute {
        let handle = *self;
        escape_listener(move || handle.close(Dismissal::FromInside))
    }

    /// Whether focus is still inside this box; `false` also where the platform
    /// cannot tell, which [`focus_inside`](Self::focus_inside) keeps apart.
    pub(crate) fn holds_focus(&self) -> bool {
        self.focus_inside() == Some(true)
    }

    /// `None` when a mounted element cannot answer and none said yes. Asks
    /// `:focus` for a descendant and `is_focused()` for the element itself.
    fn focus_inside(&self) -> Option<bool> {
        let inside = self.inside.peek();
        let elements = [&self.anchor, &self.floating]
            .into_iter()
            .chain(inside.iter().map(|(_, element, _)| element));
        focus_inside_of(elements.map(|element| {
            if element.mounted().is_none() {
                return Some(false);
            }
            match element.query_selector(":focus") {
                Ok(_) => Some(true),
                _ if element.is_focused() => Some(true),
                Err(PlatformError::Unsupported) => None,
                Err(_) => Some(false),
            }
        }))
    }

    /// Asks the consumer to close a task later (a synchronous close panics), then
    /// restores focus if `reason` says so, checked now while the box is mounted.
    ///
    /// Never pops the layer: the press that closed it is still bubbling, and an
    /// enclosing `Modal` would close too. The `open` effect's guard owns membership.
    fn close(&self, reason: Dismissal) {
        let restore = self.return_focus
            && match reason {
                Dismissal::FromInside => true,
                Dismissal::FromDocument => self.holds_focus(),
                Dismissal::FocusMoved => false,
            };

        let handle = *self;
        spawn(async move {
            let onclose = match reason {
                Dismissal::FocusMoved => handle.onfocusmoved.or(handle.onclose),
                _ => handle.onclose,
            };
            if let Some(onclose) = onclose {
                onclose.call(());
            }
            if restore {
                handle.focus_return.restore();
            }
        });
    }
}

/// Closes `floating` on Escape or focus leaving it, through `onclose`, and
/// hands focus back. Owns no open state.
///
/// `placed` is a parameter, not an option, so it cannot default into focusing
/// the hidden, unplaced box. A consumer with nothing to place passes `true`.
///
/// It joins the Escape stack only where [`keyboard()`](crate::platform::keyboard)
/// answers; elsewhere spread both `anchor_events` and `floating_events`.
pub(crate) fn use_dismiss(
    anchor: ElementHandle,
    floating: ElementHandle,
    open: bool,
    placed: bool,
    onclose: Option<Callback<()>>,
    options: DismissOptions,
) -> DismissHandle {
    let focus_return = use_focus_return();
    let inside = use_signal(Vec::<Inside>::new);
    let inside_next = use_signal(|| 0u64);
    let global = use_global_escape();
    let focus = use_focus_within(Vec::new, |_| {});
    let left_tick = use_signal(|| 0u64);
    let own_marker = use_press_marker();

    let handle = DismissHandle {
        anchor,
        floating,
        open,
        onclose,
        focus_return,
        escape: options.escape,
        outside: options.outside,
        return_focus: options.return_focus,
        global,
        inside,
        inside_next,
        onfocusmoved: options.onfocusmoved,
        focus,
        left_tick,
        marker: options.marker.unwrap_or(own_marker),
    };

    use_document_escape(open && options.escape, global, move || {
        handle.close(Dismissal::FromDocument)
    });
    let onback = use_callback(move |()| handle.close(Dismissal::FromInside));
    use_back(open && options.escape, onback);

    // Empty while closed, so a move then reports nothing.
    let watched = open && options.outside;
    use_press_outside(watched && options.press, handle);
    focus.watch(
        move || match watched {
            true => [anchor, floating]
                .into_iter()
                .chain(inside.peek().iter().map(|(_, element, _)| *element))
                .map(|element| element.mounted())
                .collect(),
            false => Vec::new(),
        },
        move |change| handle.focus_changed(change),
    );
    let mut left_seen = use_signal(|| 0u64);
    use_effect(move || {
        let tick = left_tick();
        if tick == *left_seen.peek() {
            return;
        }
        left_seen.set(tick);
        handle.close(Dismissal::FocusMoved);
    });

    // Armed on opening, consumed once focus lands: a reopen focuses again, a
    // re-placement on scroll does not. Plain cells: every consumer pays for it.
    let entering = use_hook(|| Rc::new(std::cell::Cell::new((false, false))));

    let initial_focus = options.initial_focus;
    use_effect(use_reactive!(|(open, placed)| {
        // Read first, branch second - this subscribes the effect to the box
        // mounting, and it re-runs on a *re*-mount, which is every reopen.
        let mounted = floating.mount_token().is_some();
        let (was_open, armed) = entering.get();
        let armed = if open != was_open { open } else { armed };
        entering.set((open, armed));
        if !open || !placed || !mounted || !armed {
            return;
        }
        entering.set((open, false));
        if let Some(target) = initial_focus {
            let _ = target.focus();
        }
    }));

    handle
}

/// Closes `handle` on a press outside it while `listen`, where [`press`] answers.
/// As a focus move: the press put focus where it wanted.
fn use_press_outside(listen: bool, handle: DismissHandle) {
    let listening: Rc<RefCell<Option<Box<dyn PressSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let listening = listening.clone();
        move || {
            listening.borrow_mut().take();
        }
    });
    use_effect(use_reactive!(|(listen,)| {
        if !listen {
            listening.borrow_mut().take();
            return;
        }
        if listening.borrow().is_some() {
            return;
        }
        let Some(api) = press() else {
            return;
        };
        // Records the press only, outside every scope; `use_dismiss`'s effect closes.
        let subscription = api.on_press(Box::new(move |markers| {
            if !handle.pressed_inside(&markers) {
                let mut tick = handle.left_tick;
                let next = tick.peek().wrapping_add(1);
                tick.set(next);
            }
        }));
        *listening.borrow_mut() = Some(subscription);
    }));
}

/// Whether the document can be listened to, decided once at render time. Safe:
/// it needs no mounted element, and SSR serialises no listeners.
fn use_global_escape() -> bool {
    use_hook(|| keyboard().is_some())
}

/// The document-level Escape transport and this layer's stack place, while
/// `listen` and `global` hold. `onescape` runs in this scope, once per press.
fn use_document_escape(listen: bool, global: bool, mut onescape: impl FnMut() + 'static) {
    let layer = use_dismiss_layer();

    // Subscription and stack membership share one slot and one lifetime, so a
    // leaked layer is impossible.
    type Listening = Rc<RefCell<Option<(Box<dyn KeySubscription>, LayerGuard)>>>;
    let listening: Listening = use_hook(|| Rc::new(RefCell::new(None)));

    // Bumped from the key callback, outside every scope: it only records the
    // press, the effect below acts.
    let escape_tick = use_signal(|| 0u64);
    use_drop({
        let listening = listening.clone();
        move || {
            listening.borrow_mut().take();
        }
    });

    // Acts on the recorded press, then syncs the subscription with `listen`.
    // `seen` keeps a reopen from closing on a press from an earlier open.
    let seen = use_hook(|| Rc::new(std::cell::Cell::new(0u64)));
    let slot = listening.clone();
    use_effect(use_reactive!(|(listen,)| {
        let tick = escape_tick();
        if tick != seen.replace(tick) {
            onescape();
        }
        if !listen || !global {
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
}

/// What an Escape-only box spreads on itself. See [`use_escape_dismiss`].
#[derive(Clone, Copy)]
pub(crate) struct EscapeDismiss {
    onclose: Callback<()>,
    /// Escape is heard on the box itself: on, and no document transport.
    local: bool,
}

impl EscapeDismiss {
    /// [`DismissHandle::floating_events`] without the focus-out check: the
    /// Escape listener where there is no document-level transport, else none.
    pub(crate) fn floating_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();
        if self.local {
            let onclose = self.onclose;
            events.push(escape_listener(move || {
                spawn(async move { onclose.call(()) });
            }));
        }
        events
    }
}

/// [`use_dismiss`] for a box focus never enters (a tooltip): Escape only.
/// `onclose` runs a task later; the trigger keeps its own Escape off the web.
pub(crate) fn use_escape_dismiss(open: bool, escape: bool, onclose: Callback<()>) -> EscapeDismiss {
    let global = use_global_escape();
    use_document_escape(open && escape, global, move || {
        spawn(async move { onclose.call(()) });
    });
    EscapeDismiss {
        onclose,
        local: escape && !global,
    }
}

/// The element-level Escape listener of [`DismissHandle`] and
/// [`EscapeDismiss`]; `close` runs once the press is taken.
fn escape_listener(mut close: impl FnMut() + 'static) -> Attribute {
    listener("onkeydown", move |event: Event<KeyboardData>| {
        // No repeat, no composition (untested: headless Chromium has no IME),
        // no press a field inside took (todo 348).
        if !escape_closes(&event) {
            return;
        }
        // No stack check: this transport never pushed. A bubble-phase stop,
        // so an enclosing `Modal` does not close on the same press.
        event.prevent_default();
        event.stop_propagation();
        close();
    })
}

/// Inside if any element says so, unknown if any cannot answer, else outside.
fn focus_inside_of(answers: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let mut known = true;
    for answer in answers {
        match answer {
            Some(true) => return Some(true),
            Some(false) => {}
            None => known = false,
        }
    }
    known.then_some(false)
}

/// One event attribute, built by hand. Through `EventHandlerValue`: a bare
/// `AttributeValue::listener::<T>` type-checks and panics on the first event.
pub(crate) fn listener<T>(name: &'static str, handler: impl FnMut(Event<T>) + 'static) -> Attribute
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

#[cfg(target_os = "android")]
mod back;

/// The Escape arbitration and layer lifetime, against a real `VirtualDom`. The
/// web arm is untestable here: `keyboard()` is `None` off wasm.
#[cfg(test)]
mod tests;
