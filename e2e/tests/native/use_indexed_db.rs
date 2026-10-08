//! `use_indexed_db` on Blitz keeps a file per key in `storage/indexed`: a new
//! document loads it.

use dioxus::prelude::*;
use e2e::native::{mount, with_storage_dir};
use libero::hooks::use_indexed_db;

fn app() -> Element {
    let mut stored = use_indexed_db("count", || 0_u32);
    rsx! {
        button { id: "add", onclick: move |_| stored.update(|count| *count += 1), "Add" }
        button { id: "remove", onclick: move |_| stored.remove(), "Remove" }
        p { id: "count", "{stored.get()}" }
        p { id: "loaded", "{stored.is_loaded()}" }
        p { id: "error", "{stored.error():?}" }
    }
}

/// The directory is process-wide, so the test holds it alone.
#[test]
fn a_value_outlives_the_document_in_a_file_of_its_own() {
    let dir = std::env::temp_dir().join(format!("libero-native-indexed-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    with_storage_dir(&dir, || {
        let mut page = mount(app);
        assert_eq!(page.text("#loaded"), "true");
        page.click("#add");
        page.click("#add");
        assert_eq!(page.text("#count"), "2");
        assert_eq!(page.text("#error"), "None");
        drop(page);

        let mut page = mount(app);
        assert_eq!(page.text("#loaded"), "true");
        assert_eq!(page.text("#count"), "2", "the file was not read back");
        let files: Vec<_> = std::fs::read_dir(dir.join("indexed"))
            .expect("the store's directory")
            .flatten()
            .map(|file| file.file_name())
            .collect();
        assert_eq!(files, ["count.json"]);
        assert!(
            !dir.join("count.json").exists(),
            "it wrote to local storage"
        );

        page.click("#remove");
        assert_eq!(page.text("#count"), "0");
        assert!(!dir.join("indexed").join("count.json").exists());
    });
    let _ = std::fs::remove_dir_all(&dir);
}
