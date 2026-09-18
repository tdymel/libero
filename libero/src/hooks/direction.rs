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

    /// The direction chosen through [`set`](Self::set) and kept, `None` when
    /// nothing was chosen: the start, or since [`clear`](Self::clear).
    pub fn kept(&self) -> Option<Direction> {
        *self.context.kept_direction.read()
    }

    /// Drops the choice, kept one included: back to `LiberoProvider`'s
    /// `direction`, or with none, the root's `dir` removed.
    ///
    /// ```ignore
    /// // Puts back what a preview found, choice or none.
    /// let found = direction.kept();
    /// match found {
    ///     Some(found) => direction.set(found),
    ///     None => direction.clear(),
    /// }
    /// ```
    pub fn clear(&self) {
        self.context.clear_direction();
    }

    /// Turns the text the other way.
    pub fn toggle(&self) {
        // Bound first: the peek guard would outlive the write inside `set`.
        let next = self.context.direction.peek().flipped();
        self.set(next);
    }
}
