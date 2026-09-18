//! Dismissal for a floating box: Escape, focus leaving it, and handing focus
//! back - written once, instead of once per consumer.
//!
//! [`use_popover`](super::use_popover) places a box and owns no open state.
//! This is the other half: *when* a box should close. The two are separate
//! hooks because a tooltip wants dismissal without placement, and because
//! folding policy into the placement hook would contradict the promise in its
//! own doc comment.
//!
//! It renders nothing. [`DismissHandle::floating_events`] and
//! [`DismissHandle::anchor_events`] hand the consumer attributes to spread on
//! its own box and trigger, and the consumer keeps whatever role, theming and
//! ARIA its popup type needs.
//!
//! **Without [`keyboard()`], Escape reaches only the element that has focus**
//! (the WebView floor). A consumer that can leave focus on its trigger - a
//! combobox-shaped dropdown, any pointer-opened surface - has to spread
//! `anchor_events()` too, or the surface cannot be dismissed from the keyboard
//! there.

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
    platform::{ElementApi, KeySubscription, PlatformError, key_taken, keyboard, next_task},
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
/// **A layer may push only if it hears Escape at least as widely as any layer it
/// may sit above.** The condition is relative, not absolute: `Modal` qualifies
/// with nothing but a bubbled subtree `onkeydown`, because it is the bottom
/// layer and the fallback, so there is nothing beneath it whose presses it
/// could swallow. `use_dismiss` off the web does not qualify - it would sit
/// above a `Modal` while hearing strictly less than the `Modal` does, and a
/// pointer-opened box leaves focus outside itself, so the press it silenced by
/// being top is one its own handler never receives. See [`use_dismiss`].
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

    /// Whether any dismissible layer is open - a surface already sits above
    /// the page. `Spotlight` asks it before its hotkey opens a palette over
    /// one. `peek`, for the same reason as [`is_top`](Self::is_top).
    pub(crate) fn any_open(&self) -> bool {
        !self.stack.open.peek().is_empty()
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

/// Whether an overlay's element-level Escape transport should close it on
/// this press: `Modal`, `FloatingWindow`, and the element listener in
/// [`use_dismiss`] all ask it. A held Escape is one intent, so its
/// auto-repeats do not walk on to the next layer out, and Escape
/// mid-composition cancels the composition. And a press something inside
/// already took ([`key_taken`]) is not this overlay's.
///
/// `use_dismiss`'s document transport cannot ask the last question: it hears
/// the press in the capture phase, before any field has had it. There an open
/// field list is a layer of its own instead ([`use_field_list_layer`]).
pub(crate) fn escape_closes(event: &Event<KeyboardData>) -> bool {
    event.key() == Key::Escape
        && !event.is_auto_repeating()
        && !event.is_composing()
        && !key_taken(event)
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

/// Puts a field's open list on the Escape stack, **where [`keyboard()`]
/// answers** (the web, Blitz), for as long as `open` holds. `ComboboxCore`, `Cascader`, `ColorField` and the date
/// picker fields call it; each keeps its own Escape handler.
///
/// On the web a `use_dismiss` box hears Escape in the capture phase at the
/// document, before the field inside it has had the press, so it cannot see
/// the field take it. Without this a `Select` open in a `HoverCard` closed
/// its list and the card on one Escape (todo 348). With the list on top, the
/// card declines as not-top and the field's own handler closes the list; the
/// next Escape reaches the card. It also covers focus inside a portaled
/// dropdown - a date picker's calendar - which no query of the card's own
/// subtree could see.
///
/// Blitz hears the document transport last, in bubble phase, where the list's
/// layer declines the press for the card just the same. Without `keyboard()`
/// it does not push, by the rule in [`DismissLayer`]: every Escape transport
/// is an element handler, the field's runs first, and [`escape_closes`] reads
/// the default it prevented.
///
/// **The field has to close its list when focus leaves it.** An open list
/// with focus elsewhere hears no Escape, since its handlers are on the field
/// and the dropdown, yet it sits on top: every layer under it declines as
/// not-top, and Escape does nothing until the list closes. It cannot happen
/// with today's fields, because each one closes on focus leaving: `Select`
/// (its trigger, or its search box while searchable), `Autocomplete`,
/// `TagsField` and `PhoneField` on blur, `Cascader` and `ColorField` on blur,
/// the date pickers once focus is in neither the input nor the calendar.
/// The one exception is the public `Combobox`: its caller owns the
/// `use_combobox` state and wires the input, so closing on blur is the
/// caller's job, as it already is for Tab and every other close.
pub(crate) fn use_field_list_layer(open: bool) {
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
    /// Escape heard by one of *our own* element handlers, or the consumer
    /// telling us an item was chosen. Focus goes back either way, for two
    /// different reasons.
    ///
    /// For Escape it is evidence: an element handler only fires when focus is
    /// inside the box or on the trigger, so hearing the press that way *is* the
    /// proof, and no predicate is needed.
    ///
    /// For a selection it is not. Clicking a non-focusable item moves focus
    /// nowhere, so focus may well be outside. Focus goes back because the box
    /// is going away and the trigger is where the user was - which is what APG
    /// asks for after a menu item is activated - not because focus is provably
    /// inside.
    FromInside,
    /// Escape heard at the document, through
    /// [`KeyboardApi`](crate::platform::KeyboardApi). The press carries no
    /// information about where focus is, and for a pointer-opened box it is
    /// usually somewhere else entirely - where the user is actually working.
    /// Focus goes back only if it was inside.
    FromDocument,
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
    /// **Inert on the mounted floor.** A focusout closes the box only where the
    /// platform can say focus is outside: the web and Blitz can, the mounted
    /// floor answers `query_selector` `Unsupported` (todo 46). Blitz fires no
    /// focus event for Tab; [`use_focus_within`] reports that move instead. Escape and
    /// [`DismissHandle::dismiss`] close everywhere.
    pub outside: bool,
    /// A deliberate close hands focus back to whatever opened the box.
    pub return_focus: bool,
    /// Focused once the box is open *and placed* - never on mount. `focus()`
    /// returns `Ok(())` on the `visibility: hidden` box a popover renders
    /// before it has been measured, and does nothing.
    pub initial_focus: Option<ElementHandle>,
    /// Called in place of `onclose` when focus left the box on its own
    /// ([`Dismissal::FocusMoved`]). `None` calls `onclose`, as for every other
    /// close. A submenu uses it: where focus went decides whether only it
    /// closes or the whole menu does, and only this close knows focus left.
    pub onfocusmoved: Option<Callback<()>>,
}

impl Default for DismissOptions {
    fn default() -> Self {
        Self {
            escape: true,
            outside: true,
            return_focus: true,
            initial_focus: None,
            onfocusmoved: None,
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
    /// Needed because [`anchor_events`](DismissHandle::anchor_events) sits on
    /// an element that exists whether the box is open or not. The floating box
    /// asks nobody because it is simply not rendered when closed.
    open: bool,
    onclose: Option<Callback<()>>,
    focus_return: FocusReturn,
    escape: bool,
    outside: bool,
    return_focus: bool,
    /// Whether this layer hears Escape at the document, which is the same
    /// question as whether it is allowed on the stack.
    global: bool,
    inside: Signal<Vec<(u64, ElementHandle)>>,
    inside_next: Signal<u64>,
    onfocusmoved: Option<Callback<()>>,
    focus: FocusWithin,
    /// Bumped when a silent focus move left the box.
    left_tick: Signal<u64>,
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
        //
        // **Nothing subscribes to either signal**, and nothing should: this is
        // called from a `use_hook`, so it writes during a render, and a
        // subscriber would turn that write into a re-render loop. `holds_focus`
        // reads `inside` with `peek` for the same reason. The `also_inside`
        // field this replaced carried the same note.
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
    /// chosen. Focus goes back because the box is about to disappear and the
    /// trigger is where the user was, not because focus is provably inside:
    /// clicking a non-focusable item moves focus nowhere. See
    /// [`Dismissal::FromInside`].
    pub(crate) fn dismiss(&self) {
        self.close(Dismissal::FromInside);
    }

    /// The Escape handler for the consumer's **trigger**, where there is no
    /// document-level transport. Empty where there is one.
    ///
    /// Spread this on the anchor whenever the consumer can leave focus there,
    /// which is most of them: a combobox-shaped dropdown keeps focus on its
    /// input, and any pointer-opened surface never moves focus at all. Off the
    /// web the only transport is an element handler, and
    /// [`floating_events`](Self::floating_events) sits on a box the trigger is
    /// not inside - the box is portaled to the document root - so without this
    /// Escape reaches nothing and the surface cannot be dismissed from the
    /// keyboard at all. That is a plain SC 1.4.13 failure on a shipped target.
    ///
    /// It is the same listener as the box's, deliberately: whichever element
    /// has focus hears the press, stops it, and closes. They cannot both fire,
    /// because the box is not a descendant of the trigger, and where it is the
    /// box's own `stop_propagation` settles it.
    ///
    /// **Empty while the box is closed.** The trigger outlives the box, so
    /// unlike [`floating_events`](Self::floating_events) this listener would
    /// otherwise stay live with nothing open behind it - and its
    /// `stop_propagation` would swallow Escape for whatever *is* open. A closed
    /// popover inside a `Modal`, with focus on its trigger, would make Escape
    /// dead for as long as focus sat there: a layer that is not even open
    /// wedging one that is, which is the thing the layer stack exists to
    /// prevent arriving through a third door.
    pub(crate) fn anchor_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();
        if self.open && self.escape && !self.global {
            events.push(self.escape_listener());
        }
        events
    }

    /// The attributes for the consumer's floating box: `onfocusout` for the
    /// outside check, and `onkeydown` for Escape **only where there is no
    /// document-level transport**. Where there is one, this box would be the
    /// second handler for the same press.
    ///
    /// This covers focus *inside the box*. Focus on the trigger is
    /// [`anchor_events`](Self::anchor_events), and a consumer that can leave
    /// focus there needs both.
    ///
    /// **Ungated, unlike `anchor_events`, and only because the box is rendered
    /// only while open.** That is a property of the consumer, not of this hook:
    /// `keepMounted` was declined, so nothing keeps a closed box in the tree
    /// today. A consumer that starts keeping it mounted - an exit transition
    /// through [`use_presence`](super::use_presence) holds the subtree for the
    /// whole animation - has to stop spreading this first, or a box the user
    /// has already dismissed keeps a live Escape handler whose
    /// `stop_propagation` eats the press for the `Modal` behind it. Exactly the
    /// defect `anchor_events` guards against, one door over, and it lands in
    /// the same window as the accessibility-tree obligation: between close and
    /// unmount the box is still mounted, still tabbable, still announced, and
    /// would now still be eating Escape. Raised by Bob3 reviewing D1.
    pub(crate) fn floating_events(&self) -> Vec<Attribute> {
        let mut events = Vec::new();

        if self.escape && !self.global {
            events.push(self.escape_listener());
        }

        if self.outside {
            events.push(self.focusout_listener());
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
                // On the web focus lands after `focusout`, so the answer is
                // taken after the next task. Blitz moves focus first and cannot
                // answer from a task, so its answer is the one taken here.
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

    /// Whether focus is still somewhere that counts as inside this box. `false`
    /// also where the platform cannot tell; [`focus_inside`](Self::focus_inside)
    /// keeps that case apart.
    pub(crate) fn holds_focus(&self) -> bool {
        self.focus_inside() == Some(true)
    }

    /// `None` when a mounted element cannot answer and none said yes.
    ///
    /// Each handle is asked twice: `query_selector(":focus")` finds a focused
    /// *descendant*, and `is_focused()` catches the element itself - which is
    /// the common case for an anchor, since a trigger is usually the focusable
    /// element rather than a wrapper around one. An unmounted handle has no
    /// node, so it answers `false` rather than unknown.
    fn focus_inside(&self) -> Option<bool> {
        let inside = self.inside.peek();
        let elements = [&self.anchor, &self.floating]
            .into_iter()
            .chain(inside.iter().map(|(_, element)| element));
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
    ///
    /// **Whether to restore is decided by how we heard about the close**, and
    /// the check is taken here, synchronously, while the box is still mounted
    /// and the platform can still answer for it.
    ///
    /// An element handler only fires when focus is inside the box or on the
    /// trigger, so hearing it that way *is* the evidence and no check is
    /// needed. The document listener carries no such evidence: for a
    /// pointer-opened box focus is usually where the user is working, and
    /// yanking it to the trigger on Escape is the same defect as yanking it on
    /// an outside click, arriving through the other door. So that path asks.
    ///
    /// Asking only on the path that needs it also keeps the answer honest.
    /// `holds_focus()` cannot tell off the web, and the document listener
    /// exists only on the web - so the predicate is only ever consulted where
    /// it works.
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
/// replaced. So off the web this hook does not push and the `Modal` stays top,
/// and **the `Modal`'s** Escape behaves exactly as it does today.
///
/// That is a statement about the `Modal`, not about this box. Off the web this
/// box hears Escape only through an element handler, so the consumer has to put
/// one where focus actually is: [`DismissHandle::anchor_events`] on the
/// trigger as well as [`DismissHandle::floating_events`] on the box.
///
/// ```ignore
/// # // Not compiled: `use_dismiss` is crate-internal, so a doc-test cannot name it.
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
    let inside = use_signal(Vec::<(u64, ElementHandle)>::new);
    let inside_next = use_signal(|| 0u64);
    let global = use_global_escape();
    let focus = use_focus_within(Vec::new, |_| {});
    let left_tick = use_signal(|| 0u64);

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
    };

    use_document_escape(open && options.escape, global, move || {
        handle.close(Dismissal::FromDocument)
    });

    // Empty while closed, so a move then reports nothing.
    let watched = open && options.outside;
    focus.watch(
        move || match watched {
            true => [anchor, floating]
                .into_iter()
                .chain(inside.peek().iter().map(|(_, element)| *element))
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

    // Armed on the opening edge and consumed once the focus lands, so reopening
    // focuses again and a re-placement on scroll does not. Plain cells in one
    // effect: every consumer pays for this hook, and none but a test focuses.
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

/// Decided once. Whether the document can be listened to is a property of
/// the running renderer, not of this render.
///
/// This is the first place in the codebase to read a platform capability at
/// render time, so: [[codebase/platform-api]]'s "ask in an effect" rule is
/// about *reads that need a mounted element*, and this needs none. It is
/// also safe across hydration, which is the reason the rule looks like it
/// should apply - SSR serialises no listeners, so the server and the client
/// emit identical HTML whichever way this answers, and hydration walks nodes
/// rather than attributes. Moving it into an effect costs a render and buys
/// nothing anyone has been able to name.
fn use_global_escape() -> bool {
    use_hook(|| keyboard().is_some())
}

/// The document-level Escape transport and this layer's place on the stack,
/// for as long as `listen` holds and `global` says the document can be heard.
/// `onescape` runs in this scope, once per press this layer took.
fn use_document_escape(listen: bool, global: bool, mut onescape: impl FnMut() + 'static) {
    let layer = use_dismiss_layer();

    // The document subscription and the stack membership have exactly the same
    // lifetime, so they are one slot: taking it drops both, and dropping the
    // hook's own state on unmount drops it too. That is what makes a leaked
    // layer impossible rather than merely unlikely.
    type Listening = Rc<RefCell<Option<(Box<dyn KeySubscription>, LayerGuard)>>>;
    let listening: Listening = use_hook(|| Rc::new(RefCell::new(None)));

    // Bumped from the key callback, which on the web runs outside every dioxus
    // scope. The callback deliberately does not close anything: it only records
    // the press, and the effect below acts, in the runtime.
    let escape_tick = use_signal(|| 0u64);
    use_drop({
        let listening = listening.clone();
        move || {
            listening.borrow_mut().take();
        }
    });

    // Acts on what the key callback recorded, then keeps the subscription in
    // step with `listen`: one effect, as every open box pays for it. Comparing
    // against what this scope has already seen keeps a reopen from closing on
    // a press from three opens ago; a plain cell, only this effect reads it.
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

/// [`use_dismiss`] for a box focus never enters: Escape only, no focus-out
/// check, no focus to hand back, no initial focus. A tooltip opens by the
/// dozen, so it skips those hooks instead of switching them off.
///
/// `onclose` runs a task later, as in [`use_dismiss`]. The consumer's trigger
/// keeps its own Escape listener where the document cannot be heard.
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
        // `escape_closes`, the one rule every overlay's own Escape
        // transport shares. Three of its four filters matter here.
        //
        // A held Escape is one intent, not a stream of them. Without
        // the repeat filter it walks down the stack, closing the menu
        // and then the modal behind it inside one press. A held
        // ArrowDown scrolling a list still wants every repeat, so this
        // is not a global filter.
        //
        // Mid-composition, Escape means "cancel the composition", not
        // "dismiss". `KeyboardApi` drops a composing press on both its
        // paths ahead of the filter ([[codebase/platform-api]]); this
        // is the same guard on the element path, so the contract does
        // not change with the transport. Reasoned rather than measured
        // there and here alike - headless Chromium has no IME.
        //
        // A press a field inside took is not this box's (todo 348). A
        // `Select` in a `HoverCard` closes its list, prevents the
        // default and lets the press bubble on, so without this one
        // Escape closed the list and the card around it.
        if !escape_closes(&event) {
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
        // ran first. `Menu`'s trigger stops its Escape the same way.
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
mod tests;
