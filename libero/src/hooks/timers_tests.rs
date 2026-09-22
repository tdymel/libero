//! The timer, debounce and throttle hooks against the real non-wasm timer and a
//! `VirtualDom` polled the way a renderer would.

use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use dioxus::prelude::*;

use crate::hooks::{
    IntervalHandle, TimeoutHandle, use_debounced_callback, use_debounced_value, use_interval,
    use_throttled_callback, use_throttled_value, use_timeout,
};

type Log = Signal<Vec<String>>;

thread_local! {
    static VALUE: RefCell<Option<ReadSignal<String>>> = const { RefCell::new(None) };
    static CALLBACK: RefCell<Option<Callback<String>>> = const { RefCell::new(None) };
    static TIMEOUT: RefCell<Option<TimeoutHandle>> = const { RefCell::new(None) };
    static INTERVAL: RefCell<Option<IntervalHandle>> = const { RefCell::new(None) };
}

fn pump(dom: &mut VirtualDom, span: Duration) {
    let start = Instant::now();
    while start.elapsed() < span {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        thread::sleep(Duration::from_millis(2));
    }
}

fn started(app: fn() -> Element) -> VirtualDom {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    pump(&mut dom, Duration::from_millis(10));
    dom
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
    let mut dom = started(debounced_value_app);
    assert_eq!(shown(), "first");

    for text in ["a", "ab", "abc"] {
        type_into(&dom, text);
        pump(&mut dom, Duration::from_millis(20));
    }
    assert_eq!(shown(), "first", "followed before the pause");

    pump(&mut dom, Duration::from_millis(300));
    assert_eq!(shown(), "abc");
}

#[test]
fn a_debounced_value_ignores_a_change_that_is_undone() {
    let mut dom = started(debounced_value_app);
    type_into(&dom, "other");
    pump(&mut dom, Duration::from_millis(20));
    type_into(&dom, "first");

    pump(&mut dom, Duration::from_millis(300));
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
    let mut dom = started(debounced_callback_app);
    for text in ["a", "b", "c"] {
        call(&dom, text);
        pump(&mut dom, Duration::from_millis(20));
    }
    assert!(log(&dom).is_empty(), "ran before the pause");

    pump(&mut dom, Duration::from_millis(300));
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
    let mut dom = started(throttled_callback_app);
    call(&dom, "a");
    assert_eq!(log(&dom), ["a"], "the leading call waited");

    call(&dom, "b");
    call(&dom, "c");
    pump(&mut dom, Duration::from_millis(30));
    assert_eq!(log(&dom), ["a"], "ran again inside the window");

    pump(&mut dom, Duration::from_millis(300));
    assert_eq!(log(&dom), ["a", "c"]);

    // The quiet window closed, so the next call is a leading one again.
    call(&dom, "d");
    assert_eq!(log(&dom), ["a", "c", "d"]);
}

fn throttled_value_app() -> Element {
    let input = use_context_provider(|| Signal::new("first".to_string()));
    let shown = use_throttled_value(input.into(), 100);
    VALUE.with(|value| *value.borrow_mut() = Some(shown));
    rsx! {}
}

#[test]
fn a_throttled_value_shows_the_first_change_and_the_last_of_a_burst() {
    let mut dom = started(throttled_value_app);
    type_into(&dom, "a");
    pump(&mut dom, Duration::from_millis(10));
    assert_eq!(shown(), "a", "the leading change waited");

    type_into(&dom, "b");
    pump(&mut dom, Duration::from_millis(10));
    type_into(&dom, "c");
    pump(&mut dom, Duration::from_millis(10));
    assert_eq!(shown(), "a", "changed inside the window");

    pump(&mut dom, Duration::from_millis(300));
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
    let mut dom = started(timeout_app);
    dom.in_runtime(|| timeout().start());
    assert!(dom.in_runtime(|| timeout().pending()));

    pump(&mut dom, Duration::from_millis(300));
    assert_eq!(log(&dom), ["fired"]);
    assert!(!dom.in_runtime(|| timeout().pending()));
}

#[test]
fn a_stopped_timeout_never_fires() {
    let mut dom = started(timeout_app);
    dom.in_runtime(|| timeout().start());
    dom.in_runtime(|| timeout().stop());

    pump(&mut dom, Duration::from_millis(200));
    assert!(log(&dom).is_empty());
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
    let mut dom = started(interval_app);
    pump(&mut dom, Duration::from_millis(100));
    assert!(log(&dom).is_empty(), "ticked before start");

    dom.in_runtime(|| interval().toggle());
    assert!(dom.in_runtime(|| interval().active()));
    pump(&mut dom, Duration::from_millis(400));
    let ticks = log(&dom).len();
    assert!(ticks >= 3, "only {ticks} ticks");

    dom.in_runtime(|| interval().toggle());
    assert!(!dom.in_runtime(|| interval().active()));
    pump(&mut dom, Duration::from_millis(200));
    assert!(log(&dom).len() <= ticks + 1, "kept ticking after stop");
}
