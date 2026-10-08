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

/// No render between the writes: each reads what the one before wrote.
#[test]
fn writes_in_one_handler_build_on_each_other() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let dom = mounted();
    dom.in_runtime(|| {
        handle(0).update(|count| *count += 1);
        handle(1).update(|count| *count += 1);
        assert_eq!(handle(0).get(), 2);
        handle(0).set(5);
        assert_eq!((handle(0).get(), handle(1).get()), (5, 5));
        handle(1).update(|count| *count += 1);
    });
    assert_eq!(stored_text(local), Some("6".into()));
}

thread_local! {
    static LEVEL: std::cell::Cell<Option<Stored<f64>>> = const { std::cell::Cell::new(None) };
}

fn level_app() -> Element {
    LEVEL.set(Some(use_local_storage("level", || 0.5_f64)));
    rsx! {}
}

#[test]
fn a_value_that_does_not_read_back_is_not_kept() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(level_app);
    dom.rebuild_in_place();
    dom.in_runtime(|| {
        let mut level = LEVEL.get().expect("rendered");
        level.set(0.25);
        level.set(f64::NAN);
        assert_eq!(
            (level.get(), level.error()),
            (0.25, Some(StorageError::Invalid))
        );
    });
    assert_eq!(
        local.values.borrow().get("level").cloned(),
        Some("0.25".into())
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
    // `update` starts from the default and overwrites the text, as documented.
    dom.in_runtime(|| handle(1).update(|count| *count += 1));
    assert_eq!(stored_text(local), Some("1".into()));
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

#[test]
fn a_store_read_at_the_first_render_is_loaded_at_once() {
    let (local, session) = stores();
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.in_runtime(|| assert!(handle(0).is_loaded() && handle(2).is_loaded()));
}

thread_local! {
    static LATE: std::cell::Cell<Option<Stored<u32>>> = const { std::cell::Cell::new(None) };
}

fn late_app() -> Element {
    let late = use_local_storage_with(
        "count",
        || 0_u32,
        StorageOptions {
            read_after_mount: true,
            ..Default::default()
        },
    );
    LATE.set(Some(late));
    let (count, loaded, error) = (late.get(), late.is_loaded(), late.error());
    rsx! { span { "late={count} loaded={loaded} error={error:?}" } }
}

/// The first render is what a server render shows: the default, no error.
#[test]
fn a_read_after_mount_shows_the_default_first() {
    let (local, session) = stores();
    local.values.borrow_mut().insert("count".into(), "5".into());
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(late_app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("late=0 loaded=false error=None"), "{html}");
    pump(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("late=5 loaded=true error=None"), "{html}");
}

/// The server's missing store reports nothing until the client has mounted.
#[test]
fn a_read_after_mount_defers_the_error() {
    let _fake = fake_storage(None, None);
    let mut dom = VirtualDom::new(late_app);
    dom.rebuild_in_place();
    assert!(dioxus_ssr::render(&dom).contains("error=None"));
    pump(&mut dom);
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains("error=Some(Unavailable)"), "{html}");
}

/// A write before the read lands starts from the stored value, not the default.
#[test]
fn an_update_before_the_read_after_mount_reads_first() {
    let (local, session) = stores();
    local.values.borrow_mut().insert("count".into(), "5".into());
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(late_app);
    dom.rebuild_in_place();
    dom.in_runtime(|| {
        let mut late = LATE.get().expect("rendered");
        late.update(|count| *count += 1);
        assert_eq!((late.get(), late.is_loaded()), (6, true));
    });
    pump(&mut dom);
    assert_eq!(stored_text(local), Some("6".into()));
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Scheme {
    Light,
    Dark,
}

thread_local! {
    static SCHEME: std::cell::Cell<Option<Stored<Scheme>>> = const { std::cell::Cell::new(None) };
    static BARE: std::cell::Cell<Option<Stored<u32>>> = const { std::cell::Cell::new(None) };
}

fn text_options() -> StorageOptions {
    StorageOptions {
        format: StorageFormat::Text,
        ..Default::default()
    }
}

fn scheme_app() -> Element {
    SCHEME.set(Some(use_local_storage_with(
        "lsx-color-scheme",
        || Scheme::Light,
        text_options(),
    )));
    BARE.set(Some(use_local_storage_with(
        "bare",
        || 0_u32,
        text_options(),
    )));
    rsx! {}
}

/// Bare text, as libero keeps `lsx-color-scheme`; a number has no bare text form.
#[test]
fn text_format_reads_and_writes_bare_text() {
    let (local, session) = stores();
    (local.values.borrow_mut()).insert("lsx-color-scheme".into(), "dark".into());
    let _fake = fake(local, session);
    let mut dom = VirtualDom::new(scheme_app);
    dom.rebuild_in_place();
    dom.in_runtime(|| {
        let mut scheme = SCHEME.get().expect("rendered");
        assert_eq!((scheme.get(), scheme.error()), (Scheme::Dark, None));
        scheme.set(Scheme::Light);
        let mut bare = BARE.get().expect("rendered");
        bare.set(3);
        assert_eq!((bare.get(), bare.error()), (0, Some(StorageError::Invalid)));
    });
    let values = local.values.borrow();
    assert_eq!(
        values.get("lsx-color-scheme").map(String::as_str),
        Some("light")
    );
    assert_eq!(values.get("bare"), None);
}
