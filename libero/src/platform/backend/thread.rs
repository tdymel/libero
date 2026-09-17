//! The portable non-wasm timer, the way [`mounted`](super::mounted) is the
//! portable floor for elements: an OS thread that sleeps, and a dioxus task
//! that delivers.
//!
//! **The thread is the easy half.** A timer callback writes signals, so it has
//! to run on the thread dioxus renders on - and no timer crate does that hop
//! for us. The one portable answer is to await inside a dioxus task and let the
//! sleeping thread only call `waker.wake()`: dioxus wires its wakers to
//! whatever event loop the renderer runs, so the callback lands on the right
//! thread under Blitz and the WebView floor alike. That is why this needs no
//! dependency - what a timer crate sells is the timer wheel, which is the part
//! we do not need.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

use dioxus::core::Runtime;
use dioxus::prelude::ScopeId;

use super::origin::Origin;
use crate::platform::{TimerApi, TimerSubscription};

/// `None` outside a dioxus runtime, because delivery goes through a dioxus
/// task - the same house rule as any other absent capability. That is a bare
/// test, not a server render: SSR has a runtime, so it gets a timer that
/// schedules and never fires.
pub(super) fn timer() -> Option<&'static dyn TimerApi> {
    Runtime::try_current().map(|_| &TIMER as &'static dyn TimerApi)
}

struct ThreadTimer;

static TIMER: ThreadTimer = ThreadTimer;

impl TimerApi for ThreadTimer {
    fn after(&self, delay: Duration, callback: Box<dyn FnOnce()>) -> Box<dyn TimerSubscription> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let guard = cancelled.clone();
        let origin = Origin::here();

        spawn_on_root(async move {
            sleep(delay).await;
            if !guard.load(Ordering::Acquire) {
                origin.run(callback);
            }
        });

        Box::new(ThreadTimerSubscription { cancelled })
    }

    fn every(&self, interval: Duration, callback: Box<dyn Fn()>) -> Box<dyn TimerSubscription> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let guard = cancelled.clone();
        let origin = Origin::here();

        spawn_on_root(async move {
            loop {
                sleep(interval).await;
                if guard.load(Ordering::Acquire) {
                    break;
                }
                origin.run(&callback);
            }
        });

        Box::new(ThreadTimerSubscription { cancelled })
    }
}

struct ThreadTimerSubscription {
    cancelled: Arc<AtomicBool>,
}

impl TimerSubscription for ThreadTimerSubscription {}

impl Drop for ThreadTimerSubscription {
    /// A flag rather than cancelling the task, because cancelling would need a
    /// runtime at *drop* time and a drop runs wherever its owner does.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

fn spawn_on_root(future: impl Future<Output = ()> + 'static) {
    let Some(runtime) = Runtime::try_current() else {
        // Unreachable: `timer()` already answered `None` without a runtime. If
        // it ever happens the caller holds a subscription that can never fire,
        // which is worth failing loudly for in a debug build.
        debug_assert!(false, "a timer was scheduled outside a dioxus runtime");
        return;
    };

    // The root scope, not the calling one: what ties a timer's lifetime is its
    // subscription, not whichever scope happened to be rendering. The callback
    // still runs as the calling scope while that lives (see `Origin`).
    runtime.spawn(ScopeId::ROOT, future);
}

/// One thread per sleep, and a cancelled timer's still sleeps out its delay:
/// typeahead restarts one per keystroke and `Menu` one per hover, so the live
/// count is the delay times the event rate. Measured 2026-09-18 (release):
/// a spawn costs 30-45 us, off the render path, and keys 15 ms apart under a
/// 100 ms pause peaked at 6 extra threads, all gone after the pause. A shared
/// scheduler is the thing to build if that ever grows.
fn sleep(duration: Duration) -> Sleep {
    let state = Arc::new(SleepState {
        elapsed: AtomicBool::new(false),
        waker: Mutex::new(None),
    });
    let alarm = state.clone();

    thread::spawn(move || {
        thread::sleep(duration);
        alarm.elapsed.store(true, Ordering::Release);
        let waker = alarm.waker.lock().expect("a waker slot").take();
        if let Some(waker) = waker {
            waker.wake();
        }
    });

    Sleep { state }
}

struct SleepState {
    elapsed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

struct Sleep {
    state: Arc<SleepState>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.state.elapsed.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        let mut waker = self.state.waker.lock().expect("a waker slot");
        *waker = Some(context.waker().clone());
        drop(waker);

        // Read again: the thread may have taken an empty slot and woken nobody
        // between the load above and the store. Without this the future sleeps
        // forever, which is the classic shape of this bug.
        if self.state.elapsed.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
