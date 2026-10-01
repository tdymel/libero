//! `use_timeout` and `use_interval` on the platform timer, plus [`Scheduled`],
//! the one timer cell the debounce and throttle hooks share.

use std::time::Duration;

use dioxus::prelude::*;

use crate::platform::{TimerSubscription, timer};

/// A timer whose firing is observed by an effect in the caller's scope. The
/// platform callback runs outside every scope and on the web without a runtime,
/// so it only bumps a root-owned counter ([[codebase/platform/platform-timer]]).
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Scheduled {
    fired: Signal<u32>,
    /// Replacing or dropping the subscription cancels it.
    pending: CopyValue<Option<Box<dyn TimerSubscription>>>,
}

impl Scheduled {
    /// Fires once in `ms`, cancelling a timer under way.
    pub(crate) fn after(&self, ms: u64) {
        let fired = self.fired;
        self.replace(|timer| timer.after(Duration::from_millis(ms), Box::new(move || bump(fired))));
    }

    /// Fires every `ms`, cancelling a timer under way.
    pub(crate) fn every(&self, ms: u64) {
        let fired = self.fired;
        self.replace(|timer| timer.every(Duration::from_millis(ms), Box::new(move || bump(fired))));
    }

    pub(crate) fn cancel(&self) {
        let mut pending = self.pending;
        pending.set(None);
    }

    fn replace(
        &self,
        make: impl FnOnce(&'static dyn crate::platform::TimerApi) -> Box<dyn TimerSubscription>,
    ) {
        let mut pending = self.pending;
        pending.set(timer().map(make));
    }
}

fn bump(fired: Signal<u32>) {
    let mut fired = fired;
    fired += 1;
}

/// A [`Scheduled`] that calls `on_fire` in this scope each time it fires. Two
/// firings before a render coalesce into one call.
pub(crate) fn use_scheduled(mut on_fire: impl FnMut(Scheduled) + 'static) -> Scheduled {
    let fired = use_hook(|| Signal::new_in_scope(0, ScopeId::ROOT));
    let pending = use_hook(|| CopyValue::new(None::<Box<dyn TimerSubscription>>));
    let mut seen = use_hook(|| CopyValue::new(0u32));
    let scheduled = Scheduled { fired, pending };
    use_drop(move || {
        scheduled.cancel();
        fired.manually_drop();
    });
    // `on_fire` reads subscribe this effect too: a re-run without a new firing is not one.
    use_effect(move || {
        let count = fired();
        if count != *seen.peek() {
            seen.set(count);
            on_fire(scheduled);
        }
    });
    scheduled
}

/// `ms` as of the latest render, for a closure a hook keeps from the first.
pub(crate) fn use_latest_ms(ms: u64) -> CopyValue<u64> {
    let mut latest = use_hook(|| CopyValue::new(ms));
    latest.set(ms);
    latest
}

/// The closure a hook keeps for its latest render.
pub(crate) type Slot<A> = Box<dyn FnMut(A)>;

/// Calls the closure in `slot` with the borrow released, so the call may render.
pub(crate) fn run_slot<A: 'static>(mut slot: CopyValue<Option<Slot<A>>>, arg: A) {
    let taken = slot.write().take();
    if let Some(mut callback) = taken {
        callback(arg);
        if slot.peek().is_none() {
            slot.set(Some(callback));
        }
    }
}

/// Runs a callback once, `ms` after [`start`](TimeoutHandle::start).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_timeout;
/// # fn app() -> Element {
/// let mut shown = use_signal(|| false);
/// let timeout = use_timeout(move || shown.set(false), 2000);
///
/// rsx! {
///     button {
///         onclick: move |_| {
///             shown.set(true);
///             timeout.start();
///         },
///         "Show for two seconds"
///     }
///     if shown() {
///         p { "Hello" }
///     }
/// }
/// # }
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct TimeoutHandle {
    scheduled: Scheduled,
    active: Signal<bool>,
    ms: u64,
}

impl TimeoutHandle {
    /// Starts the countdown, or restarts one under way.
    pub fn start(&self) {
        let mut active = self.active;
        active.set(true);
        self.scheduled.after(self.ms);
    }

    /// Cancels a countdown under way; the callback does not run.
    pub fn stop(&self) {
        let mut active = self.active;
        active.set(false);
        self.scheduled.cancel();
    }

    /// Whether a countdown is under way. Reactive.
    pub fn pending(&self) -> bool {
        (self.active)()
    }
}

/// Calls `callback` once, `ms` after [`TimeoutHandle::start`]. The countdown is
/// cancelled when the component unmounts. A target without a timer, such as a
/// server render, never fires it.
pub fn use_timeout(mut callback: impl FnMut() + 'static, ms: u64) -> TimeoutHandle {
    let active = use_signal(|| false);
    let mut slot = use_hook(|| CopyValue::new(None::<Slot<()>>));
    // The boxed closure is replaced every render, so it sees current state.
    slot.set(Some(Box::new(move |()| callback())));
    let scheduled = use_scheduled(move |_| {
        let mut active = active;
        active.set(false);
        run_slot(slot, ());
    });
    TimeoutHandle {
        scheduled,
        active,
        ms,
    }
}

/// Runs a callback every `ms` between [`start`](IntervalHandle::start) and
/// [`stop`](IntervalHandle::stop).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_interval;
/// # fn app() -> Element {
/// let mut seconds = use_signal(|| 0);
/// let interval = use_interval(move || seconds += 1, 1000);
///
/// rsx! {
///     button { onclick: move |_| interval.toggle(),
///         if interval.active() { "Stop" } else { "Start" }
///     }
///     p { "{seconds}" }
/// }
/// # }
/// ```
#[derive(Clone, Copy, PartialEq)]
pub struct IntervalHandle {
    scheduled: Scheduled,
    active: Signal<bool>,
    ms: u64,
}

impl IntervalHandle {
    /// Starts ticking, or restarts the period of one under way.
    pub fn start(&self) {
        let mut active = self.active;
        active.set(true);
        self.scheduled.every(self.ms);
    }

    /// Stops ticking.
    pub fn stop(&self) {
        let mut active = self.active;
        active.set(false);
        self.scheduled.cancel();
    }

    /// Stops a running interval, starts a stopped one.
    pub fn toggle(&self) {
        let running = *self.active.peek();
        match running {
            true => self.stop(),
            false => self.start(),
        }
    }

    /// Whether it is ticking. Reactive.
    pub fn active(&self) -> bool {
        (self.active)()
    }
}

/// Calls `callback` every `ms` once [`IntervalHandle::start`] ran, until
/// [`stop`](IntervalHandle::stop) or the component unmounts. It does not start
/// itself: call `start` from a handler or `use_hook`. A tick that lands while
/// the component is still rendering the last one is skipped. Natively every
/// tick costs a thread, so keep the period to a second or more.
pub fn use_interval(mut callback: impl FnMut() + 'static, ms: u64) -> IntervalHandle {
    let active = use_signal(|| false);
    let mut slot = use_hook(|| CopyValue::new(None::<Slot<()>>));
    slot.set(Some(Box::new(move |()| callback())));
    let scheduled = use_scheduled(move |_| {
        run_slot(slot, ());
    });
    IntervalHandle {
        scheduled,
        active,
        ms,
    }
}
