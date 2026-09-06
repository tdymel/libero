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

#[cfg(not(debug_assertions))]
pub(crate) fn warn(_message: &str) {}
