//! The `held-clock` feature (todo 2144): the thread timer's delays an e2e harness names wait
//! for its [`fire`], as `e2e::clock::HELD_CLOCK` holds a page's `setTimeout`. A WebView's
//! timers run here, not in the page, so its e2e app bridges these three to the driver.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

/// What a held timer waits on: each [`fire`] ticks it, a [`reset`] closes it.
#[derive(Default)]
struct Gate {
    ticks: AtomicU64,
    closed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl Gate {
    fn wake(&self) {
        let waker = self.waker.lock().expect("a waker slot").take();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

struct Parked {
    delay: Duration,
    repeats: bool,
    cancelled: Arc<AtomicBool>,
    gate: Arc<Gate>,
}

impl Parked {
    fn live(&self) -> bool {
        !self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Default)]
struct Held {
    delays: Vec<Duration>,
    parked: Vec<Parked>,
}

/// Process-wide: Android's WebView answers the bridge on another thread than its events.
static HELD: Mutex<Held> = Mutex::new(Held {
    delays: Vec::new(),
    parked: Vec::new(),
});

fn held() -> MutexGuard<'static, Held> {
    HELD.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn delay(ms: u32) -> Duration {
    Duration::from_millis(ms.into())
}

/// Holds the timers of `delays` ms set from now on; ones set before keep the real clock.
pub fn hold_timers(delays: &[u32]) {
    held().delays = delays.iter().copied().map(delay).collect();
}

/// Back to the real clock for a harness's next test: no delay held, the pending timers dropped.
pub fn reset() {
    let parked = std::mem::take(&mut *held());
    for parked in parked.parked {
        parked.gate.closed.store(true, Ordering::Release);
        parked.gate.wake();
    }
}

/// How many held timers of `ms` are pending.
pub fn armed(ms: u32) -> usize {
    let mut held = held();
    held.parked.retain(Parked::live);
    held.parked.iter().filter(|p| p.delay == delay(ms)).count()
}

/// Fires every pending held timer of `ms`, an interval staying armed; how many it released.
/// Each runs on its own thread's next poll.
pub fn fire(ms: u32) -> usize {
    let mut held = held();
    held.parked.retain(Parked::live);
    let mut fired = 0;
    held.parked.retain(|parked| {
        if parked.delay != delay(ms) {
            return true;
        }
        fired += 1;
        parked.gate.ticks.fetch_add(1, Ordering::AcqRel);
        parked.gate.wake();
        parked.repeats
    });
    fired
}

/// Whether a timer of `delay` set now waits for [`fire`].
pub(super) fn holds(delay: Duration) -> bool {
    held().delays.contains(&delay)
}

/// Parks a timer of `delay`: the returned waits resolve once per [`fire`], `None` after a [`reset`].
pub(super) fn park(delay: Duration, repeats: bool, cancelled: Arc<AtomicBool>) -> Fired {
    let gate = Arc::new(Gate::default());
    held().parked.push(Parked {
        delay,
        repeats,
        cancelled,
        gate: gate.clone(),
    });
    Fired { gate, seen: 0 }
}

/// A parked timer's side: [`Fired::next`] waits for its next [`fire`].
pub(super) struct Fired {
    gate: Arc<Gate>,
    seen: u64,
}

impl Fired {
    /// `true` per fire, `false` once a [`reset`] dropped the timer.
    pub(super) fn next(&mut self) -> impl Future<Output = bool> + '_ {
        Next { fired: self }
    }

    fn poll_tick(&mut self) -> Option<bool> {
        let ticks = self.gate.ticks.load(Ordering::Acquire);
        if ticks > self.seen {
            self.seen += 1;
            return Some(true);
        }
        self.gate.closed.load(Ordering::Acquire).then_some(false)
    }
}

struct Next<'a> {
    fired: &'a mut Fired,
}

impl Future for Next<'_> {
    type Output = bool;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<bool> {
        if let Some(tick) = self.fired.poll_tick() {
            return Poll::Ready(tick);
        }
        *self.fired.gate.waker.lock().expect("a waker slot") = Some(context.waker().clone());
        // Read again: a fire between the read and the slot found no waker.
        match self.fired.poll_tick() {
            Some(tick) => Poll::Ready(tick),
            None => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // One test: the state is process-wide and the harness runs tests in parallel.
    #[test]
    fn held_timers_wait_for_their_fire() {
        hold_timers(&[800]);
        assert!(holds(delay(800)) && !holds(delay(300)));
        let mut once = park(delay(800), false, Arc::default());
        assert_eq!((armed(800), armed(300)), (1, 0));
        assert_eq!(once.poll_tick(), None);
        assert_eq!((fire(300), fire(800)), (0, 1));
        assert_eq!((once.poll_tick(), armed(800)), (Some(true), 0));

        let cancelled = Arc::new(AtomicBool::new(true));
        park(delay(500), false, cancelled);
        assert_eq!((armed(500), fire(500)), (0, 0));

        let mut every = park(delay(100), true, Arc::default());
        assert_eq!((fire(100), fire(100)), (1, 1));
        assert_eq!(
            (every.poll_tick(), every.poll_tick(), every.poll_tick()),
            (Some(true), Some(true), None)
        );
        assert_eq!(armed(100), 1);
        reset();
        assert_eq!((every.poll_tick(), armed(100)), (Some(false), 0));
        assert!(!holds(delay(800)));
    }
}
