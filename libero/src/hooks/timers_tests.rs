//! The timer, debounce and throttle hooks on the manual test clock, with a
//! `VirtualDom` rendered after each firing the way a renderer would.

use std::cell::RefCell;

use dioxus::prelude::*;

use super::polling_tests::{elapse, flush, started};
use crate::hooks::{
    IntervalHandle, TimeoutHandle, use_debounced_callback, use_debounced_value, use_interval,
    use_throttled_callback, use_throttled_value, use_timeout,
};
use crate::platform::manual_timer;

type Log = Signal<Vec<String>>;

thread_local! {
    static VALUE: RefCell<Option<ReadSignal<String>>> = const { RefCell::new(None) };
    static CALLBACK: RefCell<Option<Callback<String>>> = const { RefCell::new(None) };
    static TIMEOUT: RefCell<Option<TimeoutHandle>> = const { RefCell::new(None) };
    static INTERVAL: RefCell<Option<IntervalHandle>> = const { RefCell::new(None) };
}

fn log(dom: &VirtualDom) -> Vec<String> {
    let log = dom.in_scope(ScopeId::APP, consume_context::<Log>);
    dom.in_runtime(|| log.peek().clone())
}

fn shown() -> String {
    VALUE.with(|value| value.borrow().expect("rendered").peek().clone())
}

fn call(dom: &VirtualDom, text: &str) {
    let callback = CALLBACK.with(|callback| callback.borrow().expect("rendered"));
    dom.in_runtime(|| callback.call(text.to_string()));
}

fn type_into(dom: &VirtualDom, text: &str) {
    let mut input = dom.in_scope(ScopeId::APP, consume_context::<Signal<String>>);
    dom.in_runtime(|| input.set(text.to_string()));
}

fn push(mut log: Log, entry: impl Into<String>) {
    log.write().push(entry.into());
}

fn debounced_value_app() -> Element {
    let input = use_context_provider(|| Signal::new("first".to_string()));
    let settled = use_debounced_value(input.into(), 80);
    VALUE.with(|value| *value.borrow_mut() = Some(settled));
    rsx! {}
}

#[test]
fn a_debounced_value_follows_after_the_typing_stops() {
    let _clock = manual_timer::install();
    let mut dom = started(debounced_value_app);
    assert_eq!(shown(), "first");

    for text in ["a", "ab", "abc"] {
        type_into(&dom, text);
        elapse(&mut dom, 20);
    }
    assert_eq!(shown(), "first", "followed before the pause");

    elapse(&mut dom, 60);
    assert_eq!(shown(), "abc");
}

#[test]
fn a_debounced_value_ignores_a_change_that_is_undone() {
    let _clock = manual_timer::install();
    let mut dom = started(debounced_value_app);
    type_into(&dom, "other");
    elapse(&mut dom, 20);
    type_into(&dom, "first");

    elapse(&mut dom, 80);
    assert_eq!(shown(), "first");
}

fn debounced_callback_app() -> Element {
    let log = use_context_provider(|| Signal::new(Vec::<String>::new()));
    let callback = use_debounced_callback(move |text: String| push(log, text), 80);
    CALLBACK.with(|slot| *slot.borrow_mut() = Some(callback));
    rsx! {}
}

#[test]
fn a_debounced_callback_runs_once_with_the_last_argument() {
    let _clock = manual_timer::install();
    let mut dom = started(debounced_callback_app);
    for text in ["a", "b", "c"] {
        call(&dom, text);
        elapse(&mut dom, 20);
    }
    assert!(log(&dom).is_empty(), "ran before the pause");

    elapse(&mut dom, 60);
    assert_eq!(log(&dom), ["c"]);
    // Once: no second call follows the first.
    elapse(&mut dom, 80);
    assert_eq!(log(&dom), ["c"]);
}

fn throttled_callback_app() -> Element {
    let log = use_context_provider(|| Signal::new(Vec::<String>::new()));
    let callback = use_throttled_callback(move |text: String| push(log, text), 100);
    CALLBACK.with(|slot| *slot.borrow_mut() = Some(callback));
    rsx! {}
}

#[test]
fn a_throttled_callback_runs_at_once_then_once_per_window() {
    let _clock = manual_timer::install();
    let mut dom = started(throttled_callback_app);
    call(&dom, "a");
    assert_eq!(log(&dom), ["a"], "the leading call waited");

    call(&dom, "b");
    call(&dom, "c");
    elapse(&mut dom, 30);
    assert_eq!(log(&dom), ["a"], "ran again inside the window");

    elapse(&mut dom, 70);
    assert_eq!(log(&dom), ["a", "c"]);

    // The quiet window closed, so the next call is a leading one again.
    elapse(&mut dom, 100);
    call(&dom, "d");
    assert_eq!(log(&dom), ["a", "c", "d"]);
}

fn reading_throttled_callback_app() -> Element {
    let log = use_context_provider(|| Signal::new(Vec::<String>::new()));
    let prefix = use_context_provider(|| Signal::new("x".to_string()));
    let callback = use_throttled_callback(
        move |text: String| push(log, format!("{prefix}{text}")),
        100,
    );
    CALLBACK.with(|slot| *slot.borrow_mut() = Some(callback));
    rsx! {}
}

#[test]
fn a_throttled_callback_that_reads_a_signal_keeps_its_window() {
    let _clock = manual_timer::install();
    let mut dom = started(reading_throttled_callback_app);
    call(&dom, "a");
    call(&dom, "b");
    elapse(&mut dom, 100);
    assert_eq!(log(&dom).len(), 2, "the trailing call");

    // The trailing call read `prefix`; changing it is no firing and keeps the window open.
    let mut prefix = dom.in_scope(ScopeId::APP, consume_context::<Signal<String>>);
    dom.in_runtime(|| prefix.set("y".to_string()));
    flush(&mut dom);
    call(&dom, "c");
    assert_eq!(log(&dom), ["xa", "xb"], "ran inside the window");
}

fn throttled_value_app() -> Element {
    let input = use_context_provider(|| Signal::new("first".to_string()));
    let shown = use_throttled_value(input.into(), 100);
    VALUE.with(|value| *value.borrow_mut() = Some(shown));
    rsx! {}
}

#[test]
fn a_throttled_value_shows_the_first_change_and_the_last_of_a_burst() {
    let _clock = manual_timer::install();
    let mut dom = started(throttled_value_app);
    type_into(&dom, "a");
    flush(&mut dom);
    assert_eq!(shown(), "a", "the leading change waited");

    type_into(&dom, "b");
    elapse(&mut dom, 10);
    type_into(&dom, "c");
    elapse(&mut dom, 10);
    assert_eq!(shown(), "a", "changed inside the window");

    elapse(&mut dom, 80);
    assert_eq!(shown(), "c");
}

fn timeout_app() -> Element {
    let log = use_context_provider(|| Signal::new(Vec::<String>::new()));
    let timeout = use_timeout(move || push(log, "fired"), 60);
    TIMEOUT.with(|slot| *slot.borrow_mut() = Some(timeout));
    rsx! {}
}

fn timeout() -> TimeoutHandle {
    TIMEOUT.with(|slot| slot.borrow().expect("rendered"))
}

#[test]
fn a_timeout_fires_once_after_start() {
    let _clock = manual_timer::install();
    let mut dom = started(timeout_app);
    dom.in_runtime(|| timeout().start());
    assert!(dom.in_runtime(|| timeout().pending()));

    elapse(&mut dom, 59);
    assert!(log(&dom).is_empty(), "fired early");
    elapse(&mut dom, 1);
    assert_eq!(log(&dom), ["fired"]);
    elapse(&mut dom, 60);
    assert_eq!(log(&dom), ["fired"]);
    assert!(!dom.in_runtime(|| timeout().pending()));
}

#[test]
fn a_stopped_timeout_never_fires() {
    let _clock = manual_timer::install();
    let mut dom = started(timeout_app);
    dom.in_runtime(|| timeout().start());
    dom.in_runtime(|| timeout().stop());

    elapse(&mut dom, 60);
    assert!(log(&dom).is_empty());
}

fn reading_timeout_app() -> Element {
    let log = use_context_provider(|| Signal::new(Vec::<String>::new()));
    let label = use_context_provider(|| Signal::new("fired".to_string()));
    let timeout = use_timeout(move || push(log, label()), 10);
    TIMEOUT.with(|slot| *slot.borrow_mut() = Some(timeout));
    rsx! {}
}

/// `count.set(count() + 1)` is the same trap, looping with no timer at all.
#[test]
fn a_timeout_that_reads_a_signal_fires_once() {
    let _clock = manual_timer::install();
    let mut dom = started(reading_timeout_app);
    dom.in_runtime(|| timeout().start());
    elapse(&mut dom, 10);
    assert_eq!(log(&dom), ["fired"]);

    // A change of what the callback read is no firing.
    let mut label = dom.in_scope(ScopeId::APP, consume_context::<Signal<String>>);
    dom.in_runtime(|| label.set("again".to_string()));
    flush(&mut dom);
    assert_eq!(log(&dom), ["fired"]);
}

fn interval_app() -> Element {
    let log = use_context_provider(|| Signal::new(Vec::<String>::new()));
    let interval = use_interval(move || push(log, "tick"), 30);
    INTERVAL.with(|slot| *slot.borrow_mut() = Some(interval));
    rsx! {}
}

fn interval() -> IntervalHandle {
    INTERVAL.with(|slot| slot.borrow().expect("rendered"))
}

#[test]
fn an_interval_ticks_between_start_and_stop() {
    let _clock = manual_timer::install();
    let mut dom = started(interval_app);
    elapse(&mut dom, 60);
    assert!(log(&dom).is_empty(), "ticked before start");

    dom.in_runtime(|| interval().toggle());
    assert!(dom.in_runtime(|| interval().active()));
    elapse(&mut dom, 90);
    assert_eq!(log(&dom).len(), 3);

    dom.in_runtime(|| interval().toggle());
    assert!(!dom.in_runtime(|| interval().active()));
    elapse(&mut dom, 60);
    assert_eq!(log(&dom).len(), 3, "kept ticking after stop");
}
