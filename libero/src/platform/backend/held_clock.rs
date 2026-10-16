//! The `held-clock` feature (todo 2144): the thread timer's delays an e2e harness names wait
//! for its [`fire`], as `e2e::clock::HELD_CLOCK` holds a page's `setTimeout`. A WebView's
//! timers run here, not in the page, so its e2e app bridges these three to the driver.
//! The `_here` variants scope them to the calling thread: the native tier's parallel tests.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::task::{Context, Poll, Waker};
use std::thread::{self, ThreadId};
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
    /// The thread whose `_here` hold parked it; `None` for a process-wide hold.
    owner: Option<ThreadId>,
}

impl Parked {
    fn live(&self) -> bool {
        !self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Default)]
struct Held {
    delays: Vec<Duration>,
    scoped: Vec<(ThreadId, Vec<Duration>)>,
    parked: Vec<Parked>,
}

/// Process-wide for a WebView, whose bridge answers on another thread than its events; the
/// `_here` fns scope it to one thread, for parallel tests in one process (native, 2929).
static HELD: Mutex<Held> = Mutex::new(Held {
    delays: Vec::new(),
    scoped: Vec::new(),
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

/// Holds the timers of `delays` ms set on this thread from now on, as [`hold_timers`] does for
/// the process: for tests that share a process and each run their app on their own thread.
pub fn hold_timers_here(delays: &[u32]) {
    let me = thread::current().id();
    let delays = delays.iter().copied().map(delay).collect();
    let mut held = held();
    held.scoped.retain(|(thread, _)| *thread != me);
    held.scoped.push((me, delays));
}

/// Back to the real clock for a harness's next test: no delay held, the pending timers dropped.
pub fn reset() {
    let parked = std::mem::take(&mut *held());
    close(parked.parked);
}

/// [`reset`] for this thread's holds and timers only.
pub fn reset_here() {
    let me = thread::current().id();
    let mut held = held();
    held.scoped.retain(|(thread, _)| *thread != me);
    let (mine, others) = std::mem::take(&mut held.parked)
        .into_iter()
        .partition(|parked| parked.owner == Some(me));
    held.parked = others;
    drop(held);
    close(mine);
}

fn close(parked: Vec<Parked>) {
    for parked in parked {
        parked.gate.closed.store(true, Ordering::Release);
        parked.gate.wake();
    }
}

/// How many held timers of `ms` are pending.
pub fn armed(ms: u32) -> usize {
    count(ms, |_| true)
}

/// [`armed`] for the timers set on this thread.
pub fn armed_here(ms: u32) -> usize {
    let me = thread::current().id();
    count(ms, |parked| parked.owner == Some(me))
}

fn count(ms: u32, of: impl Fn(&Parked) -> bool) -> usize {
    let mut held = held();
    held.parked.retain(Parked::live);
    held.parked
        .iter()
        .filter(|p| p.delay == delay(ms) && of(p))
        .count()
}

/// Fires every pending held timer of `ms`, an interval staying armed; how many it released.
/// Each runs on its own thread's next poll.
pub fn fire(ms: u32) -> usize {
    release(ms, |_| true)
}

/// [`fire`] for the timers set on this thread.
pub fn fire_here(ms: u32) -> usize {
    let me = thread::current().id();
    release(ms, |parked| parked.owner == Some(me))
}

fn release(ms: u32, of: impl Fn(&Parked) -> bool) -> usize {
    let mut held = held();
    held.parked.retain(Parked::live);
    let mut fired = 0;
    held.parked.retain(|parked| {
        if parked.delay != delay(ms) || !of(parked) {
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
    let me = thread::current().id();
    let held = held();
    held.delays.contains(&delay)
        || held
            .scoped
            .iter()
            .any(|(thread, delays)| *thread == me && delays.contains(&delay))
}

/// Parks a timer of `delay`: the returned waits resolve once per [`fire`], `None` after a [`reset`].
pub(super) fn park(delay: Duration, repeats: bool, cancelled: Arc<AtomicBool>) -> Fired {
    let gate = Arc::new(Gate::default());
    let me = thread::current().id();
    let mut held = held();
    let owner = (!held.delays.contains(&delay)).then_some(me);
    held.parked.push(Parked {
        delay,
        repeats,
        cancelled,
        gate: gate.clone(),
        owner,
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

        // The `_here` fns see this thread's timers only.
        hold_timers_here(&[900]);
        let mut mine = park(delay(900), false, Arc::default());
        let mut other = thread::spawn(|| {
            hold_timers_here(&[900]);
            park(delay(900), false, Arc::default())
        })
        .join()
        .unwrap();
        assert_eq!((armed_here(900), armed(900)), (1, 2));
        assert_eq!((fire_here(900), armed(900)), (1, 1));
        assert_eq!((mine.poll_tick(), other.poll_tick()), (Some(true), None));
        reset_here();
        assert_eq!((holds(delay(900)), armed(900)), (false, 1));
        reset();
        assert_eq!(other.poll_tick(), Some(false));
    }
}
