use std::sync::atomic::{AtomicU64, Ordering};

use dioxus::prelude::*;

use super::window::ZLayers;

static NEXT_MODAL_ID: AtomicU64 = AtomicU64::new(0);

/// Stacks the open modals: the last one opened is on top. Provided by
/// [`crate::LiberoProvider`].
///
/// Ids, not a counter: a counter leaked a step per modal closed below another.
/// Capped below `popover` so a dropdown inside a modal clears it; past the cap they tie.
#[derive(Clone, Copy)]
pub(crate) struct ModalHost {
    stack: Signal<Vec<u64>>,
    layers: ReadSignal<ZLayers>,
}

impl ModalHost {
    pub(crate) fn new(stack: Signal<Vec<u64>>, layers: ReadSignal<ZLayers>) -> Self {
        Self { stack, layers }
    }

    /// Puts a new modal on top and returns its id.
    pub(crate) fn open(&self) -> u64 {
        let id = NEXT_MODAL_ID.fetch_add(1, Ordering::Relaxed);
        let mut stack = self.stack;
        stack.write().push(id);
        id
    }

    pub(crate) fn close(&self, id: u64) {
        let mut stack = self.stack;
        stack.write().retain(|other| *other != id);
    }

    /// `id`'s z-index, subscribed: a modal closing below this one moves it
    /// down a step. `base` for an id that is not stacked.
    pub(crate) fn z_index(&self, id: u64) -> i32 {
        let position = self
            .stack
            .read()
            .iter()
            .position(|other| *other == id)
            .unwrap_or(0);
        self.layers.read().at(position)
    }
}

/// Lets a dialog rendered by [`crate::hooks::use_modal`] close itself.
/// Non-modal surfaces (popover, FloatingWindow) provide an empty one.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::context::ModalContext;
/// # fn app() -> Element {
/// let modal = use_context::<ModalContext>();
///
/// rsx! {
///     button { onclick: move |_| modal.close(), "Close" }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/overlay/modal>
#[derive(Clone, Copy)]
pub struct ModalContext {
    pub(crate) onclose: Option<EventHandler<()>>,
}

impl ModalContext {
    /// The boundary a non-modal surface provides.
    pub(crate) const NONE: Self = Self { onclose: None };

    /// Whether this content sits in a modal.
    pub fn is_modal(&self) -> bool {
        self.onclose.is_some()
    }

    /// Closes the surrounding modal; a no-op outside one.
    // Deferred: a synchronous close from a bubbling click re-enters the handler (`AlreadyBorrowedMut`).
    pub fn close(&self) {
        if let Some(onclose) = self.onclose {
            spawn(async move {
                onclose.call(());
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::VirtualDom;
    use dioxus::prelude::*;

    use std::cell::Cell;

    use super::ModalHost;
    use crate::context::window::ZLayers;
    use crate::theme::{Theme, ThemeSet, ZIndexDefaults};
    use crate::{LiberoContext, LiberoProvider};

    /// Runs `check` against a fresh host inside a scope, which `Signal` needs.
    fn with_host(check: fn(ModalHost)) {
        let mut dom = VirtualDom::new_with_props(
            |check: fn(ModalHost)| {
                let stack = use_signal(Vec::new);
                let layers = use_signal(|| ZLayers {
                    base: 1000,
                    step: 10,
                    ceiling: 2000,
                });
                use_hook(|| check(ModalHost::new(stack, layers.into())));
                rsx! {}
            },
            check,
        );
        dom.rebuild_in_place();
    }

    thread_local! {
        static SHOWN: Cell<i32> = const { Cell::new(0) };
    }

    #[test]
    fn the_base_follows_a_theme_switch() {
        static RAISED: Theme = Theme {
            z_index: ZIndexDefaults {
                modal: 5000,
                popover: 6000,
                ..ZIndexDefaults::DEFAULT
            },
            ..Theme::DEFAULT
        };

        #[component]
        fn Probe() -> Element {
            let host = use_context::<ModalHost>();
            let context = use_context::<LiberoContext>();
            use_effect(move || context.set_active_theme("raised"));
            SHOWN.with(|shown| shown.set(host.z_index(u64::MAX)));
            rsx! {}
        }

        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    themes: ThemeSet::new().light(&Theme::DEFAULT).named("raised", &RAISED),
                    Probe {}
                }
            }
        }

        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        for _ in 0..3 {
            dom.process_events();
            dom.render_immediate(&mut dioxus::dioxus_core::NoOpMutations);
        }
        assert_eq!(SHOWN.with(Cell::get), 5000);
    }

    #[test]
    fn a_modal_above_another_steps_once() {
        with_host(|host| {
            let a = host.open();
            let b = host.open();
            assert_eq!(host.z_index(a), 1000);
            assert_eq!(host.z_index(b), 1010);
        });
    }

    /// The leak: a counter that only gives back its top index lost a step to
    /// every modal closed below another one.
    #[test]
    fn closing_out_of_order_leaks_nothing() {
        with_host(|host| {
            for _ in 0..500 {
                let a = host.open();
                let b = host.open();
                host.close(a);
                assert_eq!(host.z_index(b), 1000, "b is alone, so it is the base");
                host.close(b);
            }
            let next = host.open();
            assert_eq!(host.z_index(next), 1000);
        });
    }

    /// Opening and closing while another modal stays up never climbs either.
    #[test]
    fn interleaved_modals_stay_a_dense_run() {
        with_host(|host| {
            let mut below = host.open();
            for _ in 0..500 {
                let above = host.open();
                assert_eq!(host.z_index(above), 1010);
                host.close(below);
                below = above;
            }
        });
    }

    #[test]
    fn the_top_is_capped_below_popover() {
        with_host(|host| {
            let ids: Vec<u64> = (0..200).map(|_| host.open()).collect();
            assert_eq!(host.z_index(ids[99]), 1990);
            assert_eq!(host.z_index(ids[199]), 1999);
        });
    }
}
