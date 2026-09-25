use std::rc::Rc;

use dioxus::prelude::*;

use crate::{
    components::layout::BoxStyle,
    hooks::ElementHandle,
    platform::{ElementApi, nested_interactive, next_task},
};

/// A checkable control's activation, taken off the browser. A cancelled click
/// restores the old `checked` after dioxus re-rendered inside the dispatch, so
/// label clicks are cancelled and Space is answered on `keydown`.
#[derive(Clone)]
pub(crate) struct Activation {
    /// Attached to the input by [`wire`](Self::wire), and focused by the label.
    pub(super) element: Option<ElementHandle>,
    /// Focuses the input instead, for a control with no handle per input.
    pub(super) focus: Option<Rc<dyn Fn()>>,
    pub(super) activate: Rc<dyn Fn()>,
    pub(super) enter: bool,
    /// Inside a card, whose click activates too: inner clicks stop, or they activate twice.
    pub(super) card: bool,
}

impl Activation {
    pub(crate) fn new(element: ElementHandle, activate: impl Fn() + 'static) -> Self {
        Self {
            element: Some(element),
            focus: None,
            activate: Rc::new(activate),
            enter: false,
            card: false,
        }
    }

    /// For inputs rendered in a loop, which cannot each take a handle.
    pub(crate) fn focusing(focus: impl Fn() + 'static, activate: impl Fn() + 'static) -> Self {
        Self {
            element: None,
            focus: Some(Rc::new(focus)),
            activate: Rc::new(activate),
            enter: false,
            card: false,
        }
    }

    /// Enter activates as well as Space, outside any form (todos 648, 660).
    pub(crate) fn enter_activates(mut self, enter: bool) -> Self {
        self.enter = enter;
        self
    }

    fn focus(&self) {
        match (&self.focus, &self.element) {
            (Some(focus), _) => focus(),
            (None, Some(element)) => {
                let _ = element.focus();
            }
            (None, None) => {}
        }
    }

    /// The label's half: the click is cancelled, so the input's focus is
    /// given back by hand.
    pub(crate) fn label_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let activation = self.clone();
        move |event| {
            if nested_interactive(&event, CLICK_BOUNDARY) {
                return;
            }
            event.prevent_default();
            if activation.card {
                event.stop_propagation();
            }
            (activation.activate)();
            activation.focus();
        }
    }

    /// A card's half: a click the label and input did not take.
    pub(super) fn card_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let activation = self.clone();
        move |event| {
            if nested_interactive(&event, CLICK_BOUNDARY) {
                return;
            }
            (activation.activate)();
            activation.focus();
        }
    }

    /// A click on the control's padding. Blitz places a label's `::after`
    /// against the label, not the positioned root (todo 941).
    pub(crate) fn padding_click(
        &self,
        boundary: &'static str,
    ) -> impl FnMut(Event<MouseData>) + 'static {
        let activation = self.clone();
        move |event| {
            if !event.default_action_enabled() || nested_interactive(&event, boundary) {
                return;
            }
            (activation.activate)();
            activation.focus();
        }
    }

    /// The input's half: attaches the element, and takes Space and whatever
    /// click still reaches it.
    pub(crate) fn wire(&self, control: BoxStyle) -> BoxStyle {
        let keydown = self.clone();
        let control = match &self.element {
            Some(element) => control.element(element),
            None => control,
        };
        control
            .event("onclick", self.input_click())
            .event("oninput", self.input_input())
            .event("onkeydown", move |event: Event<KeyboardData>| {
                keydown.keydown(&event);
            })
            .event("onkeyup", self.input_keyup())
    }

    /// A click still reaching the input is cancelled and taken after the
    /// dispatch, so the new `checked` is the last write.
    pub(crate) fn input_click(&self) -> impl FnMut(Event<MouseData>) + 'static {
        let (activate, card) = (self.activate.clone(), self.card);
        move |event| {
            event.prevent_default();
            if card {
                event.stop_propagation();
            }
            let activate = activate.clone();
            spawn(async move {
                next_task().await;
                activate();
            });
        }
    }

    /// Blitz forwards a `<label>` click as `input`, never `click`. Never fires
    /// on the web.
    pub(crate) fn input_input(&self) -> impl FnMut(FormEvent) + 'static {
        let activate = self.activate.clone();
        move |_| activate()
    }

    /// Space (and Enter outside a form) on `keydown`; `true` when handled.
    pub(crate) fn keydown(&self, event: &Event<KeyboardData>) -> bool {
        if !activates(event, self.enter) {
            return false;
        }
        event.prevent_default();
        if !event.is_auto_repeating() {
            (self.activate)();
        }
        true
    }

    /// A browser that clicks on `keyup` rather than checking the cancelled
    /// `keydown` would otherwise activate a second time.
    pub(crate) fn input_keyup(&self) -> impl FnMut(Event<KeyboardData>) + 'static {
        let enter = self.enter;
        move |event| {
            if activates(&event, enter) {
                event.prevent_default();
            }
        }
    }
}

/// Where a nested link or button's click stops: the label or the card's
/// wrapper, not the control `<span>`.
pub(super) const CLICK_BOUNDARY: &str = "label, div[data-state~=\"card\"]";

fn activates(event: &KeyboardData, enter: bool) -> bool {
    match event.key() {
        Key::Character(ref c) => c == " ",
        Key::Enter => enter,
        _ => false,
    }
}
