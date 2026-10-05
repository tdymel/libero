//! `use_local_storage` and `use_session_storage`: a counter in each area and a
//! second handle on the local key. Each route has its own keys: test pages share
//! one browser profile.

use dioxus::prelude::*;
use libero::hooks::{use_local_storage, use_session_storage};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/use-local-storage", || rsx! { Kept { prefix: "e2e-storage" } }),
    ("/use-local-storage/reload", || rsx! { Kept { prefix: "e2e-storage-reload" } }),
    ("/use-local-storage/tabs", || rsx! { Kept { prefix: "e2e-storage-tabs" } }),
    ("/use-local-storage/errors", || rsx! { Kept { prefix: "e2e-storage-errors" } }),
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
    let local_key = format!("{prefix}-local");
    let mut local = use_local_storage(&local_key, || 0_u32);
    let mut session = use_session_storage(&format!("{prefix}-session"), || 0_u32);

    rsx! {
        button { id: "add-local", onclick: move |_| local.update(|count| *count += 1), "Add local" }
        button { id: "remove-local", onclick: move |_| local.remove(), "Remove local" }
        button { id: "add-session", onclick: move |_| session.update(|count| *count += 1), "Add session" }
        button { id: "remove-session", onclick: move |_| session.remove(), "Remove session" }
        p { id: "local", "{local.get()}" }
        p { id: "stored", "{local.is_stored()}" }
        p { id: "error", "{local.error():?}" }
        p { id: "session", "{session.get()}" }
        Twin { storage_key: local_key }
    }
}

#[component]
fn Twin(storage_key: String) -> Element {
    let twin = use_local_storage(&storage_key, || 0_u32);
    rsx! {
        p { id: "twin", "{twin.get()}" }
    }
}
