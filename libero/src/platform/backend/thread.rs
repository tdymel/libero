//! The portable non-wasm timer: an OS thread sleeps and wakes a dioxus task, so
//! the callback runs on the render thread under any renderer.

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

/// `None` outside a dioxus runtime: a task delivers. SSR has one, so it gets a
/// timer that never fires.
pub(super) fn timer() -> Option<&'static dyn TimerApi> {
    #[cfg(test)]
    if let Some(manual) = super::manual_timer::active() {
        return Runtime::try_current().map(|_| manual);
    }
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
    /// A flag, not a task cancel: that would need a runtime at drop time.
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

fn spawn_on_root(future: impl Future<Output = ()> + 'static) {
    let Some(runtime) = Runtime::try_current() else {
        // Unreachable: `timer()` answers `None` without a runtime.
        debug_assert!(false, "a timer was scheduled outside a dioxus runtime");
        return;
    };

    // The root scope: the subscription owns a timer's lifetime. The callback
    // still runs as the calling scope while it lives (see `Origin`).
    runtime.spawn(ScopeId::ROOT, future);
}

/// One thread per sleep, cancelled ones sleep out too. Measured 2026-09-18: a
/// spawn 30-45 us, fast typing peaked at 6 threads. Share a scheduler if it grows.
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

        // Read again: the thread may have found the slot empty in between, and
        // the future would sleep forever.
        if self.state.elapsed.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}
