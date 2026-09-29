use std::{cell::Cell, future::Future, pin::Pin};

use dioxus::html::FileData;

/// The files picked, none when the dialog was cancelled.
pub(crate) type Picked = Pin<Box<dyn Future<Output = Vec<FileData>>>>;

/// The system's file dialog, where no `input[type=file]` opens one. Not
/// element-scoped, like [`ClipboardApi`](super::clipboard::ClipboardApi).
pub(crate) trait FileDialogApi {
    /// `accept` and `capture` are the attributes' syntax; `accept` narrows
    /// what the dialog shows.
    fn open(&self, accept: &str, multiple: bool, capture: Option<&str>) -> Picked;
}

/// Blitz opens no picker for a file input, so `rfd` does (`native` only).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod native {
    use super::{FileDialogApi, Picked, extensions, picked_file};

    pub(super) struct NativeFileDialog;

    impl FileDialogApi for NativeFileDialog {
        fn open(&self, accept: &str, multiple: bool, _capture: Option<&str>) -> Picked {
            let mut dialog = rfd::AsyncFileDialog::new();
            if let Some(extensions) = extensions(accept) {
                dialog = dialog.add_filter(accept, &extensions);
            }
            Box::pin(async move {
                let handles = match multiple {
                    true => dialog.pick_files().await.unwrap_or_default(),
                    false => dialog.pick_file().await.into_iter().collect(),
                };
                handles
                    .into_iter()
                    .map(|handle| picked_file(handle.path().to_path_buf()))
                    .collect()
            })
        }
    }

    pub(super) static FILE_DIALOG: NativeFileDialog = NativeFileDialog;
}

/// Opens a file picker: `input`'s own where a file input opens one, else the
/// system dialog, whose pick reaches `take` (none when cancelled).
pub(crate) fn pick_files(
    accept: &str,
    multiple: bool,
    capture: Option<&str>,
    input: impl FnOnce(),
    take: impl FnOnce(Vec<FileData>) + 'static,
) {
    match file_dialog() {
        // One dialog at a time: a WebView hears one press from the button and
        // again from the group around it (`nested_interactive` is `false` there).
        Some(_) if PICKING.get() => {}
        Some(dialog) => {
            let picked = dialog.open(accept, multiple, capture);
            let picking = Picking::start();
            dioxus::prelude::spawn(async move {
                let files = picked.await;
                drop(picking);
                take(files);
            });
        }
        None => input(),
    }
}

thread_local! {
    static PICKING: Cell<bool> = const { Cell::new(false) };
}

/// Holds [`PICKING`] until dropped, also when the field unmounts mid-pick.
struct Picking;

impl Picking {
    fn start() -> Self {
        PICKING.set(true);
        Picking
    }
}

impl Drop for Picking {
    fn drop(&mut self) {
        PICKING.set(false);
    }
}

/// `None` where a file input opens the platform's own picker: the web. A
/// WebView's input cannot be clicked from Rust, so it gets one of its own.
fn file_dialog() -> Option<&'static dyn FileDialogApi> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return Some(&native::FILE_DIALOG);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_file_dialog();
    #[cfg(target_arch = "wasm32")]
    return None;
}

/// Extensions and the media type a picked file reports, for `accept`'s types.
#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
const TYPES: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("svg", "image/svg+xml"),
    ("bmp", "image/bmp"),
    ("ico", "image/vnd.microsoft.icon"),
    ("tif", "image/tiff"),
    ("tiff", "image/tiff"),
    ("heic", "image/heic"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("ogg", "audio/ogg"),
    ("flac", "audio/flac"),
    ("m4a", "audio/mp4"),
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("mov", "video/quicktime"),
    ("mkv", "video/x-matroska"),
    ("txt", "text/plain"),
    ("csv", "text/csv"),
    ("md", "text/markdown"),
    ("html", "text/html"),
    ("htm", "text/html"),
    ("css", "text/css"),
    ("pdf", "application/pdf"),
    ("json", "application/json"),
    ("xml", "application/xml"),
    ("zip", "application/zip"),
    ("doc", "application/msword"),
    (
        "docx",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ),
    ("xls", "application/vnd.ms-excel"),
    (
        "xlsx",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    ),
];

/// The media type of a file named `name`, by its extension.
#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
fn content_type(name: &str) -> Option<&'static str> {
    let (_, extension) = name.rsplit_once('.')?;
    TYPES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(extension))
        .map(|(_, content_type)| *content_type)
}

/// The dialog's filter for `accept`. `None` shows every file: an open
/// `accept`, or a type this table does not know, which a filter would hide.
#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
fn extensions(accept: &str) -> Option<Vec<String>> {
    let mut extensions = Vec::new();
    for entry in accept.split(',').map(str::trim).filter(|e| !e.is_empty()) {
        if let Some(extension) = entry.strip_prefix('.') {
            extensions.push(extension.to_ascii_lowercase());
            continue;
        }
        let entry = entry.to_ascii_lowercase();
        let known: Vec<_> = TYPES
            .iter()
            .filter(|(_, content_type)| match entry.strip_suffix("/*") {
                Some(group) if group != "*" => content_type.split('/').next() == Some(group),
                Some(_) => true,
                None => *content_type == entry,
            })
            .map(|(extension, _)| extension.to_string())
            .collect();
        if known.is_empty() || entry == "*/*" {
            return None;
        }
        extensions.extend(known);
    }
    (!extensions.is_empty()).then_some(extensions)
}

/// A file the dialog picked, read from the disk when asked.
#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
fn picked_file(path: std::path::PathBuf) -> FileData {
    FileData::new(PickedFile(path))
}

#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
struct PickedFile(std::path::PathBuf);

#[cfg(not(target_arch = "wasm32"))]
type Chunk = Result<dioxus::html::bytes::Bytes, dioxus::CapturedError>;

/// The whole file as one chunk.
#[cfg(not(target_arch = "wasm32"))]
struct Whole(Option<Chunk>);

#[cfg(not(target_arch = "wasm32"))]
impl futures_core::Stream for Whole {
    type Item = Chunk;

    fn poll_next(
        mut self: Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Chunk>> {
        std::task::Poll::Ready(self.0.take())
    }
}

/// A file in memory, a WebView's pick or a native crop: its path is only its name.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn held_file(
    name: String,
    content_type: String,
    last_modified: u64,
    bytes: Vec<u8>,
) -> FileData {
    FileData::new(HeldFile {
        name,
        content_type: (!content_type.is_empty()).then_some(content_type),
        last_modified,
        bytes: bytes.into(),
    })
}

#[cfg(not(target_arch = "wasm32"))]
struct HeldFile {
    name: String,
    content_type: Option<String>,
    last_modified: u64,
    bytes: dioxus::html::bytes::Bytes,
}

#[cfg(not(target_arch = "wasm32"))]
impl dioxus::html::NativeFileData for HeldFile {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn size(&self) -> u64 {
        self.bytes.len() as u64
    }

    fn last_modified(&self) -> u64 {
        self.last_modified
    }

    fn path(&self) -> std::path::PathBuf {
        self.name.clone().into()
    }

    fn content_type(&self) -> Option<String> {
        self.content_type.clone()
    }

    fn read_bytes(&self) -> Pin<Box<dyn Future<Output = Chunk>>> {
        Box::pin(std::future::ready(Ok(self.bytes.clone())))
    }

    fn byte_stream(&self) -> Pin<Box<dyn futures_core::Stream<Item = Chunk> + Send>> {
        Box::pin(Whole(Some(Ok(self.bytes.clone()))))
    }

    fn read_string(&self) -> Pin<Box<dyn Future<Output = Result<String, dioxus::CapturedError>>>> {
        let text = String::from_utf8(self.bytes.to_vec()).map_err(dioxus::CapturedError::from);
        Box::pin(std::future::ready(text))
    }

    fn inner(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
mod picked {
    use std::{
        future::{Future, ready},
        pin::Pin,
        time::UNIX_EPOCH,
    };

    use dioxus::{CapturedError, html::bytes::Bytes};

    use super::{Chunk, PickedFile, Whole, content_type};

    impl PickedFile {
        fn read(&self) -> Chunk {
            std::fs::read(&self.0)
                .map(Bytes::from)
                .map_err(CapturedError::from)
        }
    }

    impl dioxus::html::NativeFileData for PickedFile {
        fn name(&self) -> String {
            self.0
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default()
        }

        fn size(&self) -> u64 {
            std::fs::metadata(&self.0).map_or(0, |meta| meta.len())
        }

        /// Milliseconds since the epoch, as the web's `lastModified`.
        fn last_modified(&self) -> u64 {
            std::fs::metadata(&self.0)
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |since| since.as_millis() as u64)
        }

        fn path(&self) -> std::path::PathBuf {
            self.0.clone()
        }

        fn content_type(&self) -> Option<String> {
            content_type(&self.name()).map(str::to_string)
        }

        fn read_bytes(&self) -> Pin<Box<dyn Future<Output = Chunk>>> {
            Box::pin(ready(self.read()))
        }

        fn byte_stream(&self) -> Pin<Box<dyn futures_core::Stream<Item = Chunk> + Send>> {
            Box::pin(Whole(Some(self.read())))
        }

        fn read_string(&self) -> Pin<Box<dyn Future<Output = Result<String, CapturedError>>>> {
            Box::pin(ready(
                std::fs::read_to_string(&self.0).map_err(CapturedError::from),
            ))
        }

        fn inner(&self) -> &dyn std::any::Any {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter(accept: &str) -> Option<Vec<String>> {
        extensions(accept)
    }

    #[test]
    fn an_open_accept_filters_nothing() {
        assert_eq!(filter(""), None);
        assert_eq!(filter("*/*"), None);
        assert_eq!(filter(".pdf, */*"), None);
    }

    #[test]
    fn suffixes_and_types_become_extensions() {
        assert_eq!(filter(".PDF"), Some(vec!["pdf".into()]));
        assert_eq!(
            filter("image/jpeg, .txt"),
            Some(vec!["jpg".into(), "jpeg".into(), "txt".into()])
        );
        let images = filter("image/*").unwrap();
        assert!(images.contains(&"png".into()) && images.contains(&"svg".into()));
        assert!(!images.contains(&"pdf".into()));
    }

    /// A filter would hide the files of a type the table cannot name.
    #[test]
    fn an_unknown_type_filters_nothing() {
        assert_eq!(filter(".pdf, application/x-unknown"), None);
        assert_eq!(filter("model/*"), None);
    }

    #[test]
    fn a_picked_file_reads_off_the_disk() {
        let path = std::env::temp_dir().join(format!("libero-picked-{}.TXT", std::process::id()));
        std::fs::write(&path, "hello").unwrap();
        let file = picked_file(path.clone());
        assert_eq!(file.name(), path.file_name().unwrap().to_string_lossy());
        assert_eq!(file.size(), 5);
        assert_eq!(file.content_type().as_deref(), Some("text/plain"));
        assert!(file.last_modified() > 0);
        let text = ready_now(file.read_string());
        std::fs::remove_file(&path).unwrap();
        assert_eq!(text.unwrap(), "hello");
    }

    /// Polls a future that is ready at once.
    fn ready_now<T>(future: impl Future<Output = T>) -> T {
        let mut future = std::pin::pin!(future);
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => value,
            std::task::Poll::Pending => panic!("a picked file's read waits on nothing"),
        }
    }
}
