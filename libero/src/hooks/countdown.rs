//! `use_countdown` on one ticker that every mounted countdown shares, and
//! `format_duration` in the active language.

use std::cell::RefCell;
use std::ops::ControlFlow;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;

use super::current_localization;
use crate::localization::DurationStyle;
use crate::platform::{TimerSubscription, timer, unix_millis};

const SECOND_MS: u64 = 1000;

struct Entry {
    id: u64,
    end_ms: u64,
    left: Signal<Duration>,
}

#[derive(Default)]
struct Ticker {
    next_id: u64,
    entries: Vec<Entry>,
    /// The timeout up to the next whole second, then the 1 s interval.
    timer: Option<Box<dyn TimerSubscription>>,
    /// False once every countdown reached zero and the interval stopped itself.
    running: bool,
    /// The timeout, swapped out inside its own callback: dropping it there would free
    /// the closure the web is running. Dropped at the next tick.
    retired: Option<Box<dyn TimerSubscription>>,
}

thread_local! {
    static TICKER: RefCell<Ticker> = RefCell::new(Ticker::default());
}

/// Milliseconds since the Unix epoch; the manual clock's under test.
fn now_ms() -> u64 {
    #[cfg(all(test, not(target_arch = "wasm32")))]
    if crate::platform::manual_timer::installed() {
        let elapsed = crate::platform::manual_timer::now().as_millis() as u64;
        return tests::EPOCH_MS + elapsed;
    }
    unix_millis()
}

fn end_ms(end: SystemTime) -> u64 {
    end.duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as u64)
}

/// The time left at `now`, rounded up to the whole second: zero only once the end passed.
fn left(end_ms: u64, now: u64) -> Duration {
    Duration::from_secs(end_ms.saturating_sub(now).div_ceil(SECOND_MS))
}

/// Writes every countdown's time left; whether any is above zero. Runs outside every
/// scope, so it only writes root-owned signals.
fn tick() -> bool {
    let now = now_ms();
    let (updates, retired) = TICKER.with_borrow_mut(|ticker| {
        let updates: Vec<_> = ticker
            .entries
            .iter()
            .map(|entry| (entry.left, left(entry.end_ms, now)))
            .collect();
        (updates, ticker.retired.take())
    });
    drop(retired);
    let mut live = false;
    for (mut signal, value) in updates {
        live |= !value.is_zero();
        if *signal.peek() != value {
            signal.set(value);
        }
    }
    live
}

fn every_second() -> Option<Box<dyn TimerSubscription>> {
    let api = timer()?;
    Some(api.every(
        Duration::from_millis(SECOND_MS),
        Box::new(|| {
            if tick() {
                return ControlFlow::Continue(());
            }
            TICKER.with_borrow_mut(|ticker| ticker.running = false);
            ControlFlow::Break(())
        }),
    ))
}

/// The ticker, starting at the next whole second so all labels turn together.
fn start() -> Option<Box<dyn TimerSubscription>> {
    let into_second = now_ms() % SECOND_MS;
    if into_second == 0 {
        return every_second();
    }
    let api = timer()?;
    Some(api.after(
        Duration::from_millis(SECOND_MS - into_second),
        Box::new(|| {
            let live = tick();
            let interval = live.then(every_second).flatten();
            TICKER.with_borrow_mut(|ticker| {
                ticker.running = interval.is_some();
                ticker.retired = std::mem::replace(&mut ticker.timer, interval);
            });
        }),
    ))
}

/// Adds or moves countdown `id`, and starts the ticker if it is live and the ticker is not.
fn register(id: u64, end_ms: u64, mut signal: Signal<Duration>) {
    let value = left(end_ms, now_ms());
    if *signal.peek() != value {
        signal.set(value);
    }
    let stale = TICKER.with_borrow_mut(|ticker| {
        match ticker.entries.iter_mut().find(|entry| entry.id == id) {
            Some(entry) => entry.end_ms = end_ms,
            None => ticker.entries.push(Entry {
                id,
                end_ms,
                left: signal,
            }),
        }
        if ticker.running || value.is_zero() {
            return None;
        }
        let stale = (ticker.timer.take(), ticker.retired.take());
        ticker.timer = start();
        ticker.running = ticker.timer.is_some();
        Some(stale)
    });
    drop(stale);
}

/// Removes countdown `id`; the last one out stops the ticker.
fn unregister(id: u64) {
    let stale = TICKER.with_borrow_mut(|ticker| {
        ticker.entries.retain(|entry| entry.id != id);
        if !ticker.entries.is_empty() {
            return None;
        }
        ticker.running = false;
        Some((ticker.timer.take(), ticker.retired.take()))
    });
    drop(stale);
}

/// The time left until `end`, live, in whole seconds rounded up: `Duration::ZERO`
/// once it passed. Every mounted countdown shares one ticker, aligned to the
/// second, which stops when none is mounted or all reached zero. A new `end`
/// restarts it. Build `end` from [`ClockApi::now`](crate::platform::ClockApi::now):
/// `SystemTime::now()` panics on wasm.
///
/// Spell the time out for a screen reader with [`DurationStyle::Spoken`], and keep
/// the label out of a live region: it would speak every second.
///
/// ```rust
/// # use std::time::{Duration, UNIX_EPOCH};
/// # use dioxus::prelude::*;
/// # use libero::{
/// #     components::VisuallyHidden,
/// #     hooks::{format_duration, use_countdown},
/// #     localization::DurationStyle,
/// #     platform::clock,
/// # };
/// # fn app() -> Element {
/// let end = use_hook(|| clock().map_or(UNIX_EPOCH, |clock| clock.now()) + Duration::from_secs(90));
/// let left = use_countdown(end);
///
/// rsx! {
///     span { aria_hidden: "true", {format_duration(left(), DurationStyle::Clock)} }
///     VisuallyHidden { {format_duration(left(), DurationStyle::Spoken)} }
/// }
/// # }
/// ```
pub fn use_countdown(end: SystemTime) -> ReadSignal<Duration> {
    let end_ms = end_ms(end);
    let left_signal = use_hook(|| Signal::new_in_scope(left(end_ms, now_ms()), ScopeId::ROOT));
    let id = use_hook(|| {
        TICKER.with_borrow_mut(|ticker| {
            ticker.next_id += 1;
            ticker.next_id
        })
    });
    use_effect(use_reactive!(|end_ms| register(id, end_ms, left_signal)));
    use_drop(move || {
        unregister(id);
        left_signal.manually_drop();
    });
    left_signal.into()
}

/// `span` as text in the active language:
/// [`DateLocale::format_duration`](crate::localization::DateLocale::format_duration)
/// for it. Subscribes the caller to a language switch; panics outside a `LiberoProvider`.
///
/// ```rust
/// # use std::time::Duration;
/// # use dioxus::prelude::*;
/// # use libero::{hooks::format_duration, localization::DurationStyle};
/// # fn app() -> Element {
/// let span = Duration::from_secs(65);
/// rsx! {
///     span { aria_label: format_duration(span, DurationStyle::Spoken), {format_duration(span, DurationStyle::Clock)} }
/// }
/// # }
/// ```
pub fn format_duration(span: Duration, style: DurationStyle) -> String {
    current_localization().date.format_duration(span, style)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(super) mod tests {
    use std::cell::Cell;

    use super::*;
    use crate::hooks::polling_tests::{elapse, flush, started};
    use crate::platform::manual_timer;

    /// A quarter second past a whole one, so the first tick is 750 ms in.
    pub(crate) const EPOCH_MS: u64 = 1_700_000_000_250;

    thread_local! {
        static LEFT: RefCell<Vec<ReadSignal<Duration>>> = const { RefCell::new(Vec::new()) };
        static SHOW: RefCell<Option<Signal<usize>>> = const { RefCell::new(None) };
        /// When the countdowns mounted next count from, in ms after the epoch.
        static BASE: Cell<u64> = const { Cell::new(0) };
    }

    fn at(ms: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_millis(EPOCH_MS + BASE.get() + ms)
    }

    #[component]
    fn One(ends_in_ms: u64) -> Element {
        let left = use_countdown(at(ends_in_ms));
        use_hook(|| LEFT.with_borrow_mut(|all| all.push(left)));
        rsx! {}
    }

    /// Fifty countdowns ending 1 to 3.5 s in, as many as `SHOW` says.
    fn fifty() -> Element {
        let show = use_signal(|| 50);
        use_hook(|| SHOW.set(Some(show)));
        rsx! {
            for index in 0..show() {
                One { key: "{index}", ends_in_ms: 1000 + index as u64 % 6 * 500 }
            }
        }
    }

    fn seconds_left() -> Vec<u64> {
        LEFT.with_borrow(|all| all.iter().map(|left| left.peek().as_secs()).collect())
    }

    fn show(dom: &mut VirtualDom, count: usize) {
        let mut show = SHOW.with_borrow(|show| show.expect("mounted"));
        dom.in_runtime(|| show.set(count));
        flush(dom);
    }

    #[test]
    fn fifty_countdowns_share_one_ticker_that_stops_at_zero() {
        let _clock = manual_timer::install();
        let mut dom = started(fifty);
        assert_eq!(
            manual_timer::pending(),
            1,
            "one ticker for fifty countdowns"
        );
        assert_eq!(seconds_left()[..6], [1, 2, 2, 3, 3, 4]);

        // The first tick lands on the whole second, 750 ms in.
        elapse(&mut dom, 749);
        assert_eq!(seconds_left()[..2], [1, 2]);
        elapse(&mut dom, 1);
        assert_eq!(seconds_left()[..6], [1, 1, 2, 2, 3, 3]);
        assert_eq!(
            manual_timer::pending(),
            1,
            "the interval replaced the timeout"
        );

        elapse(&mut dom, 3000);
        assert!(
            seconds_left().iter().all(|left| *left == 0),
            "{:?}",
            seconds_left()
        );
        assert_eq!(manual_timer::pending(), 0, "the ticker stopped at zero");
    }

    #[test]
    fn the_last_countdown_unmounted_stops_the_ticker() {
        let _clock = manual_timer::install();
        let mut dom = started(fifty);
        elapse(&mut dom, 750);
        show(&mut dom, 1);
        assert_eq!(manual_timer::pending(), 1);
        show(&mut dom, 0);
        assert_eq!(manual_timer::pending(), 0);
    }

    #[test]
    fn a_countdown_mounted_after_all_reached_zero_restarts_the_ticker() {
        let _clock = manual_timer::install();
        let mut dom = started(fifty);
        elapse(&mut dom, 4000);
        assert_eq!(manual_timer::pending(), 0);

        show(&mut dom, 0);
        LEFT.with_borrow_mut(Vec::clear);
        BASE.set(4000);
        show(&mut dom, 2);
        assert_eq!(manual_timer::pending(), 1, "the ticker restarted");
        assert_eq!(seconds_left(), [1, 2]);
        elapse(&mut dom, 2000);
        assert_eq!(seconds_left(), [0, 0]);
        assert_eq!(manual_timer::pending(), 0);
    }

    #[test]
    fn the_time_left_rounds_up_to_the_second() {
        assert_eq!(left(10_000, 10_000), Duration::ZERO);
        assert_eq!(left(10_000, 9_999), Duration::from_secs(1));
        assert_eq!(left(10_000, 9_000), Duration::from_secs(1));
        assert_eq!(left(10_000, 8_999), Duration::from_secs(2));
        assert_eq!(left(10_000, 20_000), Duration::ZERO);
    }
}
