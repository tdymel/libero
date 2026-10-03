//! A test clock: while [`install`]ed on a thread, its timers fire only when the
//! test moves time on with [`advance_to`], so a test waits on nothing real.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::platform::{TimerApi, TimerSubscription};

enum Callback {
    Once(Box<dyn FnOnce()>),
    Every(Rc<dyn Fn()>, Duration),
}

struct Pending {
    id: u64,
    due: Duration,
    callback: Callback,
}

#[derive(Default)]
struct Clock {
    now: Duration,
    next_id: u64,
    pending: Vec<Pending>,
}

thread_local! {
    static CLOCK: RefCell<Option<Clock>> = const { RefCell::new(None) };
}

/// Uninstalls the clock when dropped.
pub(crate) struct Installed;

impl Drop for Installed {
    fn drop(&mut self) {
        let clock = CLOCK.with(|clock| clock.borrow_mut().take());
        drop(clock);
    }
}

/// Answers every `timer()` on this thread from now on, with time standing at 0.
pub(crate) fn install() -> Installed {
    CLOCK.with(|clock| *clock.borrow_mut() = Some(Clock::default()));
    Installed
}

pub(super) fn active() -> Option<&'static dyn TimerApi> {
    let installed = CLOCK.with(|clock| clock.borrow().is_some());
    installed.then_some(&MANUAL as &'static dyn TimerApi)
}

/// How long the clock has run.
pub(crate) fn now() -> Duration {
    CLOCK.with(|clock| {
        clock
            .borrow()
            .as_ref()
            .map_or(Duration::ZERO, |clock| clock.now)
    })
}

/// When the next timer is due, if one is set.
pub(crate) fn next_due() -> Option<Duration> {
    CLOCK.with(|clock| {
        let clock = clock.borrow();
        clock.as_ref()?.pending.iter().map(|entry| entry.due).min()
    })
}

/// Moves time to `time`, firing every timer due by then in order, each with
/// the clock released so it may set or cancel timers.
pub(crate) fn advance_to(time: Duration) {
    loop {
        let due = CLOCK.with(|clock| {
            let mut clock = clock.borrow_mut();
            let clock = clock.as_mut()?;
            let index = (0..clock.pending.len())
                .filter(|&index| clock.pending[index].due <= time)
                .min_by_key(|&index| (clock.pending[index].due, clock.pending[index].id))?;
            let entry = clock.pending.remove(index);
            clock.now = entry.due;
            Some(entry)
        });
        let Some(entry) = due else {
            break;
        };
        match entry.callback {
            Callback::Once(callback) => callback(),
            Callback::Every(callback, interval) => {
                CLOCK.with(|clock| {
                    if let Some(clock) = clock.borrow_mut().as_mut() {
                        clock.pending.push(Pending {
                            id: entry.id,
                            due: entry.due + interval,
                            callback: Callback::Every(callback.clone(), interval),
                        });
                    }
                });
                callback();
            }
        }
    }
    CLOCK.with(|clock| {
        if let Some(clock) = clock.borrow_mut().as_mut() {
            clock.now = clock.now.max(time);
        }
    });
}

struct ManualTimer;

static MANUAL: ManualTimer = ManualTimer;

fn schedule(delay: Duration, callback: Callback) -> Box<dyn TimerSubscription> {
    let id = CLOCK.with(|clock| {
        let mut clock = clock.borrow_mut();
        let clock = clock.as_mut().expect("the manual clock is installed");
        let id = clock.next_id;
        clock.next_id += 1;
        let due = clock.now + delay;
        clock.pending.push(Pending { id, due, callback });
        id
    });
    Box::new(Subscription { id })
}

impl TimerApi for ManualTimer {
    fn after(&self, delay: Duration, callback: Box<dyn FnOnce()>) -> Box<dyn TimerSubscription> {
        schedule(delay, Callback::Once(callback))
    }

    fn every(&self, interval: Duration, callback: Box<dyn Fn()>) -> Box<dyn TimerSubscription> {
        schedule(interval, Callback::Every(Rc::from(callback), interval))
    }
}

struct Subscription {
    id: u64,
}

impl TimerSubscription for Subscription {}

impl Drop for Subscription {
    fn drop(&mut self) {
        let removed = CLOCK.with(|clock| {
            let mut clock = clock.try_borrow_mut().ok()?;
            let clock = clock.as_mut()?;
            let index = clock.pending.iter().position(|entry| entry.id == self.id)?;
            Some(clock.pending.remove(index))
        });
        drop(removed);
    }
}
