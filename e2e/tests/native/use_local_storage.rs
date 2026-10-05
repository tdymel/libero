//! `use_local_storage` on Blitz keeps a file per key: a new document reads it,
//! while session storage starts over (todo 2209).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::{
    hooks::{use_local_storage, use_session_storage},
    platform::set_storage_dir,
};

fn app() -> Element {
    let mut local = use_local_storage("count", || 0_u32);
    let mut session = use_session_storage("count", || 0_u32);
    rsx! {
        button {
            id: "add",
            onclick: move |_| {
                local.update(|count| *count += 1);
                session.update(|count| *count += 1);
            },
            "Add"
        }
        p { id: "local", "{local.get()}" }
        p { id: "session", "{session.get()}" }
        p { id: "error", "{local.error():?}" }
    }
}

/// The only test here that stores: the directory is process-wide.
#[test]
fn a_local_value_outlives_the_document_and_a_session_one_does_not() {
    let dir = std::env::temp_dir().join(format!("libero-native-storage-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    set_storage_dir(dir.clone());

    let mut page = mount(app);
    page.click("#add");
    page.click("#add");
    assert_eq!(page.text("#local"), "2");
    assert_eq!(page.text("#session"), "2");
    assert_eq!(page.text("#error"), "None");
    drop(page);

    let page = mount(app);
    assert_eq!(page.text("#local"), "2", "the file was not read back");
    assert_eq!(
        page.text("#session"),
        "0",
        "session storage outlived its window"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
