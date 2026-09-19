//! `FileField` natively: no drop reaches Blitz, so the files come in through
//! `value`. Where focus goes once the file it was on is removed (todo 406).

use dioxus::CapturedError;
use dioxus::html::{FileData, NativeFileData, bytes::Bytes};
use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{FileField, Files};

struct FakeFile(&'static str);

type Read<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, CapturedError>>>>;

impl NativeFileData for FakeFile {
    fn name(&self) -> String {
        self.0.to_string()
    }
    fn size(&self) -> u64 {
        0
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn path(&self) -> std::path::PathBuf {
        self.0.into()
    }
    fn content_type(&self) -> Option<String> {
        None
    }
    fn read_bytes(&self) -> Read<Bytes> {
        unimplemented!("never read")
    }
    fn byte_stream(
        &self,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Bytes, CapturedError>> + Send>> {
        unimplemented!("never read")
    }
    fn read_string(&self) -> Read<String> {
        unimplemented!("never read")
    }
    fn inner(&self) -> &dyn std::any::Any {
        self
    }
}

fn two_files() -> Files {
    ["alpha.txt", "beta.txt"]
        .into_iter()
        .map(|name| FileData::new(FakeFile(name)))
        .collect()
}

fn chips() -> Element {
    let mut files = use_signal(two_files);
    rsx! {
        FileField {
            id: "contract",
            label: "Contract",
            multiple: true,
            value: files(),
            onchange: move |next| files.set(next),
        }
    }
}

fn cards() -> Element {
    let mut files = use_signal(two_files);
    rsx! {
        FileField {
            label: "Drop",
            multiple: true,
            variant: "dropzone",
            value: files(),
            onchange: move |next| files.set(next),
        }
    }
}

#[test]
fn deleting_a_chip_moves_focus_to_the_one_left_then_to_browse() {
    let mut page = mount(chips);
    page.focus("#contract-file-1");
    page.press(Key::Delete);
    assert!(
        page.is_focused("#contract-file-0"),
        "after removing beta: {}",
        page.focus_owner()
    );
    page.press(Key::Delete);
    assert!(
        page.is_focused("#contract"),
        "after removing alpha: {}",
        page.focus_owner()
    );
    assert!(!page.exists("#contract-file-0"));
}

#[test]
fn removing_a_card_moves_focus_to_what_took_its_place() {
    let mut page = mount(cards);
    page.focus("[aria-label=\"Remove beta.txt\"]");
    page.press(Key::Enter);
    assert!(
        page.is_focused("[aria-label=\"Remove alpha.txt\"]"),
        "after removing beta: {}",
        page.focus_owner()
    );
    page.press(Key::Character(" ".into()));
    assert!(
        page.is_focused("[data-slot=browse]"),
        "after removing alpha: {}",
        page.focus_owner()
    );
}
