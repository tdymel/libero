use std::time::Duration;

use super::backend;

/// A live timer. **Dropping it cancels** - that is the whole contract, which is
/// why the trait has no methods.
///
/// So a cancelled timer is never a special case: a notification that is
/// dismissed early drops its subscription, a carousel that unmounts drops its
/// autoplay, and neither needs a generation guard to ignore a callback that
/// should no longer run.
pub trait TimerSubscription {}

/// Being called back later. The second callback-shaped capability here, after
/// [`ScrollApi`](super::ScrollApi), and deliberately the same shape.
///
/// **The callback runs outside every scope**, exactly like a scroll callback,
/// so anything it writes has to be a signal that outlives the moment - see the
/// `Signal::new_in_scope(.., ScopeId::ROOT)` pattern `use_popover` uses for the
/// same reason.
pub trait TimerApi {
    /// Calls `callback` once, `delay` from now, unless the returned
    /// subscription is dropped first.
    fn after(&self, delay: Duration, callback: Box<dyn FnOnce()>) -> Box<dyn TimerSubscription>;

    /// Calls `callback` every `interval` until the returned subscription is
    /// dropped. Carousel autoplay is why this exists and not just [`after`](Self::after):
    /// re-arming a one-shot from inside its own callback means the callback has
    /// to own the subscription it is running under.
    fn every(&self, interval: Duration, callback: Box<dyn Fn()>) -> Box<dyn TimerSubscription>;
}

/// `Some` on every renderer: the browser's own `setTimeout` on the web, and a
/// sleeping thread delivering through a dioxus task everywhere else. `None`
/// only where there is no dioxus runtime to deliver into, which is a server
/// render.
pub fn timer() -> Option<Box<dyn TimerApi>> {
    backend::timer()
}

/// Against the real non-wasm arm - a real thread, a real sleep, and a real
/// `VirtualDom` polling the task the callback is delivered on. Each test runs
/// on its own thread, which is what makes the `thread_local` state below
/// isolated rather than shared.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::thread;
    use std::time::Instant;

    use dioxus::prelude::*;

    use super::*;

    thread_local! {
        static FIRED: Cell<usize> = const { Cell::new(0) };
        /// The subscription the test drops to cancel. Handing it out of the
        /// component is the whole point: the contract is what happens when it
        /// dies, so the test has to own the moment it does.
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

    /// Polls the dom until `done`, or until `limit` runs out. `process_events`
    /// is what drains a woken task, so this is the renderer's job stood in for.
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
