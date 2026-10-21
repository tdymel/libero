//! The portable non-wasm timer: an OS thread sleeps and wakes a dioxus task, so
//! the callback runs on the render thread under any renderer.

use std::future::Future;
use std::ops::ControlFlow;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

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
        #[cfg(feature = "held-clock")]
        if super::held_clock::holds(delay) {
            let mut fired = super::held_clock::park(delay, false, guard.clone());
            spawn_on_root(async move {
                if fired.next().await && !guard.load(Ordering::Acquire) {
                    origin.run(callback);
                }
            });
            return Box::new(ThreadTimerSubscription { cancelled });
        }

        spawn_on_root(async move {
            sleep(delay).await;
            if !guard.load(Ordering::Acquire) {
                origin.run(callback);
            }
        });

        Box::new(ThreadTimerSubscription { cancelled })
    }

    fn every(
        &self,
        interval: Duration,
        callback: Box<dyn Fn() -> ControlFlow<()>>,
    ) -> Box<dyn TimerSubscription> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let guard = cancelled.clone();
        let origin = Origin::here();
        // A `Break` sets the flag too, so a held clock stops counting the timer.
        let stop = cancelled.clone();
        let tick = move || {
            if origin.run(&callback).is_break() {
                stop.store(true, Ordering::Release);
            }
        };
        #[cfg(feature = "held-clock")]
        if super::held_clock::holds(interval) {
            let mut fired = super::held_clock::park(interval, true, guard.clone());
            spawn_on_root(async move {
                while fired.next().await && !guard.load(Ordering::Acquire) {
                    tick();
                }
            });
            return Box::new(ThreadTimerSubscription { cancelled });
        }

        spawn_on_root(async move {
            let mut due = Instant::now();
            while !guard.load(Ordering::Acquire) {
                let now = Instant::now();
                due = next_due(due, interval, now);
                sleep(due - now).await;
                if guard.load(Ordering::Acquire) {
                    break;
                }
                tick();
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

/// The next tick on the interval's grid after `now`: ticks keep their phase instead of
/// drifting by each sleep's overshoot, and ones missed while busy are skipped, not burst.
fn next_due(last: Instant, interval: Duration, now: Instant) -> Instant {
    let next = last + interval;
    if next > now || interval.is_zero() {
        return next.max(now);
    }
    let behind = (now - next).as_nanos() / interval.as_nanos();
    next + interval * u32::try_from(behind + 1).unwrap_or(u32::MAX)
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
        let waker = alarm
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
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

        let mut waker = self
            .state
            .waker
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
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

#[cfg(test)]
mod tests {
    use std::task::Wake;
    use std::time::Instant;

    use super::*;

    struct Flag(AtomicBool);

    impl Wake for Flag {
        fn wake(self: Arc<Self>) {
            self.0.store(true, Ordering::Release);
        }
    }

    #[test]
    fn an_interval_keeps_its_phase_and_skips_missed_ticks() {
        let start = Instant::now();
        let second = Duration::from_secs(1);
        // A tick that overslept 30 ms still aims at the next whole second.
        let late = start + second + Duration::from_millis(30);
        assert_eq!(next_due(start + second, second, late), start + second * 2);
        // A dom busy for 3.5 s skips the missed ticks instead of firing them at once.
        let busy = start + Duration::from_millis(3500);
        assert_eq!(next_due(start, second, busy), start + second * 4);
        assert_eq!(next_due(start, second, start), start + second);
    }

    #[test]
    fn a_sleep_wakes_through_a_poisoned_waker_lock() {
        let mut sleep = sleep(Duration::from_millis(30));
        let state = sleep.state.clone();
        let _ = thread::spawn(move || {
            let _guard = state.waker.lock().unwrap();
            panic!("poison the waker slot");
        })
        .join();
        assert!(sleep.state.waker.is_poisoned());

        let flag = Arc::new(Flag(AtomicBool::new(false)));
        let waker = Waker::from(flag.clone());
        let mut context = Context::from_waker(&waker);
        // Under load the poisoning outlasts the sleep: the first poll is then Ready.
        if Pin::new(&mut sleep).poll(&mut context).is_ready() {
            return;
        }

        let deadline = Instant::now() + Duration::from_secs(5);
        while !flag.0.load(Ordering::Acquire) && Instant::now() < deadline {
            thread::yield_now();
        }
        assert!(flag.0.load(Ordering::Acquire), "the sleep never woke");
        assert_eq!(Pin::new(&mut sleep).poll(&mut context), Poll::Ready(()));
    }
}
