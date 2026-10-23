use dioxus::prelude::*;

use crate::{
    hooks::{ElementHandle, use_silent_focus_in, use_silent_focus_out_of},
    platform::ElementApi,
};

/// Closing a searchable list clears its query and refocuses the trigger, unless the
/// search box's blur closed it: focus went elsewhere then (todo 1497).
/// Opening is `ComboboxCore`'s: the list is hidden until measured.
/// Returns the blur mark, which the search box sets before it closes.
pub(crate) fn use_refocus_on_close(
    opened: bool,
    searchable: bool,
    trigger: ElementHandle,
    mut query: Signal<String>,
) -> Signal<bool> {
    let mut was_open = use_signal(|| false);
    let mut blurred = use_signal(|| false);
    use_effect(use_reactive!(|(opened, searchable)| {
        if !searchable {
            return;
        }
        // `peek`, so writing it below cannot re-trigger this effect forever.
        let previously = *was_open.peek();
        if previously && !opened {
            query.set(String::new());
            if !*blurred.peek() {
                let _ = trigger.focus();
            }
        }
        if previously != opened {
            was_open.set(opened);
            blurred.set(false);
        }
    }));
    blurred
}

/// A list opened by a touch keeps focus on its trigger, so no soft keyboard covers the rows;
/// the search box focuses natively on its own tap, inside the gesture (todo 3899).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct TouchHold {
    held: Signal<bool>,
    /// The press went into the search box, so the trigger's blur keeps the list open.
    into_search: CopyValue<bool>,
}

pub(crate) fn use_touch_hold() -> TouchHold {
    TouchHold {
        held: use_signal(|| false),
        into_search: use_hook(|| CopyValue::new(false)),
    }
}

impl TouchHold {
    /// Whether the trigger keeps focus and the combobox role. Read, so the role moves with focus.
    pub(crate) fn held(self) -> bool {
        (self.held)()
    }

    /// [`held`](Self::held) at a press, not a render.
    pub(crate) fn holding(self) -> bool {
        *self.held.peek()
    }

    /// The trigger's press while closed: a touch or pen holds, a mouse does not.
    pub(crate) fn press(self, event: &PointerEvent, open: bool) {
        if !open {
            self.set(event.pointer_type() != "mouse");
        }
    }

    /// A key on the closed trigger opens into the search box, as before.
    pub(crate) fn key(self, open: bool) {
        if !open {
            self.set(false);
        }
    }

    /// The search box's `pointerdown`.
    pub(crate) fn search_press(self) {
        let mut into_search = self.into_search;
        into_search.set(true);
    }

    /// The search box's `focus`: it is the combobox from here.
    pub(crate) fn search_focus(self) {
        let mut into_search = self.into_search;
        into_search.set(false);
        self.set(false);
    }

    /// Whether the held trigger's blur closes the list: not when the press went into the search box.
    pub(crate) fn closes_on_blur(self) -> bool {
        let mut into_search = self.into_search;
        let into = std::mem::take(&mut *into_search.write());
        self.holding() && !into
    }

    fn set(self, next: bool) {
        let mut held = self.held;
        if *held.peek() != next {
            held.set(next);
        }
    }
}

/// `open_on_focus`: a focus opens the list, but not one a press brings, whose click
/// toggles, nor the refocus after a close.
#[derive(Clone, Copy)]
pub(crate) struct FocusOpen {
    enabled: bool,
    open: Callback<()>,
    /// Deduplicates a platform that fires both `focus` and a silent move.
    focused: CopyValue<bool>,
    pressed: CopyValue<bool>,
    returning: CopyValue<bool>,
    was_open: CopyValue<bool>,
}

/// Blitz's Tab fires no `focus`/`blur`, so the silent moves on `element` count too.
pub(crate) fn use_focus_open(
    enabled: bool,
    element: ElementHandle,
    open: Callback<()>,
) -> FocusOpen {
    let focus_open = FocusOpen {
        enabled,
        open,
        focused: use_hook(|| CopyValue::new(false)),
        pressed: use_hook(|| CopyValue::new(false)),
        returning: use_hook(|| CopyValue::new(false)),
        was_open: use_hook(|| CopyValue::new(false)),
    };
    use_silent_focus_in(element, move || focus_open.focus());
    use_silent_focus_out_of(element, move || focus_open.blur());
    focus_open
}

impl FocusOpen {
    /// Called during render, before the effect that refocuses the trigger: `refocuses`
    /// says the close hands focus back to it.
    pub(crate) fn rendered(self, opened: bool, refocuses: bool) {
        let (mut was_open, mut returning) = (self.was_open, self.returning);
        if std::mem::replace(&mut *was_open.write(), opened) && !opened && refocuses {
            returning.set(true);
        }
    }

    /// A pointer press on the control: its click opens or closes.
    pub(crate) fn press(self) {
        let mut pressed = self.pressed;
        pressed.set(true);
    }

    pub(crate) fn click(self) {
        let mut pressed = self.pressed;
        pressed.set(false);
    }

    pub(crate) fn focus(self) {
        let (mut focused, mut returning) = (self.focused, self.returning);
        if std::mem::replace(&mut *focused.write(), true) {
            return;
        }
        let returned = std::mem::take(&mut *returning.write());
        if self.enabled && !returned && !*self.pressed.peek() {
            self.open.call(());
        }
    }

    pub(crate) fn blur(self) {
        let (mut focused, mut pressed) = (self.focused, self.pressed);
        focused.set(false);
        pressed.set(false);
    }
}
