use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

/// A pure derivation of props, rebuilt only when `deps` change.
///
/// Not `use_memo`: a `Memo` re-runs when a *signal* it read changes, and props
/// are not signals, so it would keep returning the first render's value.
/// `use_memo(use_reactive!(..))` fixes that with a `use_signal` written during
/// render, which dirties the scope and costs a second render pass on every
/// change. A derivation of props needs no reactivity at all - the prop change
/// re-renders the component by itself.
///
/// `deps` are moved in and only cloned on a miss, so passing a value the
/// component does not otherwise need costs nothing per render.
pub(crate) fn use_cache<D, T>(deps: D, build: impl FnOnce(&D) -> T) -> T
where
    D: PartialEq + Clone + 'static,
    T: Clone + 'static,
{
    let cache = use_hook(|| Rc::new(RefCell::new(None::<(D, T)>)));
    let mut cache = cache.borrow_mut();

    match &*cache {
        Some((cached, value)) if *cached == deps => value.clone(),
        _ => {
            let value = build(&deps);
            *cache = Some((deps, value.clone()));
            value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    thread_local! {
        static BUILDS: Cell<usize> = const { Cell::new(0) };
        static DEP: Cell<u32> = const { Cell::new(1) };
    }

    #[component]
    fn Cached(dep: u32) -> Element {
        let value = use_cache(dep, |dep| {
            BUILDS.with(|builds| builds.set(builds.get() + 1));
            dep * 2
        });

        rsx! { span { "{value}" } }
    }

    /// Reads the dep on every render, so a test can change it between two.
    #[component]
    fn App() -> Element {
        rsx! { Cached { dep: DEP.with(Cell::get) } }
    }

    fn rerender(dom: &mut VirtualDom) {
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }

    /// Re-rendering with the same dep must not rebuild; a changed one must.
    #[test]
    fn it_rebuilds_only_when_the_deps_change() {
        let mut dom = VirtualDom::new(App);
        dom.rebuild_in_place();
        assert_eq!(BUILDS.with(Cell::get), 1);
        assert!(dioxus_ssr::render(&dom).contains("2"));

        rerender(&mut dom);
        assert_eq!(BUILDS.with(Cell::get), 1, "same dep must not rebuild");

        DEP.with(|dep| dep.set(5));
        rerender(&mut dom);
        assert_eq!(BUILDS.with(Cell::get), 2, "a changed dep must rebuild");
        assert!(dioxus_ssr::render(&dom).contains("10"));
    }
}
