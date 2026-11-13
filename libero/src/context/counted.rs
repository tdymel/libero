//! Contexts nearly every component asks for and few pages provide (a theme scope, a button
//! group, a toolbar, a disabled fieldset): unprovided, `try_consume_context` climbs to the root.

use std::{any::TypeId, cell::RefCell, rc::Rc};

use dioxus::prelude::*;

thread_local! {
    /// How many providers of each counted type are mounted, in any app on this thread.
    static LIVE: RefCell<Vec<(TypeId, usize)>> = const { RefCell::new(Vec::new()) };
}

/// The provided type: private, so a counted context is reachable through this module only.
#[derive(Clone)]
struct Counted<T>(T);

/// One mounted provider; uncounted when its scope goes.
struct Live(TypeId);

impl Drop for Live {
    fn drop(&mut self) {
        // `try_with`: a thread's teardown may drop an app after its thread locals.
        let _ = LIVE.try_with(|live| {
            if let Some((_, count)) = live.borrow_mut().iter_mut().find(|(id, _)| *id == self.0) {
                *count = count.saturating_sub(1);
            }
        });
    }
}

fn count<T: 'static>() -> Rc<Live> {
    let id = TypeId::of::<T>();
    LIVE.with_borrow_mut(
        |live| match live.iter_mut().find(|(known, _)| *known == id) {
            Some((_, count)) => *count += 1,
            None => live.push((id, 1)),
        },
    );
    Rc::new(Live(id))
}

/// `use_context_provider` for a context read through [`try_counted_context`].
pub(crate) fn use_counted_context_provider<T: Clone + 'static>(value: impl FnOnce() -> T) -> T {
    let (_, value) = use_hook(|| (count::<T>(), provide_context(Counted(value())).0));
    value
}

/// `try_consume_context`, answering `None` at once while no provider is mounted anywhere.
pub(crate) fn try_counted_context<T: Clone + 'static>() -> Option<T> {
    let id = TypeId::of::<T>();
    let provided =
        LIVE.with_borrow(|live| live.iter().any(|(known, count)| *known == id && *count > 0));
    provided
        .then(try_consume_context::<Counted<T>>)
        .flatten()
        .map(|Counted(value)| value)
}
