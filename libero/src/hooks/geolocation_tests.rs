//! `use_geolocation`'s state on a target without a Geolocation API (a native
//! test build), and its answers to fixes and errors fed in directly.

use std::cell::{Cell, RefCell};

use super::*;

thread_local! {
    static HANDLE: RefCell<Option<Geolocation>> = const { RefCell::new(None) };
    static DROPPED: Cell<bool> = const { Cell::new(false) };
}

struct Fake;

impl GeolocationSubscription for Fake {}

impl Drop for Fake {
    fn drop(&mut self) {
        DROPPED.set(true);
    }
}

const FIX: Position = Position {
    latitude: 52.52,
    longitude: 13.405,
    accuracy: 20.0,
    altitude: None,
    altitude_accuracy: None,
    heading: None,
    speed: None,
    timestamp_ms: 0.0,
};

fn app() -> Element {
    let shown = use_signal(|| true);
    provide_context(shown);
    rsx! {
        if shown() {
            Located {}
        }
    }
}

#[component]
fn Located() -> Element {
    let location = use_geolocation(GeolocationOptions::default());
    HANDLE.with_borrow_mut(|handle| *handle = Some(location));
    rsx! {}
}

fn pump(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

fn mounted() -> VirtualDom {
    DROPPED.set(false);
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    pump(&mut dom);
    dom
}

fn handle() -> Geolocation {
    HANDLE.with_borrow(|handle| handle.expect("rendered"))
}

/// Pretends a watch runs, as `watch` would leave it on a target with one.
fn start_fake_watch(dom: &VirtualDom) -> Geolocation {
    let mut location = handle();
    dom.in_runtime(|| {
        location.watch.set(Some(Box::new(Fake)));
        location.watching.set(true);
    });
    location
}

#[test]
fn without_a_geolocation_api_it_reports_unsupported() {
    let mut dom = mounted();
    let mut location = handle();
    dom.in_runtime(|| {
        assert!(!location.is_supported());
        assert_eq!(location.permission(), PermissionState::Unsupported);
        location.request();
        location.watch();
    });
    pump(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(location.error(), Some(GeolocationError::Unsupported));
        assert!(!location.is_pending());
        assert!(!location.is_watching());
        assert_eq!(location.position(), None);
    });
}

#[test]
fn a_fix_clears_the_error_and_grants() {
    let dom = mounted();
    let mut location = handle();
    dom.in_runtime(|| {
        location.settle(Err(GeolocationError::Timeout));
        assert_eq!(location.error(), Some(GeolocationError::Timeout));
        location.settle(Ok(FIX));
        assert_eq!(location.position(), Some(FIX));
        assert_eq!(location.error(), None);
        assert_eq!(location.permission(), PermissionState::Granted);
    });
}

#[test]
fn a_denial_ends_the_watch_and_keeps_the_last_fix() {
    let mut dom = mounted();
    let mut location = start_fake_watch(&dom);
    dom.in_runtime(|| {
        location.settle(Ok(FIX));
        location.settle(Err(GeolocationError::Denied));
        assert!(!location.is_watching());
        assert_eq!(location.permission(), PermissionState::Denied);
        assert_eq!(location.position(), Some(FIX));
    });
    assert!(!DROPPED.get(), "dropped inside its own callback");
    pump(&mut dom);
    assert!(DROPPED.get(), "the effect drops the ended watch");
}

#[test]
fn a_timeout_keeps_the_watch() {
    let mut dom = mounted();
    let mut location = start_fake_watch(&dom);
    dom.in_runtime(|| location.settle(Err(GeolocationError::Timeout)));
    pump(&mut dom);
    dom.in_runtime(|| assert!(location.is_watching()));
    assert!(!DROPPED.get());
}

#[test]
fn stop_and_unmount_drop_the_watch() {
    let mut dom = mounted();
    let mut location = start_fake_watch(&dom);
    dom.in_runtime(|| location.stop());
    assert!(DROPPED.get());

    DROPPED.set(false);
    start_fake_watch(&dom);
    let mut shown = dom.in_scope(ScopeId::APP, consume_context::<Signal<bool>>);
    dom.in_runtime(|| shown.set(false));
    pump(&mut dom);
    assert!(DROPPED.get(), "unmount clears the watch");
}
