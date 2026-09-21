use std::time::Duration;

use super::backend;

/// A live timer. Dropping it cancels, so no generation guard is needed.
pub trait TimerSubscription {}

/// Being called back later. The callback runs outside every scope: anything it
/// writes must be a signal that outlives it (`Signal::new_in_scope`, root).
pub trait TimerApi {
    /// Calls `callback` once, `delay` from now, unless the returned
    /// subscription is dropped first.
    fn after(&self, delay: Duration, callback: Box<dyn FnOnce()>) -> Box<dyn TimerSubscription>;

    /// Calls `callback` every `interval` until the returned subscription is
    /// dropped.
    fn every(&self, interval: Duration, callback: Box<dyn Fn()>) -> Box<dyn TimerSubscription>;
}

/// The timer: `setTimeout` on the web, a thread and a dioxus task elsewhere.
/// `None` outside a runtime; a server render is `Some` and never fires.
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use std::time::Duration;
/// # fn app() -> Element {
/// let _autoplay = use_hook(|| {
///     libero::platform::timer().map(|timer| {
///         std::rc::Rc::new(timer.every(Duration::from_secs(5), Box::new(|| {})))
///     })
/// });
/// # rsx! {}
/// # }
/// ```
pub fn timer() -> Option<&'static dyn TimerApi> {
    backend::timer()
}

/// Against the real non-wasm arm: a real thread, sleep and `VirtualDom`. Each
/// test has its own thread, so the `thread_local` state is isolated.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::thread;
    use std::time::Instant;

    use dioxus::prelude::*;

    use super::*;

    thread_local! {
        static FIRED: Cell<usize> = const { Cell::new(0) };
        /// The subscription the test drops to cancel, so it owns that moment.
        static SUBSCRIPTION: RefCell<Option<Box<dyn TimerSubscription>>> =
            const { RefCell::new(None) };
    }

    fn count_one_fire() {
        FIRED.with(|fired| fired.set(fired.get() + 1));
    }

    fn fires() -> usize {
        FIRED.with(Cell::get)
    }

    fn keep(subscription: Box<dyn TimerSubscription>) {
        SUBSCRIPTION.with(|slot| *slot.borrow_mut() = Some(subscription));
    }

    fn drop_subscription() {
        SUBSCRIPTION.with(|slot| slot.borrow_mut().take());
    }

    fn delayed() -> Element {
        use_hook(|| {
            let api = timer().expect("a timer inside a VirtualDom");
            keep(api.after(Duration::from_millis(20), Box::new(count_one_fire)));
        });
        rsx! {}
    }

    fn repeating() -> Element {
        use_hook(|| {
            let api = timer().expect("a timer inside a VirtualDom");
            keep(api.every(Duration::from_millis(10), Box::new(count_one_fire)));
        });
        rsx! {}
    }

    /// Polls the dom until `done` or `limit`, standing in for the renderer.
    fn drive(dom: &mut VirtualDom, limit: Duration, done: impl Fn() -> bool) {
        let start = Instant::now();
        while start.elapsed() < limit {
            dom.process_events();
            if done() {
                return;
            }
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn a_timer_fires_once_after_its_delay() {
        let mut dom = VirtualDom::new(delayed);
        dom.rebuild_in_place();

        drive(&mut dom, Duration::from_secs(2), || fires() == 1);
        assert_eq!(fires(), 1, "the callback never arrived");

        // And exactly once, not once per poll.
        drive(&mut dom, Duration::from_millis(100), || false);
        assert_eq!(fires(), 1);

        drop_subscription();
    }

    #[test]
    fn dropping_the_subscription_cancels_the_timer() {
        let mut dom = VirtualDom::new(delayed);
        dom.rebuild_in_place();

        drop_subscription();

        // Well past the 20ms it was scheduled for.
        drive(&mut dom, Duration::from_millis(300), || false);
        assert_eq!(fires(), 0, "a cancelled timer fired anyway");
    }

    #[test]
    fn an_interval_repeats_until_it_is_dropped() {
        let mut dom = VirtualDom::new(repeating);
        dom.rebuild_in_place();

        drive(&mut dom, Duration::from_secs(2), || fires() >= 3);
        let ticks = fires();
        assert!(ticks >= 3, "the interval stopped after {ticks}");

        drop_subscription();
        drive(&mut dom, Duration::from_millis(200), || false);
        assert_eq!(fires(), ticks, "a cancelled interval kept ticking");
    }
}
