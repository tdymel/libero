//! `use_system_notification` on a target without a Notifications API (a
//! native test build), and its bookkeeping of shown notifications fed in directly.

use std::cell::{Cell, RefCell};

use super::*;

thread_local! {
    static HANDLE: RefCell<Option<SystemNotifier>> = const { RefCell::new(None) };
    static CLOSED: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static DROPPED: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static CLICKS: Cell<u32> = const { Cell::new(0) };
    static ACTIONS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

struct Fake(u64);

impl ShownNotification for Fake {
    fn close(&self) {
        CLOSED.with_borrow_mut(|closed| closed.push(self.0));
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        DROPPED.with_borrow_mut(|dropped| dropped.push(self.0));
    }
}

fn app() -> Element {
    let shown = use_signal(|| true);
    provide_context(shown);
    rsx! {
        if shown() {
            Notifying {}
        }
    }
}

#[component]
fn Notifying() -> Element {
    let notifier = use_system_notification();
    HANDLE.with_borrow_mut(|handle| *handle = Some(notifier));
    rsx! {}
}

fn pump(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

fn mounted() -> VirtualDom {
    CLOSED.take();
    DROPPED.take();
    CLICKS.set(0);
    ACTIONS.take();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    pump(&mut dom);
    dom
}

fn handle() -> SystemNotifier {
    HANDLE.with_borrow(|handle| handle.expect("rendered"))
}

/// Pretends `show` succeeded for `id`, as it would on a target with an API.
fn fake_shown(dom: &VirtualDom, id: u64, tag: Option<&str>) {
    let mut notifier = handle();
    dom.in_scope(ScopeId::APP, || {
        let on_click = Callback::new(|()| CLICKS.set(CLICKS.get() + 1));
        let on_action =
            Callback::new(|action: String| ACTIONS.with_borrow_mut(|actions| actions.push(action)));
        notifier.shown.write().push(Entry {
            id,
            tag: tag.map(str::to_string),
            on_click: Some(on_click),
            on_action: Some(on_action),
            shown: Box::new(Fake(id)),
        });
    });
}

#[test]
fn without_a_notifications_api_it_reports_unsupported() {
    let mut dom = mounted();
    let mut notifier = handle();
    dom.in_runtime(|| {
        assert!(!notifier.is_supported());
        assert_eq!(notifier.permission(), PermissionState::Unsupported);
        notifier.request();
        assert_eq!(notifier.error(), Some(SystemNotificationError::Unsupported));
        notifier.show(SystemNotification::new("Build done"));
    });
    pump(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(notifier.error(), Some(SystemNotificationError::Unsupported));
        assert!(!notifier.is_pending());
    });
}

#[test]
fn a_refused_show_keeps_the_permission() {
    let dom = mounted();
    let mut notifier = handle();
    dom.in_runtime(|| {
        notifier.permission.set(PermissionState::Prompt);
        notifier.fail(SystemNotificationError::Denied);
        assert_eq!(notifier.permission(), PermissionState::Prompt);
        assert_eq!(notifier.error(), Some(SystemNotificationError::Denied));
    });
}

#[test]
fn close_closes_only_the_tag() {
    let dom = mounted();
    fake_shown(&dom, 1, Some("build"));
    fake_shown(&dom, 2, Some("chat"));
    fake_shown(&dom, 3, None);
    let mut notifier = handle();
    dom.in_runtime(|| notifier.close("build"));
    assert_eq!(CLOSED.take(), [1]);
    assert_eq!(DROPPED.take(), [1]);
    dom.in_runtime(|| assert_eq!(notifier.shown.peek().len(), 2));
}

#[test]
fn a_click_runs_on_click_and_a_close_forgets_it() {
    let mut dom = mounted();
    fake_shown(&dom, 1, None);
    let mut notifier = handle();
    dom.in_runtime(|| notifier.events.write().push((1, NotificationEvent::Click)));
    pump(&mut dom);
    assert_eq!(CLICKS.get(), 1);
    assert!(DROPPED.with_borrow(Vec::is_empty));

    dom.in_runtime(|| notifier.events.write().push((1, NotificationEvent::Close)));
    pump(&mut dom);
    assert_eq!(DROPPED.take(), [1]);
    assert!(
        CLOSED.with_borrow(Vec::is_empty),
        "the user closed it already"
    );
}

#[test]
fn an_action_runs_on_action_with_its_id_and_not_on_click() {
    let mut dom = mounted();
    fake_shown(&dom, 1, None);
    fake_shown(&dom, 2, None);
    let mut notifier = handle();
    dom.in_runtime(|| {
        let event = NotificationEvent::from_name("action:reply").expect("an action name");
        notifier.events.write().push((2, event));
    });
    pump(&mut dom);
    assert_eq!(ACTIONS.take(), ["reply"]);
    assert_eq!(CLICKS.get(), 0);
    assert!(DROPPED.with_borrow(Vec::is_empty));
}

#[test]
fn unmount_stops_events_but_leaves_notifications_shown() {
    let mut dom = mounted();
    fake_shown(&dom, 1, Some("build"));
    let mut shown = dom.in_scope(ScopeId::APP, consume_context::<Signal<bool>>);
    dom.in_runtime(|| shown.set(false));
    pump(&mut dom);
    assert_eq!(DROPPED.take(), [1]);
    assert!(CLOSED.with_borrow(Vec::is_empty));
}
