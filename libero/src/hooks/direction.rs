use dioxus::prelude::*;

use crate::{context::LiberoContext, tokens::Direction};

/// The app's text direction, and how to turn it. It sets the document root's
/// `dir`, so every component and the overlays' portal follow.
///
/// [`DirectionToggle`](crate::components::DirectionToggle) is the ready-made
/// switch built on it.
///
/// ```ignore
/// let direction = use_direction();
///
/// rsx! {
///     Switch {
///         checked: direction.is_rtl(),
///         onchange: move |rtl: bool| direction.set(if rtl { Direction::Rtl } else { Direction::Ltr }),
///         "Right to left"
///     }
/// }
/// ```
///
/// Reactive: a component that reads it re-renders when it changes. It knows
/// only what `LiberoProvider` started in and what was set through it, not a
/// `dir` written on the root by other means.
pub fn use_direction() -> DirectionHandle {
    DirectionHandle {
        context: use_context::<LiberoContext>(),
    }
}

/// What [`use_direction`] hands back.
#[derive(Clone)]
pub struct DirectionHandle {
    context: LiberoContext,
}

impl DirectionHandle {
    pub fn get(&self) -> Direction {
        *self.context.direction.read()
    }

    pub fn is_rtl(&self) -> bool {
        self.get() == Direction::Rtl
    }

    /// Turns the app's text. Kept where the platform has somewhere to keep it
    /// (the web's `localStorage`), so a reload comes back the same way.
    pub fn set(&self, direction: Direction) {
        self.context.set_direction(direction);
    }

    /// Turns the text the other way.
    pub fn toggle(&self) {
        // Bound first: the peek guard would outlive the write inside `set`.
        let next = self.context.direction.peek().flipped();
        self.set(next);
    }
}
