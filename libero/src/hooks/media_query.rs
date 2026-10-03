use dioxus::prelude::*;

use super::use_subscription_slot;
use crate::platform::{MediaQuerySubscription, media_query};

/// The widest viewport [`use_is_mobile`] counts as mobile, the 768px breakpoint.
const MOBILE_QUERY: &str = "(max-width: 767px)";

/// Whether a CSS media query matches, live. `false` on the first render and
/// wherever the platform has no media queries (a server render, native Blitz),
/// then the real answer once the component is mounted.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_media_query;
/// # fn app() -> Element {
/// let wide = use_media_query("(min-width: 1024px)");
///
/// rsx! {
///     if wide() {
///         p { "Wide layout" }
///     } else {
///         p { "Narrow layout" }
///     }
/// }
/// # }
/// ```
pub fn use_media_query(query: &str) -> ReadSignal<bool> {
    let mut matches = use_signal(|| false);
    let query = query.to_owned();
    let slot = use_subscription_slot::<dyn MediaQuerySubscription>();
    use_effect(use_reactive!(|query| {
        slot.clear();
        let Some(api) = media_query() else {
            matches.set(false);
            return;
        };
        let watch = api.watch(
            &query,
            Box::new(move |now| {
                let mut matches = matches;
                if *matches.peek() != now {
                    matches.set(now);
                }
            }),
        );
        slot.set(Some(watch));
    }));
    matches.into()
}

/// Whether the viewport is narrower than 768px, live. Same defaults as
/// [`use_media_query`]: `false` until mounted and where nothing can measure.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_is_mobile;
/// # fn app() -> Element {
/// let mobile = use_is_mobile();
///
/// rsx! {
///     if mobile() { "Menu button" } else { "Full navigation" }
/// }
/// # }
/// ```
pub fn use_is_mobile() -> ReadSignal<bool> {
    use_media_query(MOBILE_QUERY)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::cell::RefCell;

    use super::*;

    thread_local! {
        static ANSWERS: RefCell<Vec<(bool, bool)>> = const { RefCell::new(Vec::new()) };
    }

    fn app() -> Element {
        let wide = use_media_query("(min-width: 1024px)");
        let mobile = use_is_mobile();
        ANSWERS.with_borrow_mut(|answers| answers.push((wide(), mobile())));
        rsx! {}
    }

    #[test]
    fn without_media_queries_both_hooks_answer_false() {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let answers = ANSWERS.with_borrow(|answers| answers.clone());
        assert!(!answers.is_empty());
        assert!(answers.iter().all(|answer| *answer == (false, false)));
    }
}
