//! `use_history` against the real non-wasm timer and a polled `VirtualDom`.

use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use dioxus::prelude::*;

use crate::hooks::{HistoryHandle, UndoHistory, use_history};

thread_local! {
    static HANDLE: RefCell<Option<HistoryHandle<String>>> = const { RefCell::new(None) };
    static RENDERS: RefCell<u32> = const { RefCell::new(0) };
}

fn pump(dom: &mut VirtualDom, span: Duration) {
    let start = Instant::now();
    while start.elapsed() < span {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        thread::sleep(Duration::from_millis(2));
    }
}

fn app() -> Element {
    let text = use_history(|| UndoHistory::new(String::new()), 80);
    let _ = text.value();
    RENDERS.with(|renders| *renders.borrow_mut() += 1);
    HANDLE.with(|handle| *handle.borrow_mut() = Some(text));
    rsx! {}
}

fn handle() -> HistoryHandle<String> {
    HANDLE.with(|handle| handle.borrow().expect("rendered"))
}

fn renders() -> u32 {
    RENDERS.with(|renders| *renders.borrow())
}

fn started() -> VirtualDom {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    pump(&mut dom, Duration::from_millis(10));
    dom
}

fn shown(dom: &VirtualDom) -> String {
    dom.in_runtime(|| handle().value().to_string())
}

fn type_into(dom: &mut VirtualDom, text: &str) {
    dom.in_runtime(|| handle().merge(text.to_string()));
    pump(dom, Duration::from_millis(10));
}

#[test]
fn typing_without_a_pause_undoes_as_one_step() {
    let mut dom = started();
    for text in ["a", "ab", "abc"] {
        type_into(&mut dom, text);
    }
    assert_eq!(shown(&dom), "abc");

    assert!(dom.in_runtime(|| handle().undo()));
    assert_eq!(shown(&dom), "");
    assert!(dom.in_runtime(|| handle().can_redo()));
}

#[test]
fn a_pause_closes_the_group() {
    let mut dom = started();
    type_into(&mut dom, "a");
    type_into(&mut dom, "ab");
    pump(&mut dom, Duration::from_millis(300));
    type_into(&mut dom, "abc");

    dom.in_runtime(|| handle().undo());
    assert_eq!(shown(&dom), "ab");
    dom.in_runtime(|| handle().undo());
    assert_eq!(shown(&dom), "");
}

#[test]
fn undo_closes_the_group_before_the_pause() {
    let mut dom = started();
    type_into(&mut dom, "a");
    dom.in_runtime(|| handle().undo());
    dom.in_runtime(|| handle().redo());
    type_into(&mut dom, "ab");

    dom.in_runtime(|| handle().undo());
    assert_eq!(shown(&dom), "a");
}

#[test]
fn a_change_renders_the_reader_and_a_pause_does_not() {
    let mut dom = started();
    let before = renders();
    type_into(&mut dom, "a");
    let after_change = renders();
    assert!(after_change > before, "the change did not render");

    pump(&mut dom, Duration::from_millis(300));
    assert_eq!(renders(), after_change, "closing the group rendered");
}

#[test]
fn reset_forgets_the_steps() {
    let dom = started();
    dom.in_runtime(|| handle().push("a".to_string()));
    dom.in_runtime(|| handle().reset("b".to_string()));

    assert_eq!(shown(&dom), "b");
    assert!(!dom.in_runtime(|| handle().can_undo()));
}
