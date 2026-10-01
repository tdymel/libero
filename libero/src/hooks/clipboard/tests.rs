//! `use_clipboard` on a fake clipboard, so a test build sees both the
//! accepted and the denied write. No synthetic click: the lib tests share one
//! process-wide event converter, which another module may have swapped.

use std::cell::{Cell, RefCell};

use dioxus::prelude::*;

use super::{Clipboard, use_clipboard};
use crate::hooks::polling_tests::{pump_until, started};
use crate::platform::{ClipboardApi, PlatformError, Write, fake_clipboard};

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
    CLIPBOARD.with(|slot| slot.set(Some(use_clipboard())));
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
