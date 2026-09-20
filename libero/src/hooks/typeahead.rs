//! Typeahead for a list that owns its focus: typing "sav" moves to "Save". The
//! pure [`typeahead_match`], and [`use_typeahead`], a buffer a pause forgets.

use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;

use crate::platform::{TimerSubscription, timer};

/// How long a pause forgets the typed text. What desktop menus and APG's
/// listbox example use.
pub(crate) const TYPEAHEAD_RESET: Duration = Duration::from_millis(500);

/// The typed text, and the timer that forgets it. A `RefCell`, not a signal:
/// nothing renders it, and the web timer's callback runs without a runtime.
#[derive(Clone)]
pub(crate) struct Typeahead {
    query: Rc<RefCell<String>>,
    reset: Rc<RefCell<Option<Box<dyn TimerSubscription>>>>,
    delay: Duration,
}

impl Typeahead {
    /// Adds `ch` to the typed text and returns the whole of it, restarting the
    /// pause that forgets it.
    pub(crate) fn push(&self, ch: char) -> String {
        let mut query = self.query.borrow_mut();
        query.push(ch);

        let buffer = self.query.clone();
        // Replacing the subscription drops the old one, which cancels it.
        *self.reset.borrow_mut() = timer()
            .map(|timer| timer.after(self.delay, Box::new(move || buffer.borrow_mut().clear())));
        query.clone()
    }

    /// Whether a query is still being typed - which is what lets a space be
    /// part of "save as" rather than an activation.
    pub(crate) fn is_typing(&self) -> bool {
        !self.query.borrow().is_empty()
    }
}

/// A typeahead buffer that forgets what was typed after `delay` without a
/// keystroke: [`TYPEAHEAD_RESET`] unless a test needs it shorter.
pub(crate) fn use_typeahead(delay: Duration) -> Typeahead {
    use_hook(|| Typeahead {
        query: Rc::new(RefCell::new(String::new())),
        reset: Rc::new(RefCell::new(None)),
        delay,
    })
}

/// The row a typed `query` lands on, from `current`, wrapping; `label` is `None`
/// for a skipped row. One repeated character ("ss") cycles; longer narrows.
pub(crate) fn typeahead_match<'a>(
    len: usize,
    current: Option<usize>,
    query: &str,
    label: impl Fn(usize) -> Option<&'a str>,
) -> Option<usize> {
    let mut chars = query.chars();
    let first = chars.next()?;
    let repeated = chars.all(|ch| ch == first);
    let needle = match repeated {
        true => &query[..first.len_utf8()],
        false => query,
    };
    // With nothing focused yet there is nothing to move on from, so the top
    // row is a candidate either way.
    let (start, skip) = match current {
        Some(current) => (current, usize::from(repeated)),
        None => (0, 0),
    };

    (skip..len + skip).find_map(|offset| {
        let index = (start + offset) % len;
        label(index)
            .filter(|label| starts_with_ignoring_case(label, needle))
            .map(|_| index)
    })
}

fn starts_with_ignoring_case(label: &str, needle: &str) -> bool {
    let mut label = label.chars().flat_map(char::to_lowercase);
    needle
        .chars()
        .flat_map(char::to_lowercase)
        .all(|ch| label.next() == Some(ch))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LABELS: [&str; 5] = ["Save", "Save as", "Share", "Delete", "Settings"];

    fn find(current: Option<usize>, query: &str, disabled: &[usize]) -> Option<usize> {
        typeahead_match(LABELS.len(), current, query, |index| {
            (!disabled.contains(&index)).then_some(LABELS[index])
        })
    }

    #[test]
    fn one_character_cycles_through_the_rows_it_starts() {
        assert_eq!(find(Some(0), "s", &[]), Some(1));
        assert_eq!(find(Some(1), "s", &[]), Some(2));
        assert_eq!(find(Some(2), "s", &[]), Some(4));
        // Wraps back past the end.
        assert_eq!(find(Some(4), "s", &[]), Some(0));
    }

    #[test]
    fn a_repeated_character_still_cycles() {
        assert_eq!(find(Some(1), "ss", &[]), Some(2));
    }

    #[test]
    fn a_longer_query_stays_on_a_row_that_still_matches() {
        assert_eq!(find(Some(0), "sa", &[]), Some(0));
        assert_eq!(find(Some(0), "save ", &[]), Some(1));
        assert_eq!(find(Some(0), "sh", &[]), Some(2));
    }

    #[test]
    fn it_ignores_case() {
        assert_eq!(find(Some(0), "DEL", &[]), Some(3));
    }

    #[test]
    fn a_skipped_row_is_never_landed_on() {
        assert_eq!(find(Some(0), "sh", &[2]), None);
        assert_eq!(find(Some(0), "s", &[1, 2]), Some(4));
    }

    #[test]
    fn nothing_focused_searches_from_the_top() {
        assert_eq!(find(None, "s", &[]), Some(0));
        assert_eq!(find(None, "d", &[]), Some(3));
        assert_eq!(find(None, "sa", &[]), Some(0));
    }

    #[test]
    fn no_match_and_no_query_are_none() {
        assert_eq!(find(Some(0), "x", &[]), None);
        assert_eq!(find(Some(0), "", &[]), None);
    }

    /// The reset is the part that needs a timer, so it runs against the real
    /// non-wasm arm and a real `VirtualDom` polling the task the callback is
    /// delivered on - `platform/timer.rs`'s own harness.
    #[cfg(not(target_arch = "wasm32"))]
    mod reset {
        use std::cell::RefCell;
        use std::thread;
        use std::time::{Duration, Instant};

        use super::super::*;

        thread_local! {
            static HANDLE: RefCell<Option<Typeahead>> = const { RefCell::new(None) };
        }

        fn app() -> Element {
            let typeahead = use_typeahead(Duration::from_millis(100));
            HANDLE.with(|handle| *handle.borrow_mut() = Some(typeahead));
            rsx! {}
        }

        fn handle() -> Typeahead {
            HANDLE
                .with(|handle| handle.borrow().clone())
                .expect("the app rendered")
        }

        fn drive(dom: &mut VirtualDom, limit: Duration) {
            let start = Instant::now();
            while start.elapsed() < limit {
                dom.process_events();
                thread::sleep(Duration::from_millis(1));
            }
        }

        #[test]
        fn the_query_builds_up_and_is_forgotten_after_a_pause() {
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            let typeahead = handle();

            dom.in_runtime(|| {
                assert_eq!(typeahead.push('s'), "s");
                assert_eq!(typeahead.push('a'), "sa");
            });
            assert!(typeahead.is_typing());

            drive(&mut dom, Duration::from_millis(500));
            assert!(!typeahead.is_typing(), "the pause did not forget the query");
            dom.in_runtime(|| assert_eq!(typeahead.push('d'), "d"));
        }

        #[test]
        fn each_keystroke_restarts_the_pause() {
            let mut dom = VirtualDom::new(app);
            dom.rebuild_in_place();
            let typeahead = handle();

            // Five keystrokes 20ms apart span 100ms, the whole pause, but no
            // single gap comes near it.
            for ch in "hello".chars() {
                dom.in_runtime(|| {
                    typeahead.push(ch);
                });
                drive(&mut dom, Duration::from_millis(20));
            }
            dom.in_runtime(|| assert_eq!(typeahead.push('!'), "hello!"));
        }
    }
}
