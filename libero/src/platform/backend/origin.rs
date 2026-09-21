//! The scope a callback was handed over in, to run it there again later: a read
//! from a scope above the owner is what `dioxus_signals` warns about (718).

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use dioxus::core::Runtime;
use dioxus::prelude::*;

thread_local! {
    /// One liveness token per scope, owned by it, so it reads as gone once the
    /// scope is. Keyed by runtime too: several documents share a thread.
    static TOKENS: RefCell<HashMap<(usize, ScopeId), CopyValue<()>>> =
        RefCell::new(HashMap::new());
}

/// Copy, so a callback run many times keeps it.
#[derive(Clone, Copy)]
pub(super) struct Origin(Option<CopyValue<()>>);

impl Origin {
    /// The current scope, if any.
    pub(super) fn here() -> Self {
        let Some(runtime) = Runtime::try_current() else {
            return Self(None);
        };
        let Some(scope) = runtime.try_current_scope_id() else {
            return Self(None);
        };
        let key = (Rc::as_ptr(&runtime) as usize, scope);
        Self(Some(TOKENS.with_borrow_mut(|tokens| {
            if let Some(token) = tokens.get(&key).filter(|token| alive(token)) {
                return *token;
            }
            // Once per scope: a good time to forget the scopes that are gone.
            tokens.retain(|_, token| alive(token));
            *tokens.entry(key).or_insert_with(|| CopyValue::new(()))
        })))
    }

    /// Runs `run` in the scope while it is alive, else where it is.
    pub(super) fn run<T>(&self, run: impl FnOnce() -> T) -> T {
        let live = self.0.filter(alive).zip(Runtime::try_current());
        match live {
            Some((token, runtime)) => runtime.in_scope(token.origin_scope(), run),
            None => run(),
        }
    }
}

/// Without the read `dioxus_signals` checks for scope.
fn alive(token: &CopyValue<()>) -> bool {
    token.value().try_read().is_ok()
}

#[cfg(test)]
mod tests {
    use dioxus::core::current_scope_id;

    use super::*;

    thread_local! {
        static ORIGIN: RefCell<Option<(Origin, ScopeId)>> = const { RefCell::new(None) };
    }

    #[component]
    fn Child() -> Element {
        use_hook(|| ORIGIN.set(Some((Origin::here(), current_scope_id()))));
        rsx! {}
    }

    thread_local! {
        static SHOW: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
    }

    fn mount() -> (VirtualDom, Signal<bool>) {
        let mut dom = VirtualDom::new(|| {
            let show = use_signal(|| true);
            use_hook(|| SHOW.set(Some(show)));
            rsx! {
                if show() {
                    Child {}
                }
            }
        });
        dom.rebuild_in_place();
        (dom, SHOW.take().unwrap())
    }

    #[test]
    fn a_live_origin_runs_in_its_scope() {
        let (dom, _) = mount();
        let (origin, child) = ORIGIN.take().unwrap();
        let ran_in = dom.in_scope(ScopeId::ROOT, || origin.run(current_scope_id));
        assert_eq!(ran_in, child);
    }

    #[test]
    fn a_dropped_origin_runs_where_it_is() {
        let (mut dom, mut show) = mount();
        let (origin, child) = ORIGIN.take().unwrap();
        dom.in_scope(ScopeId::ROOT, || show.set(false));
        dom.render_immediate_to_vec();
        let ran_in = dom.in_scope(ScopeId::ROOT, || origin.run(current_scope_id));
        assert_ne!(ran_in, child);
        assert_eq!(ran_in, ScopeId::ROOT);
    }

    #[test]
    fn a_scope_keeps_one_token() {
        let (dom, _) = mount();
        let (_, child) = ORIGIN.take().unwrap();
        let [first, second] = dom.in_scope(child, || {
            [Origin::here(), Origin::here()].map(|o| o.0.unwrap().id())
        });
        assert_eq!(first, second);
    }
}
