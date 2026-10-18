use dioxus::prelude::*;

use crate::{context::LiberoContext, tokens::Direction};

/// The app's text direction, set on the document root's `dir` so every
/// component and the overlays' portal follow.
///
/// [`DirectionToggle`](crate::components::DirectionToggle) is the ready-made switch built on it.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::Switch;
/// # use libero::hooks::use_direction;
/// # fn app() -> Element {
/// let direction = use_direction();
/// let rtl = direction.is_rtl();
///
/// rsx! {
///     Switch {
///         label: "Right to left",
///         checked: rtl,
///         onchange: move |_| direction.toggle(),
///     }
/// }
/// # }
/// ```
///
/// Reactive. It knows only what `LiberoProvider` started in and what was set
/// through it, not a `dir` written on the root by other means. Panics outside a
/// `LiberoProvider`.
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
    /// The current direction. Reactive.
    pub fn get(&self) -> Direction {
        *self.context.direction.read()
    }

    /// Whether the text runs right to left. Reactive.
    pub fn is_rtl(&self) -> bool {
        self.get() == Direction::Rtl
    }

    /// Turns the app's text. Kept where the platform can (the web's
    /// `localStorage`), so a reload keeps it.
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
    /// ```rust
    /// # use libero::hooks::DirectionHandle;
    /// // Puts back what a preview found, choice or none.
    /// fn restore(direction: &DirectionHandle, found: Option<libero::theme::Direction>) {
    ///     match found {
    ///         Some(found) => direction.set(found),
    ///         None => direction.clear(),
    ///     }
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::LiberoProvider;

    thread_local! {
        static HANDLE: RefCell<Option<DirectionHandle>> = const { RefCell::new(None) };
    }

    #[component]
    fn Probe() -> Element {
        let direction = use_direction();
        HANDLE.with(|handle| *handle.borrow_mut() = Some(direction.clone()));
        rsx! { span { "{direction.get().as_str()}" } }
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Probe {} } }
    }

    #[test]
    fn toggle_turns_the_text_each_time() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        let handle = HANDLE
            .with(|handle| handle.borrow().clone())
            .expect("the probe rendered");

        for expected in ["rtl", "ltr", "rtl"] {
            dom.in_scope(ScopeId::APP, || handle.toggle());
            dom.render_immediate(&mut dioxus::core::NoOpMutations);
            let html = dioxus_ssr::render(&dom);
            assert!(html.contains(&format!(">{expected}<")), "{html}");
        }
    }
}
