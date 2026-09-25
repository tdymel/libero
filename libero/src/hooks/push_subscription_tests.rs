//! `use_push_subscription` on a target without web push (a native test build),
//! and its answers to subscriptions and errors fed in directly.

use std::cell::RefCell;

use super::*;

thread_local! {
    static HANDLE: RefCell<Option<PushSubscription>> = const { RefCell::new(None) };
}

fn app() -> Element {
    let push = use_push_subscription(PushOptions {
        service_worker: "/sw.js".into(),
        vapid_public_key: "key".into(),
    });
    HANDLE.with_borrow_mut(|handle| *handle = Some(push));
    rsx! {}
}

fn mounted() -> VirtualDom {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    dom
}

fn handle() -> PushSubscription {
    HANDLE.with_borrow(|handle| handle.expect("rendered"))
}

fn endpoint() -> PushEndpoint {
    PushEndpoint {
        endpoint: "https://push.example/abc".into(),
        p256dh: "p256dh".into(),
        auth: "auth".into(),
    }
}

#[test]
fn without_web_push_it_reports_unsupported() {
    let dom = mounted();
    let mut push = handle();
    dom.in_runtime(|| {
        assert!(!push.is_supported());
        assert_eq!(push.permission(), PermissionState::Unsupported);
        push.subscribe();
        assert_eq!(push.error(), Some(PushError::Unsupported));
        assert!(!push.is_pending());
        push.unsubscribe();
        assert_eq!(push.subscription(), None);
    });
}

#[test]
fn a_subscription_grants_and_an_unsubscribe_clears_it() {
    let dom = mounted();
    let mut push = handle();
    dom.in_runtime(|| {
        push.settle(Err(PushError::Failed));
        push.settle(Ok(Some(endpoint())));
        assert_eq!(push.subscription(), Some(endpoint()));
        assert_eq!(push.error(), None);
        assert_eq!(push.permission(), PermissionState::Granted);
        push.settle(Ok(None));
        assert_eq!(push.subscription(), None);
    });
}

#[test]
fn a_denial_keeps_the_subscription_state() {
    let dom = mounted();
    let mut push = handle();
    dom.in_runtime(|| {
        push.settle(Err(PushError::Denied));
        assert_eq!(push.permission(), PermissionState::Denied);
        assert_eq!(push.error(), Some(PushError::Denied));
        assert_eq!(push.subscription(), None);
    });
}
