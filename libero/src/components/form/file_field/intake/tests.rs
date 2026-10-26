use std::{any::Any, future::Future, path::PathBuf, pin::Pin};

use dioxus::{
    CapturedError,
    html::{NativeFileData, bytes::Bytes},
};

use super::*;

type Read<T> = Pin<Box<dyn Future<Output = Result<T, CapturedError>>>>;

/// A file with a name, a type and a size, and nothing to read.
struct Stub(&'static str, &'static str, u64);

impl NativeFileData for Stub {
    fn name(&self) -> String {
        self.0.to_string()
    }
    fn size(&self) -> u64 {
        self.2
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn path(&self) -> PathBuf {
        self.0.into()
    }
    fn content_type(&self) -> Option<String> {
        Some(self.1.to_string())
    }
    fn read_bytes(&self) -> Read<Bytes> {
        unimplemented!("a size check reads nothing")
    }
    fn byte_stream(
        &self,
    ) -> Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, CapturedError>> + Send>> {
        unimplemented!("a size check reads nothing")
    }
    fn read_string(&self) -> Read<String> {
        unimplemented!("a size check reads nothing")
    }
    fn inner(&self) -> &dyn Any {
        self
    }
}

fn file(name: &'static str, kind: &'static str, size: u64) -> FileData {
    FileData::new(Stub(name, kind, size))
}

/// Todo 4298: a photo past `max_size` waits for its crop, which is held to it instead.
#[test]
fn a_file_awaiting_its_crop_skips_the_size_limit() {
    let limits = Limits {
        max_size: Some(2),
        ..Limits::default()
    };
    let photo = file("a.jpg", "image/jpeg", 6);
    let by_type = |file: &FileData| file.content_type().as_deref() == Some("image/jpeg");
    let (kept, rejected) = keep_accepted(vec![photo.clone()], "", limits, true, by_type);
    assert_eq!(kept.len(), 1);
    assert!(rejected.is_empty());

    let (kept, rejected) = keep_accepted(vec![photo], "", limits, true, |_| false);
    assert!(kept.is_empty());
    assert_eq!(rejected[0].reason, RejectReason::TooLarge);

    let document = file("a.pdf", "application/pdf", 6);
    let (kept, rejected) = keep_accepted(vec![document], "", limits, true, by_type);
    assert!(kept.is_empty());
    assert_eq!(rejected[0].reason, RejectReason::TooLarge);
}
