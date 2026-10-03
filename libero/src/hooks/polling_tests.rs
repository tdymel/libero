//! Drives a `VirtualDom` on the real non-wasm timer the way a renderer would, waiting on
//! conditions and timers rather than fixed spans.

use std::cell::Cell;
use std::thread;
use std::time::{Duration, Instant};

use dioxus::prelude::*;

use crate::platform::manual_timer;

thread_local! {
    static SETTLED: Cell<bool> = const { Cell::new(false) };
}

/// Bounds a wait for what should happen, so a broken hook fails rather than hangs.
const LIMIT: Duration = Duration::from_secs(2);

/// Builds and renders `app`, its first effects run.
pub(crate) fn started(app: fn() -> Element) -> VirtualDom {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    flush(&mut dom);
    dom
}

/// Runs what is queued now and renders it, without waiting for a timer.
pub(crate) fn flush(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

/// Polls the dom until `done`, and says how long that took.
pub(crate) fn pump_until(
    dom: &mut VirtualDom,
    what: &str,
    done: impl Fn(&VirtualDom) -> bool,
) -> Duration {
    let start = Instant::now();
    while !done(dom) {
        assert!(start.elapsed() < LIMIT, "{what}: not within {LIMIT:?}");
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        thread::sleep(Duration::from_millis(1));
    }
    start.elapsed()
}

/// Moves an installed [`manual_timer`] on by `ms`, rendering after each timer
/// that fires, as a renderer would. Waits on nothing real.
pub(crate) fn elapse(dom: &mut VirtualDom, ms: u64) {
    let target = manual_timer::now() + Duration::from_millis(ms);
    flush(dom);
    while let Some(due) = manual_timer::next_due().filter(|due| *due <= target) {
        dom.in_runtime(|| manual_timer::advance_to(due));
        flush(dom);
    }
    dom.in_runtime(|| manual_timer::advance_to(target));
    flush(dom);
}

/// Polls through a gap the scenario sets, like the time between two keystrokes.
pub(crate) fn pump(dom: &mut VirtualDom, ms: u64) {
    let start = Instant::now();
    pump_until(dom, "a gap", |_| {
        start.elapsed() >= Duration::from_millis(ms)
    });
}

/// Polls until a timer of `ms` started now has fired: one started after it has.
pub(crate) fn settle(dom: &mut VirtualDom, ms: u64) {
    SETTLED.with(|settled| settled.set(false));
    let _sentinel = dom.in_scope(ScopeId::APP, || {
        crate::platform::timer().expect("a timer").after(
            Duration::from_millis(ms + 20),
            Box::new(|| SETTLED.with(|settled| settled.set(true))),
        )
    });
    pump_until(dom, "the sentinel timer", |_| SETTLED.with(Cell::get));
}
