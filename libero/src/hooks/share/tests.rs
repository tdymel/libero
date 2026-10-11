//! `use_share` on a fake share sheet and a fake clipboard.

use std::cell::{Cell, RefCell};

use dioxus::prelude::*;

use super::{Share, use_share};
use crate::hooks::polling_tests::{pump_until, started};
use crate::platform::{
    ClipboardApi, PlatformError, ShareApi, ShareData, ShareOutcome, Sharing, Write, fake_clipboard,
    fake_share,
};

/// Answers every share with `answer`, and keeps what it was given.
struct FakeShare {
    answer: ShareOutcome,
    shared: RefCell<Vec<ShareData>>,
}

impl ShareApi for FakeShare {
    fn share(&self, data: ShareData) -> Sharing {
        self.shared.borrow_mut().push(data);
        Box::pin(std::future::ready(self.answer.clone()))
    }
}

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

fn fakes(answer: ShareOutcome, accept: bool) -> (&'static FakeShare, &'static FakeClipboard) {
    let share = Box::leak(Box::new(FakeShare {
        answer,
        shared: RefCell::default(),
    }));
    let clipboard = Box::leak(Box::new(FakeClipboard {
        accept,
        written: RefCell::default(),
    }));
    (share, clipboard)
}

thread_local! {
    static SHARE: Cell<Option<Share>> = const { Cell::new(None) };
}

fn hook_app() -> Element {
    let share = use_share();
    SHARE.with(|slot| slot.set(Some(share)));
    rsx! {}
}

fn the_share() -> Share {
    SHARE.with(Cell::get).expect("rendered")
}

fn data() -> ShareData {
    ShareData {
        title: Some("Libero".into()),
        ..ShareData::url("https://libero-ui.dev")
    }
}

/// Shares `data()` through the hook, pressing `presses` times, and waits for it to settle.
fn share_through_the_hook(
    answer: ShareOutcome,
    accept: bool,
    presses: usize,
) -> (
    &'static FakeShare,
    &'static FakeClipboard,
    Option<ShareOutcome>,
) {
    let (share, clipboard) = fakes(answer, accept);
    let _share = fake_share(share);
    let _clipboard = fake_clipboard(clipboard);
    let mut dom = started(hook_app);
    dom.in_scope(ScopeId::APP, || {
        for _ in 0..presses {
            the_share().share(data());
        }
    });
    pump_until(&mut dom, "the share to settle", |dom| {
        dom.in_runtime(|| the_share().outcome.peek().is_some())
    });
    let (outcome, pending) = dom.in_runtime(|| {
        let hook = the_share();
        (hook.outcome.peek().clone(), *hook.pending.peek())
    });
    assert!(!pending, "settled");
    (share, clipboard, outcome)
}

#[test]
fn a_shared_sheet_reports_shared_and_copies_nothing() {
    let (share, clipboard, outcome) = share_through_the_hook(ShareOutcome::Shared, true, 1);
    assert_eq!(*share.shared.borrow(), [data()]);
    assert!(clipboard.written.borrow().is_empty());
    assert_eq!(outcome, Some(ShareOutcome::Shared));
}

#[test]
fn no_share_sheet_copies_the_url() {
    let (_, clipboard, outcome) = share_through_the_hook(ShareOutcome::Unsupported, true, 1);
    assert_eq!(*clipboard.written.borrow(), ["https://libero-ui.dev"]);
    assert_eq!(outcome, Some(ShareOutcome::Copied));
}

#[test]
fn a_denied_fallback_copy_reports_failed() {
    let (_, clipboard, outcome) = share_through_the_hook(ShareOutcome::Unsupported, false, 1);
    assert_eq!(*clipboard.written.borrow(), ["https://libero-ui.dev"]);
    assert!(matches!(outcome, Some(ShareOutcome::Failed(_))));
}

#[test]
fn a_cancelled_sheet_neither_copies_nor_fails() {
    let (_, clipboard, outcome) = share_through_the_hook(ShareOutcome::Cancelled, true, 1);
    assert!(clipboard.written.borrow().is_empty());
    assert_eq!(outcome, Some(ShareOutcome::Cancelled));
}

#[test]
fn a_press_while_one_is_pending_is_ignored() {
    let (share, _, _) = share_through_the_hook(ShareOutcome::Shared, true, 2);
    assert_eq!(share.shared.borrow().len(), 1);
}
