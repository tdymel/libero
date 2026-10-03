//! `use_clipboard` and the copy button on a fake clipboard, so a test build
//! sees both the accepted and the denied write.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::html::PlatformEventData;
use dioxus::prelude::*;

use super::{Clipboard, use_clipboard};
use crate::LiberoProvider;
use crate::components::CodeBlock;
use crate::hooks::polling_tests::{pump_until, started};
use crate::platform::{ClipboardApi, PlatformError, Write, fake_clipboard};
use crate::test_converter::{FakeMouse, TestConverter};

/// Accepts or denies every write, and keeps what it was given.
struct FakeClipboard {
    accept: bool,
    written: RefCell<Vec<String>>,
}

impl ClipboardApi for FakeClipboard {
    fn write_text(&self, text: String) -> Write {
        self.written.borrow_mut().push(text);
        let answer = if self.accept {
            Ok(())
        } else {
            Err(PlatformError::Denied)
        };
        Box::pin(std::future::ready(answer))
    }
}

fn leaked(accept: bool) -> &'static FakeClipboard {
    Box::leak(Box::new(FakeClipboard {
        accept,
        written: RefCell::default(),
    }))
}

thread_local! {
    static CLIPBOARD: Cell<Option<Clipboard>> = const { Cell::new(None) };
}

fn hook_app() -> Element {
    let clipboard = use_clipboard();
    CLIPBOARD.with(|slot| slot.set(Some(clipboard)));
    rsx! {}
}

fn copied_and_failed() -> (bool, bool) {
    let clipboard = CLIPBOARD.with(Cell::get).expect("rendered");
    (*clipboard.copied.peek(), *clipboard.failed.peek())
}

/// Copies "text" through the hook and waits for the write to settle.
fn copy_through_the_hook(accept: bool) -> (&'static FakeClipboard, (bool, bool)) {
    let fake = leaked(accept);
    let _guard = fake_clipboard(fake);
    let mut dom = started(hook_app);
    dom.in_scope(ScopeId::APP, || {
        let mut clipboard = CLIPBOARD.with(Cell::get).expect("rendered");
        clipboard.copy("text");
    });
    pump_until(&mut dom, "the write to settle", |dom| {
        dom.in_runtime(copied_and_failed) != (false, false)
    });
    (fake, dom.in_runtime(copied_and_failed))
}

#[test]
fn an_accepted_write_reports_copied() {
    let (fake, flags) = copy_through_the_hook(true);
    assert_eq!(*fake.written.borrow(), ["text"]);
    assert_eq!(flags, (true, false), "(copied, failed)");
}

#[test]
fn a_denied_write_reports_failed() {
    let (fake, flags) = copy_through_the_hook(false);
    assert_eq!(*fake.written.borrow(), ["text"]);
    assert_eq!(flags, (false, true), "(copied, failed)");
}

/// The last element with a click listener: `CodeBlock`'s copy button.
#[derive(Default)]
struct LastClick {
    last: Option<ElementId>,
    click: Option<ElementId>,
}

impl WriteMutations for LastClick {
    fn push_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn set_id(&mut self, id: ElementId) {
        self.last = Some(id);
    }
    fn add_event_listener(&mut self, name: &str) {
        if name == "click" {
            self.click = self.last;
        }
    }
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_attribute(&mut self, _n: &str, _ns: Option<&str>, _v: &AttributeValue) {}
    fn set_text(&mut self, _value: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

fn code_block_app() -> Element {
    rsx! { LiberoProvider { CodeBlock { source: "let x = 1;" } } }
}

/// The text of the copy button's `role="status"` region.
fn status(dom: &VirtualDom) -> String {
    let html = dioxus_ssr::render(dom);
    let status = &html[html.find(r#"role="status""#).expect("a status region")..];
    let text = &status[status.find('>').unwrap() + 1..];
    text[..text.find('<').unwrap()].to_string()
}

/// Clicks `CodeBlock`'s copy button and returns its status once it is announced.
fn status_after_a_copy(accept: bool) -> (&'static FakeClipboard, String) {
    dioxus::html::set_event_converter(Box::new(TestConverter));
    let fake = leaked(accept);
    let _guard = fake_clipboard(fake);
    let mut dom = VirtualDom::new(code_block_app);
    let mut find = LastClick::default();
    dom.rebuild(&mut find);
    assert_eq!(status(&dom), "", "announced before a copy");
    let data = Rc::new(PlatformEventData::new(Box::new(FakeMouse))) as Rc<dyn std::any::Any>;
    let button = find.click.expect("the copy button listens");
    dom.runtime()
        .handle_event("click", Event::new(data, true), button);
    pump_until(&mut dom, "the status", |dom| !status(dom).is_empty());
    (fake, status(&dom))
}

#[test]
fn a_code_block_announces_an_accepted_copy() {
    let (fake, status) = status_after_a_copy(true);
    assert_eq!(*fake.written.borrow(), ["let x = 1;"]);
    assert_eq!(status, "Copied");
}

#[test]
fn a_code_block_announces_a_denied_copy() {
    let (fake, status) = status_after_a_copy(false);
    assert_eq!(*fake.written.borrow(), ["let x = 1;"]);
    assert_eq!(status, "Copy failed");
}
