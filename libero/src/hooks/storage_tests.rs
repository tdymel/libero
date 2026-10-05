//! `use_local_storage` and `use_session_storage` over in-memory fake stores: a
//! new `VirtualDom` on the same fakes stands in for a reload.

use std::cell::RefCell;

use super::*;
use crate::platform::{MemoryStorage, StorageApi, fake_storage};

thread_local! {
    static HANDLES: RefCell<[Option<Stored<u32>>; 3]> = const { RefCell::new([None; 3]) };
}

fn app() -> Element {
    rsx! {
        Local { slot: 0 }
        Local { slot: 1 }
        Session { slot: 2 }
    }
}

#[component]
fn Local(slot: usize) -> Element {
    let stored = use_local_storage("count", || 0_u32);
    HANDLES.with_borrow_mut(|handles| handles[slot] = Some(stored));
    rsx! { span { "local={stored.get()}" } }
}

#[component]
fn Session(slot: usize) -> Element {
    let stored = use_session_storage("count", || 0_u32);
    HANDLES.with_borrow_mut(|handles| handles[slot] = Some(stored));
    rsx! { span { "session={stored.get()}" } }
}

fn pump(dom: &mut VirtualDom) {
    for _ in 0..3 {
        dom.process_events();
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
}

fn mounted() -> VirtualDom {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    pump(&mut dom);
    dom
}

fn handle(slot: usize) -> Stored<u32> {
    HANDLES.with_borrow(|handles| handles[slot].expect("rendered"))
}

fn stores() -> (&'static MemoryStorage, &'static MemoryStorage) {
    (MemoryStorage::leaked(), MemoryStorage::leaked())
}

fn fake(local: &'static MemoryStorage, session: &'static MemoryStorage) -> impl Drop {
    fake_storage(
        Some(local as &dyn StorageApi),
        Some(session as &dyn StorageApi),
    )
}

fn stored_text(store: &MemoryStorage) -> Option<String> {
    store.values.borrow().get("count").cloned()
}

#[test]
fn the_stored_value_shows_at_the_first_render() {
    let (local, session) = stores();
    local.values.borrow_mut().insert("count".into(), "5".into());
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("local=5"), "{html}");
    assert!(html.contains("session=0"), "{html}");
}

#[test]
fn handles_on_one_key_stay_in_sync_and_write_through() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let mut dom = mounted();
    dom.in_runtime(|| handle(0).set(3));
    pump(&mut dom);
    dom.in_runtime(|| assert_eq!(handle(1).get(), 3));
    assert_eq!(dioxus_ssr::render(&dom).matches("local=3").count(), 2);
    assert_eq!(stored_text(local), Some("3".into()));
    assert_eq!(stored_text(session), None);
}

#[test]
fn a_new_document_reads_what_was_set() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let dom = mounted();
    dom.in_runtime(|| {
        handle(0).update(|count| *count += 9);
        handle(2).set(4);
    });
    drop(dom);
    let dom = mounted();
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("local=9") && html.contains("session=4"),
        "{html}"
    );
}

#[test]
fn remove_falls_back_to_the_default() {
    let (local, session) = stores();
    local.values.borrow_mut().insert("count".into(), "5".into());
    let _fake = fake(local, session);
    let mut dom = mounted();
    dom.in_runtime(|| {
        assert!(handle(0).is_stored());
        handle(0).remove();
    });
    pump(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(handle(1).get(), 0);
        assert!(!handle(1).is_stored());
        assert_eq!(handle(1).error(), None);
    });
    assert_eq!(stored_text(local), None);
}

#[test]
fn text_that_does_not_parse_shows_the_default_and_stays_until_a_set() {
    let (local, session) = stores();
    local
        .values
        .borrow_mut()
        .insert("count".into(), "\"many\"".into());
    let _fake = fake(local, session);
    let mut dom = mounted();
    dom.in_runtime(|| {
        assert_eq!(handle(0).get(), 0);
        assert_eq!(handle(0).error(), Some(StorageError::Invalid));
    });
    assert_eq!(stored_text(local), Some("\"many\"".into()));
    dom.in_runtime(|| handle(0).set(2));
    pump(&mut dom);
    dom.in_runtime(|| assert_eq!((handle(0).get(), handle(0).error()), (2, None)));
    assert_eq!(stored_text(local), Some("2".into()));
}

#[test]
fn a_refused_write_still_holds_for_the_session() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let mut dom = mounted();
    local.refuse.set(Some(StorageError::Full));
    dom.in_runtime(|| handle(0).set(7));
    pump(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(handle(1).get(), 7);
        assert_eq!(handle(1).error(), Some(StorageError::Full));
    });
    assert_eq!(stored_text(local), None);
    local.refuse.set(None);
    dom.in_runtime(|| handle(0).set(8));
    pump(&mut dom);
    dom.in_runtime(|| assert_eq!(handle(1).error(), None));
    assert_eq!(stored_text(local), Some("8".into()));
}

#[test]
fn a_refused_read_shows_the_default_and_reports_it() {
    let (local, session) = stores();
    local.refuse.set(Some(StorageError::Unavailable));
    let _fake = fake(local, session);
    let dom = mounted();
    dom.in_runtime(|| {
        assert_eq!(handle(0).get(), 0);
        assert_eq!(handle(0).error(), Some(StorageError::Unavailable));
        assert_eq!(handle(2).error(), None);
    });
}

#[test]
fn without_a_store_values_live_in_memory_and_only_local_reports_it() {
    let _fake = fake_storage(None, None);
    let mut dom = mounted();
    dom.in_runtime(|| {
        assert_eq!(handle(0).error(), Some(StorageError::Unavailable));
        handle(0).set(1);
        handle(2).set(2);
    });
    pump(&mut dom);
    dom.in_runtime(|| {
        assert_eq!((handle(1).get(), handle(2).get()), (1, 2));
        assert_eq!(handle(1).error(), Some(StorageError::Unavailable));
        assert_eq!(handle(2).error(), None);
        assert!(handle(2).is_stored());
    });
}

#[test]
fn a_change_from_another_tab_arrives() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let mut dom = mounted();
    local.change_elsewhere(Some("count"), Some("6"));
    pump(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert_eq!(html.matches("local=6").count(), 2, "{html}");
    assert!(html.contains("session=0"), "{html}");
    local.change_elsewhere(None, None);
    pump(&mut dom);
    dom.in_runtime(|| assert!(!handle(0).is_stored()));
    assert!(dioxus_ssr::render(&dom).contains("local=0"));
}

thread_local! {
    static TEXT: std::cell::Cell<Option<SessionText>> = const { std::cell::Cell::new(None) };
}

fn text_app() -> Element {
    let text = session_text("stars");
    TEXT.set(Some(text));
    rsx! { span { "stars={text.get().unwrap_or_default()}" } }
}

fn set_text(dom: &mut VirtualDom, text: &str) {
    dom.in_runtime(|| TEXT.get().expect("rendered").set(text.into()));
    pump(dom);
}

/// Libero's own session caches: raw text, written through, shown where read.
#[test]
fn session_text_is_raw_and_rerenders_its_readers() {
    let (local, session) = stores();
    session
        .values
        .borrow_mut()
        .insert("stars".into(), "7".into());
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(text_app);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("stars=7"));
    set_text(&mut dom, "8");
    assert!(dioxus_ssr::render(&dom).contains("stars=8"));
    assert_eq!(
        session.values.borrow().get("stars").cloned(),
        Some("8".into())
    );
}

/// Off the web session text lives in its document, not in a thread-wide map.
#[test]
fn session_text_without_a_store_stays_in_its_document() {
    let _fake = fake_storage(None, None);
    let mut dom = VirtualDom::new(text_app);
    dom.rebuild_in_place();
    set_text(&mut dom, "8");
    assert!(dioxus_ssr::render(&dom).contains("stars=8"));
    let mut other = VirtualDom::new(text_app);
    other.rebuild_in_place();
    let html = dioxus_ssr::render(&other);
    assert!(!html.contains("stars=8"), "{html}");
}
