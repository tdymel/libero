//! Saving bytes the app made (an export, a report) as a file the user keeps.

use std::{future::Future, pin::Pin};

/// What became of a [`save_file`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveOutcome {
    /// Written to a file, or handed to the browser's download.
    Saved,
    /// Handed to the system share sheet (Android); where it went is the user's choice.
    Shared,
    /// The user closed the save dialog.
    Cancelled,
    /// Nothing was saved, with the reason.
    Failed(String),
}

/// The reason where a target has no save path.
pub(crate) const UNSUPPORTED: &str = "not supported on this platform";

pub(crate) type Saving = Pin<Box<dyn Future<Output = SaveOutcome>>>;

/// One way to save per target. Not element-scoped, like
/// [`ClipboardApi`](super::clipboard::ClipboardApi).
pub(crate) trait SaveFileApi {
    fn save(&self, name: &str, mime: &str, bytes: Vec<u8>) -> Saving;
}

/// Saves `bytes` as a file named `name`: a download on the web, a save dialog
/// on the desktop, the share sheet on Android (text only).
///
/// Docs: <https://libero-ui.dev/hooks/save-file>
/// ```rust,no_run
/// use libero::platform::{SaveOutcome, save_file};
///
/// async fn export(csv: String) {
///     let outcome = save_file("rows.csv", "text/csv", csv.into_bytes()).await;
///     if let SaveOutcome::Failed(reason) = outcome {
///         eprintln!("export failed: {reason}");
///     }
/// }
/// ```
pub async fn save_file(name: &str, mime: &str, bytes: Vec<u8>) -> SaveOutcome {
    match save_file_api() {
        Some(api) => api.save(name, mime, bytes).await,
        None => SaveOutcome::Failed(UNSUPPORTED.to_string()),
    }
}

/// A Blob behind a clicked `<a download>`: the browser saves without asking back.
#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::JsCast;

    use super::{SaveFileApi, SaveOutcome, Saving};

    pub(super) struct WebSaveFile;

    impl SaveFileApi for WebSaveFile {
        fn save(&self, name: &str, mime: &str, bytes: Vec<u8>) -> Saving {
            let outcome = download(name, mime, &bytes)
                .map(|()| SaveOutcome::Saved)
                .unwrap_or_else(|error| SaveOutcome::Failed(format!("{error:?}")));
            Box::pin(std::future::ready(outcome))
        }
    }

    fn download(name: &str, mime: &str, bytes: &[u8]) -> Result<(), wasm_bindgen::JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(bytes));
        let options = web_sys::BlobPropertyBag::new();
        options.set_type(mime);
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)?;
        let link: web_sys::HtmlAnchorElement = document.create_element("a")?.unchecked_into();
        link.set_href(&url);
        link.set_download(name);
        document.body().ok_or("no body")?.append_child(&link)?;
        link.click();
        link.remove();
        // Revoked later: Firefox reads the URL after `click` returns.
        let revoke = wasm_bindgen::closure::Closure::once_into_js(move || {
            let _ = web_sys::Url::revoke_object_url(&url);
        });
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            revoke.unchecked_ref(),
            40_000,
        )?;
        Ok(())
    }

    pub(super) static SAVE_FILE: WebSaveFile = WebSaveFile;
}

/// The system's save dialog through `rfd`: Blitz, and the desktop WebView's window.
#[cfg(all(
    not(target_arch = "wasm32"),
    any(
        feature = "native",
        all(feature = "desktop", not(target_os = "android"))
    )
))]
mod dialog {
    use super::{SaveFileApi, Saving, written};

    pub(super) struct DialogSaveFile;

    impl SaveFileApi for DialogSaveFile {
        fn save(&self, name: &str, _mime: &str, bytes: Vec<u8>) -> Saving {
            let mut dialog = rfd::AsyncFileDialog::new().set_file_name(name);
            if let Some((_, extension)) = name.rsplit_once('.') {
                dialog = dialog.add_filter(extension, &[extension]);
            }
            Box::pin(async move {
                let picked = dialog.save_file().await;
                written(picked.map(|handle| handle.path().to_path_buf()), &bytes)
            })
        }
    }

    pub(super) static SAVE_FILE: DialogSaveFile = DialogSaveFile;
}

#[cfg(all(target_os = "android", not(feature = "native")))]
#[path = "save_file_android.rs"]
mod android;

/// The outcome of writing `bytes` to the path a dialog picked, `None` when cancelled.
#[cfg(any(
    test,
    all(
        not(target_arch = "wasm32"),
        any(
            feature = "native",
            all(feature = "desktop", not(target_os = "android"))
        )
    )
))]
fn written(path: Option<std::path::PathBuf>, bytes: &[u8]) -> SaveOutcome {
    let Some(path) = path else {
        return SaveOutcome::Cancelled;
    };
    match std::fs::write(&path, bytes) {
        Ok(()) => SaveOutcome::Saved,
        Err(error) => SaveOutcome::Failed(format!("{}: {error}", path.display())),
    }
}

#[cfg(test)]
thread_local! {
    static FAKE: std::cell::Cell<Option<&'static dyn SaveFileApi>> = const { std::cell::Cell::new(None) };
}

/// Puts back the platform's save path when dropped.
#[cfg(test)]
pub(crate) struct FakeSaveFileGuard;

#[cfg(test)]
impl Drop for FakeSaveFileGuard {
    fn drop(&mut self) {
        FAKE.with(|fake| fake.set(None));
    }
}

/// Answers `save_file` with `fake` on this thread until the guard drops.
#[cfg(test)]
pub(crate) fn fake_save_file(fake: &'static dyn SaveFileApi) -> FakeSaveFileGuard {
    FAKE.with(|slot| slot.set(Some(fake)));
    FakeSaveFileGuard
}

/// `None` where the target has no save path: a server render, a page without scripts.
fn save_file_api() -> Option<&'static dyn SaveFileApi> {
    #[cfg(test)]
    if let Some(fake) = FAKE.with(std::cell::Cell::get) {
        return Some(fake);
    }
    #[cfg(target_arch = "wasm32")]
    return Some(&web::SAVE_FILE);
    #[cfg(all(
        not(target_arch = "wasm32"),
        any(
            feature = "native",
            all(feature = "desktop", not(target_os = "android"))
        )
    ))]
    return Some(&dialog::SAVE_FILE);
    #[cfg(all(target_os = "android", not(feature = "native")))]
    return Some(&android::SAVE_FILE);
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        not(feature = "native"),
        not(feature = "desktop")
    ))]
    return super::backend::webview_save_file();
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        let waker = std::task::Waker::noop();
        match future
            .as_mut()
            .poll(&mut std::task::Context::from_waker(waker))
        {
            std::task::Poll::Ready(value) => value,
            std::task::Poll::Pending => panic!("the fake answers at once"),
        }
    }

    struct Recording;

    thread_local! {
        static SAVED: RefCell<Vec<(String, String, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
    }

    impl SaveFileApi for Recording {
        fn save(&self, name: &str, mime: &str, bytes: Vec<u8>) -> Saving {
            SAVED.with_borrow_mut(|saved| saved.push((name.into(), mime.into(), bytes)));
            Box::pin(std::future::ready(SaveOutcome::Shared))
        }
    }

    static RECORDING: Recording = Recording;

    #[test]
    fn save_file_hands_name_type_and_bytes_to_the_platform_and_returns_its_outcome() {
        let _fake = fake_save_file(&RECORDING);
        let outcome = block_on(save_file("rows.csv", "text/csv", b"a,b\n1,2\n".to_vec()));
        assert_eq!(outcome, SaveOutcome::Shared);
        let saved = SAVED.with_borrow(Clone::clone);
        assert_eq!(
            saved,
            [("rows.csv".into(), "text/csv".into(), b"a,b\n1,2\n".to_vec())]
        );
    }

    #[cfg(not(any(feature = "native", feature = "desktop")))]
    #[test]
    fn save_file_fails_without_a_page_to_save_from() {
        let outcome = block_on(save_file("rows.csv", "text/csv", Vec::new()));
        assert_eq!(outcome, SaveOutcome::Failed(UNSUPPORTED.to_string()));
    }

    #[test]
    fn a_cancelled_dialog_saves_nothing() {
        assert_eq!(written(None, b"x"), SaveOutcome::Cancelled);
    }

    #[test]
    fn a_picked_path_gets_the_bytes() {
        let path = std::env::temp_dir().join(format!("libero-save-{}.csv", std::process::id()));
        assert_eq!(written(Some(path.clone()), b"a,b\n"), SaveOutcome::Saved);
        assert_eq!(std::fs::read(&path).unwrap(), b"a,b\n");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn an_unwritable_path_fails_with_the_reason() {
        let path = std::env::temp_dir().join("libero-no-such-dir/rows.csv");
        let SaveOutcome::Failed(reason) = written(Some(path), b"x") else {
            panic!("a missing directory cannot be written");
        };
        assert!(reason.contains("libero-no-such-dir"), "{reason}");
    }
}
