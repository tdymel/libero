//! `use_indexed_db` over an in-memory async store that can hold its answers back, so a
//! test sees a load still on its way.

use std::cell::RefCell;

use super::*;
use crate::hooks::polling_tests::{pump_until, started};
use crate::platform::{IndexedDbApi, MemoryIndexedDb, fake_indexed_db};

thread_local! {
    static HANDLES: RefCell<[Option<StoredAsync<u32>>; 2]> = const { RefCell::new([None; 2]) };
}

fn app() -> Element {
    rsx! {
        Count { slot: 0 }
        Count { slot: 1 }
    }
}

#[component]
fn Count(slot: usize) -> Element {
    let stored = use_indexed_db("count", || 0_u32);
    HANDLES.with_borrow_mut(|handles| handles[slot] = Some(stored));
    rsx! { span { "count={stored.get()}" } }
}

fn handle(slot: usize) -> StoredAsync<u32> {
    HANDLES.with_borrow(|handles| handles[slot].expect("rendered"))
}

fn fake(db: &'static MemoryIndexedDb) -> impl Drop {
    fake_indexed_db(Some(db as &dyn IndexedDbApi))
}

fn stored_text(db: &MemoryIndexedDb) -> Option<String> {
    db.values.borrow().get("count").cloned()
}

fn seeded(text: &str) -> &'static MemoryIndexedDb {
    let db = MemoryIndexedDb::leaked();
    db.values.borrow_mut().insert("count".into(), text.into());
    db
}

fn wait_for_load(dom: &mut VirtualDom) {
    pump_until(dom, "the load", |dom| {
        dom.in_runtime(|| handle(0).is_loaded())
    });
}

#[test]
fn the_default_shows_until_the_load_lands() {
    let db = seeded("5");
    db.hold();
    let _fake = fake(db);
    let mut dom = started(app);
    dom.in_runtime(|| {
        assert_eq!((handle(0).get(), handle(0).is_loaded()), (0, false));
        assert!(!handle(0).is_stored());
    });
    db.release();
    wait_for_load(&mut dom);
    dom.in_runtime(|| {
        assert_eq!((handle(0).get(), handle(1).get()), (5, 5));
        assert!(handle(0).is_stored());
        assert_eq!(handle(0).error(), None);
    });
}

#[test]
fn a_missing_key_loads_as_the_default() {
    let db = MemoryIndexedDb::leaked();
    let _fake = fake(db);
    let mut dom = started(app);
    wait_for_load(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(handle(0).get(), 0);
        assert!(!handle(0).is_stored());
    });
}

#[test]
fn a_set_before_the_load_lands_wins() {
    let db = seeded("1");
    db.hold();
    let _fake = fake(db);
    let mut dom = started(app);
    dom.in_runtime(|| handle(0).set(5));
    db.release();
    wait_for_load(&mut dom);
    pump_until(&mut dom, "the save", |_| {
        stored_text(db).as_deref() == Some("5")
    });
    dom.in_runtime(|| assert_eq!((handle(0).get(), handle(1).get()), (5, 5)));
}

#[test]
fn two_handles_share_the_key_and_the_value_shows_before_the_save() {
    let db = MemoryIndexedDb::leaked();
    let _fake = fake(db);
    let mut dom = started(app);
    wait_for_load(&mut dom);
    db.hold();
    dom.in_runtime(|| {
        handle(0).set(3);
        assert_eq!(handle(1).get(), 3);
        handle(1).update(|count| *count += 1);
        assert_eq!(handle(0).get(), 4);
    });
    assert_eq!(stored_text(db), None);
    db.release();
    pump_until(&mut dom, "the saves", |_| {
        stored_text(db).as_deref() == Some("4")
    });
}

#[test]
fn a_failed_save_keeps_the_value_and_reports_why() {
    let db = MemoryIndexedDb::leaked();
    let _fake = fake(db);
    let mut dom = started(app);
    wait_for_load(&mut dom);
    db.refuse.set(Some(StorageError::Full));
    dom.in_runtime(|| handle(0).set(7));
    pump_until(&mut dom, "the failure", |dom| {
        dom.in_runtime(|| handle(1).error()) == Some(StorageError::Full)
    });
    dom.in_runtime(|| assert_eq!(handle(1).get(), 7));
    assert_eq!(stored_text(db), None);
    db.refuse.set(None);
    dom.in_runtime(|| handle(0).set(8));
    pump_until(&mut dom, "the next save", |dom| {
        dom.in_runtime(|| handle(1).error()).is_none()
    });
    assert_eq!(stored_text(db).as_deref(), Some("8"));
}

#[test]
fn a_failed_load_reports_why_and_is_done() {
    let db = seeded("5");
    db.refuse.set(Some(StorageError::Unavailable));
    let _fake = fake(db);
    let mut dom = started(app);
    wait_for_load(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(handle(0).get(), 0);
        assert_eq!(handle(0).error(), Some(StorageError::Unavailable));
    });
}

#[test]
fn remove_falls_back_to_the_default_and_drops_the_stored_value() {
    let db = seeded("5");
    let _fake = fake(db);
    let mut dom = started(app);
    wait_for_load(&mut dom);
    dom.in_runtime(|| {
        assert!(handle(0).is_stored());
        handle(0).remove();
        assert_eq!((handle(1).get(), handle(1).is_stored()), (0, false));
    });
    pump_until(&mut dom, "the removal", |_| stored_text(db).is_none());
    dom.in_runtime(|| assert_eq!(handle(0).error(), None));
}

#[test]
fn a_change_from_another_tab_arrives_and_beats_a_load_on_its_way() {
    let db = seeded("1");
    db.hold();
    let _fake = fake(db);
    let mut dom = started(app);
    db.change_elsewhere("count", Some("9"));
    db.release();
    wait_for_load(&mut dom);
    dom.in_runtime(|| assert_eq!((handle(0).get(), handle(1).get()), (9, 9)));
    db.change_elsewhere("count", Some("6"));
    crate::hooks::polling_tests::flush(&mut dom);
    assert_eq!(dioxus_ssr::render(&dom).matches("count=6").count(), 2);
    db.change_elsewhere("count", None);
    crate::hooks::polling_tests::flush(&mut dom);
    dom.in_runtime(|| assert!(!handle(0).is_stored()));
    assert_eq!(dioxus_ssr::render(&dom).matches("count=0").count(), 2);
}

#[test]
fn without_a_store_values_live_in_memory_and_report_it() {
    let _fake = fake_indexed_db(None);
    let mut dom = started(app);
    wait_for_load(&mut dom);
    dom.in_runtime(|| {
        assert_eq!(handle(0).error(), Some(StorageError::Unavailable));
        handle(0).set(1);
    });
    dom.in_runtime(|| {
        assert_eq!(handle(1).get(), 1);
        assert_eq!(handle(1).error(), Some(StorageError::Unavailable));
    });
}
