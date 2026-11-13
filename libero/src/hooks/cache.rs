use std::cell::RefCell;
use std::rc::Rc;

use dioxus::prelude::*;

/// A pure derivation of props, rebuilt only when `deps` change. Not `use_memo`,
/// which ignores props, nor `use_reactive!`, which costs a second render pass.
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

/// How many results [`use_shared_cache`] keeps per call site before it starts over.
#[cfg(any(
    feature = "button",
    feature = "icon",
    feature = "action-icon",
    feature = "avatar"
))]
const SHARED_KEPT: usize = 64;

/// [`use_cache`] whose build is shared by every instance: the rows of a list each built the
/// same colour variables on mount. `build` is a plain `fn`, so its result hangs on `deps` alone.
#[cfg(any(
    feature = "button",
    feature = "icon",
    feature = "action-icon",
    feature = "avatar"
))]
pub(crate) fn use_shared_cache<D, T>(deps: D, build: fn(&D) -> T) -> T
where
    D: PartialEq + Clone + 'static,
    T: Clone + 'static,
{
    use std::{any::Any, collections::HashMap};
    thread_local! {
        /// Per `build`, the deps it has seen with their results.
        static SHARED: RefCell<HashMap<usize, Box<dyn Any>>> = RefCell::new(HashMap::new());
    }
    use_cache(deps, |deps| {
        SHARED.with_borrow_mut(|shared| {
            let seen = shared
                .entry(build as usize)
                .or_insert_with(|| Box::new(Vec::<(D, T)>::new()))
                .downcast_mut::<Vec<(D, T)>>();
            // Two `fn`s the linker folded into one address, of other types: build unshared.
            let Some(seen) = seen else {
                return build(deps);
            };
            if let Some((_, value)) = seen.iter().find(|(known, _)| known == deps) {
                return value.clone();
            }
            if seen.len() >= SHARED_KEPT {
                seen.clear();
            }
            let value = build(deps);
            seen.push((deps.clone(), value.clone()));
            value
        })
    })
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
