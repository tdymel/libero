//! `use_indexed_db`: a counter, a second handle on its key and what `is_loaded`
//! showed. Each route has its own key: test pages share one browser profile.

use dioxus::prelude::*;
use libero::hooks::use_indexed_db;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-indexed-db", || rsx! { Kept { prefix: "e2e-idb" } }),
    (
        "/use-indexed-db/reload",
        || rsx! { Kept { prefix: "e2e-idb-reload" } },
    ),
    (
        "/use-indexed-db/tabs",
        || rsx! { Kept { prefix: "e2e-idb-tabs" } },
    ),
    (
        "/use-indexed-db/errors",
        || rsx! { Kept { prefix: "e2e-idb-errors" } },
    ),
    (
        "/use-indexed-db/blocked",
        || rsx! { Kept { prefix: "e2e-idb-blocked" } },
    ),
];

/// Off the web, the run's directory (`E2E_STORAGE_DIR`), never the user's data dir.
/// Android keeps the app's own files directory.
fn point_storage_dir() {
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    libero::platform::set_storage_dir(
        std::env::var_os("E2E_STORAGE_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("libero-e2e-storage")),
    );
}

#[component]
fn Kept(prefix: &'static str) -> Element {
    use_hook(point_storage_dir);
    let key = format!("{prefix}-count");
    let mut count = use_indexed_db(&key, || 0_u32);
    // One letter per value `is_loaded` showed: F, then T once the load landed.
    let mut shown = use_signal(String::new);
    use_effect(move || {
        let loaded = count.is_loaded();
        shown.write().push(if loaded { 'T' } else { 'F' });
    });

    rsx! {
        button { id: "add", onclick: move |_| count.update(|count| *count += 1), "Add" }
        button { id: "remove", onclick: move |_| count.remove(), "Remove" }
        p { id: "count", "{count.get()}" }
        p { id: "stored", "{count.is_stored()}" }
        p { id: "loaded", "{count.is_loaded()}" }
        p { id: "shown", "{shown}" }
        p { id: "error", "{count.error():?}" }
        Twin { storage_key: key }
    }
}

#[component]
fn Twin(storage_key: String) -> Element {
    let twin = use_indexed_db(&storage_key, || 0_u32);
    rsx! {
        p { id: "twin", "{twin.get()}" }
    }
}
