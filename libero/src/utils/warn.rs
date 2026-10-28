use std::{cell::RefCell, collections::HashSet, rc::Rc};

/// Debug-only warning. Plain string, no `dioxus::warn!` format semantics.
#[cfg(debug_assertions)]
pub(crate) fn warn(message: &str) {
    #[cfg(test)]
    WARNINGS.with_borrow_mut(|warnings| warnings.push(message.to_string()));
    dioxus::prelude::warn!("{message}");
}

// Per thread, and every test runs on its own, so a test sees only its warnings.
#[cfg(test)]
thread_local! {
    static WARNINGS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Every `warn()` this thread made since the last call.
#[cfg(test)]
pub(crate) fn take_warnings() -> Vec<String> {
    WARNINGS.take()
}

/// What `warn()` saw while `app` built and applied the pass its effects queued,
/// so a warning from a second render is not missed.
#[cfg(test)]
pub(crate) fn warnings_of(app: fn() -> dioxus::prelude::Element) -> Vec<String> {
    take_warnings();
    let mut dom = dioxus::prelude::VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    take_warnings()
}

#[cfg(not(debug_assertions))]
pub(crate) fn warn(_message: &str) {}

/// Per-instance warning keys: each fires once, not on every render.
#[derive(Clone, Default)]
pub(crate) struct Warned(Rc<RefCell<HashSet<String>>>);

impl Warned {
    pub(crate) fn once(&self, key: &str, message: impl FnOnce() -> String) {
        if !self.0.borrow().contains(key) {
            self.0.borrow_mut().insert(key.to_owned());
            warn(&message());
        }
    }
}

/// A [`Warned`] that lives as long as the calling component.
pub(crate) fn use_warn_once() -> Warned {
    dioxus::prelude::use_hook(Warned::default)
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;

    #[test]
    fn each_key_warns_once() {
        take_warnings();
        let warned = Warned::default();
        for key in ["a", "b", "a", "b"] {
            warned.once(key, || format!("bad {key}"));
        }
        assert_eq!(take_warnings(), ["bad a", "bad b"]);
    }
}
