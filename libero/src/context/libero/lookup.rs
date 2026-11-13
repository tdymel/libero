//! `LiberoContext` without the walk: `consume_context` climbs every ancestor scope, and a
//! styled component asks several times per mount (`use_theme`, `use_box`, ..).

use std::{cell::RefCell, rc::Rc};

use dioxus::{core::Runtime, prelude::*};

use super::LiberoContext;

thread_local! {
    /// Every mounted provider's context, by its app's runtime.
    static PROVIDED: RefCell<Vec<(usize, Rc<()>, LiberoContext)>> = const { RefCell::new(Vec::new()) };
}

fn runtime_key() -> usize {
    Rc::as_ptr(&Runtime::current()) as usize
}

/// Takes its provider out of [`PROVIDED`] when the provider's scope goes.
struct Provided(Rc<()>);

impl Drop for Provided {
    fn drop(&mut self) {
        // `try_with`: a thread's teardown may drop an app after its thread locals.
        let _ = PROVIDED.try_with(|provided| {
            provided
                .borrow_mut()
                .retain(|(_, id, _)| !Rc::ptr_eq(id, &self.0));
        });
    }
}

/// Makes `context` what [`libero_context`] answers while this provider is its app's only one.
pub(super) fn use_provided(context: &LiberoContext) {
    use_hook(|| {
        let id = Rc::new(());
        PROVIDED.with_borrow_mut(|provided| {
            provided.push((runtime_key(), id.clone(), context.clone()));
        });
        Rc::new(Provided(id))
    });
}

/// `consume_context::<LiberoContext>()`. An app with one provider, the usual one, answers
/// from the thread local; nested providers walk, as the nearest one counts.
pub(crate) fn libero_context() -> LiberoContext {
    let runtime = runtime_key();
    let sole = PROVIDED.with_borrow(|provided| {
        let mut own = provided.iter().filter(|(key, ..)| *key == runtime);
        match (own.next(), own.next()) {
            (Some((.., context)), None) => Some(context.clone()),
            _ => None,
        }
    });
    sole.unwrap_or_else(consume_context::<LiberoContext>)
}

/// `use_context::<LiberoContext>()` through [`libero_context`].
pub(crate) fn use_libero_context() -> LiberoContext {
    use_hook(libero_context)
}
