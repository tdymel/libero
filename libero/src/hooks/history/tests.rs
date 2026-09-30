//! `use_history` against the real non-wasm timer and a polled `VirtualDom`.

use std::cell::RefCell;

use dioxus::prelude::*;

use crate::hooks::polling_tests::{flush, settle};
use crate::hooks::{HistoryHandle, UndoHistory, use_history};

thread_local! {
    static HANDLE: RefCell<Option<HistoryHandle<String>>> = const { RefCell::new(None) };
    static RENDERS: RefCell<u32> = const { RefCell::new(0) };
}

/// The pause that closes a group.
const PAUSE_MS: u64 = 80;

fn app() -> Element {
    let text = use_history(|| UndoHistory::new(String::new()), PAUSE_MS);
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
    crate::hooks::polling_tests::started(app)
}

fn shown(dom: &VirtualDom) -> String {
    dom.in_runtime(|| handle().value().to_string())
}

fn type_into(dom: &mut VirtualDom, text: &str) {
    dom.in_runtime(|| handle().merge(text.to_string()));
    flush(dom);
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
    settle(&mut dom, PAUSE_MS);
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

    settle(&mut dom, PAUSE_MS);
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
