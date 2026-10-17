use std::{rc::Rc, time::Duration};

use blitz_dom::BaseDocument;
use dioxus::core::Runtime;
use dioxus::prelude::*;
use dioxus_native_dom::NodeHandle;

use super::{Doc, Origin, anchor, doc};

pub(super) type Deferred = Box<dyn FnOnce()>;

/// Runs `run` at [`Outlet`]'s next flush, at the end of this poll.
pub(super) fn defer(anchor: &NodeHandle, run: impl FnOnce(&mut BaseDocument) + 'static) {
    let anchor = anchor.clone();
    later(move || run(&mut anchor.doc_mut()));
}

/// Makes [`Outlet`] flush at the end of this poll, sharing a pending flush.
pub(super) fn flush_soon() {
    later(|| {});
}

/// Runs `run` now if the document is free, else at [`Outlet`]'s next flush,
/// where it is: reads made there answer.
pub(in crate::platform::backend) fn when_free(run: Box<dyn FnOnce()>) {
    if anchor().is_none_or(|anchor| anchor.try_doc().is_some()) {
        run();
        return;
    }
    later(run);
}

/// Runs `run` where the document is free and laid out: effects run before the
/// shell lays out what this poll mounted, so a timer waits for the next poll.
pub(in crate::platform::backend) fn when_laid_out(run: Box<dyn FnOnce()>) {
    let Some(doc) = doc() else {
        run();
        return;
    };
    let first = {
        let mut waiting = doc.laid_out.borrow_mut();
        waiting.push(in_origin(run));
        waiting.len() == 1
    };
    if !first {
        return;
    }
    let weak = Rc::downgrade(&doc);
    let wait = crate::platform::backend::thread::timer().map(|timer| {
        timer.after(
            Duration::ZERO,
            Box::new(move || {
                let Some(doc) = weak.upgrade() else {
                    return;
                };
                let waiting = std::mem::take(&mut *doc.laid_out.borrow_mut());
                for run in waiting {
                    when_free(run);
                }
            }),
        )
    });
    drop(doc.laid_out_wait.replace(wait));
}

/// How much longer each retry waits: about a frame.
const RETRY_STEP: Duration = Duration::from_millis(16);

/// [`when_laid_out`] after `failed` retries, each a frame later than the one before: the
/// zero-delay wake can come before the shell's layout and burn every try in one frame.
pub(in crate::platform::backend) fn when_laid_out_retry(failed: u8, run: Box<dyn FnOnce()>) {
    let (Some(doc), true) = (doc(), failed > 0) else {
        return when_laid_out(run);
    };
    let Some(timer) = crate::platform::backend::thread::timer() else {
        return when_laid_out(run);
    };
    let weak = Rc::downgrade(&doc);
    doc.retrying.set(doc.retrying.get() + 1);
    let wait = timer.after(
        RETRY_STEP * u32::from(failed),
        Box::new(move || {
            let Some(doc) = weak.upgrade() else {
                return;
            };
            when_laid_out(run);
            doc.retrying.set(doc.retrying.get() - 1);
        }),
    );
    let mut waits = doc.retry_waits.borrow_mut();
    if doc.retrying.get() == 1 {
        waits.clear();
    }
    waits.push(wait);
}

pub(in crate::platform::backend) fn awaits_layout() -> bool {
    doc().is_some_and(|doc| !doc.laid_out.borrow().is_empty() || doc.retrying.get() > 0)
}

/// `run`, to run in the scope it is deferred from.
pub(super) fn in_origin(run: impl FnOnce() + 'static) -> Deferred {
    let origin = Origin::here();
    Box::new(move || origin.run(run))
}

pub(super) fn later(run: impl FnOnce() + 'static) {
    let Some(doc) = doc() else {
        return;
    };
    doc.deferred.borrow_mut().push(in_origin(run));
    // One remount per flush: replacing it before its mount event ran panics in
    // dioxus-native (664).
    let newest = doc
        .outlets
        .borrow()
        .last()
        .map(|outlet| (outlet.flushes, outlet.mounted));
    let Some((mut flushes, mounted)) = newest else {
        return;
    };
    let mut bump = move || {
        if mounted == Some(*flushes.peek()) {
            let next = flushes.peek().wrapping_add(1);
            flushes.set(next);
        }
    };
    // As `Outlet`, which owns the signal: callers sit in any scope.
    match Runtime::try_current() {
        Some(runtime) => runtime.in_scope(flushes.origin_scope(), bump),
        None => bump(),
    }
}

pub(super) fn run_deferred(doc: &Doc) {
    let deferred = std::mem::take(&mut *doc.deferred.borrow_mut());
    for run in deferred {
        run();
    }
}

/// Runs `run` now if the document is free, else at [`Outlet`]'s next flush.
pub(super) fn run_or_defer(anchor: &NodeHandle, run: impl FnOnce(&mut BaseDocument) + 'static) {
    if anchor.try_doc().is_some() {
        run(&mut anchor.doc_mut());
    } else {
        defer(anchor, run);
    }
}
