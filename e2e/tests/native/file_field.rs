//! `FileField` natively: no drop reaches Blitz, so the files come in through
//! `value`, or from a pick through `fake_file_dialog`. Where focus goes once the
//! file it was on is removed (todo 406).

use dioxus::CapturedError;
use dioxus::html::{FileData, NativeFileData, bytes::Bytes};
use dioxus::prelude::*;
use e2e::native::{Key, mount};
use libero::components::{CropOptions, CropRect, FileField, Files};
use libero::platform::fake_file_dialog;

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

/// A 40 x 20 blue PNG.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x14, 0x08, 0x02, 0x00, 0x00, 0x00, 0x70, 0x24, 0xe8,
    0xec, 0x00, 0x00, 0x00, 0x24, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x30, 0x9e, 0xf5, 0x61,
    0x40, 0x10, 0xc3, 0xa8, 0xc5, 0xa3, 0x16, 0x8f, 0x5a, 0x3c, 0x6a, 0xf1, 0xa8, 0xc5, 0xa3, 0x16,
    0x8f, 0x5a, 0x3c, 0x6a, 0xf1, 0xc8, 0xb1, 0x18, 0x00, 0x1c, 0x28, 0x6e, 0xec, 0x01, 0x3a, 0xc9,
    0x68, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

struct Picture;

impl NativeFileData for Picture {
    fn name(&self) -> String {
        "photo.png".into()
    }
    fn size(&self) -> u64 {
        PNG.len() as u64
    }
    fn last_modified(&self) -> u64 {
        0
    }
    fn path(&self) -> std::path::PathBuf {
        "photo.png".into()
    }
    fn content_type(&self) -> Option<String> {
        Some("image/png".into())
    }
    fn read_bytes(&self) -> Read<Bytes> {
        Box::pin(std::future::ready(Ok(Bytes::from_static(PNG))))
    }
    fn byte_stream(
        &self,
    ) -> std::pin::Pin<Box<dyn futures::Stream<Item = Result<Bytes, CapturedError>> + Send>> {
        unimplemented!("never streamed")
    }
    fn read_string(&self) -> Read<String> {
        unimplemented!("never read as text")
    }
    fn inner(&self) -> &dyn std::any::Any {
        self
    }
}

// As the browser's `/file-field/crop`: `#rect` the crop in whole percent, `#out`
// the file `onchange` got with its PNG size.
fn cropping() -> Element {
    let mut files = use_signal(Files::default);
    let mut rect = use_signal(String::new);
    let mut out = use_signal(String::new);
    rsx! {
        FileField {
            label: "Avatar",
            variant: "dropzone",
            accept: "image/*",
            crop: CropOptions { aspect: Some(1.0), max_size: Some(16), ..CropOptions::default() },
            value: files(),
            oncrop: move |crop: CropRect| {
                let percent = |fraction: f64| (fraction * 100.0).round();
                rect.set(format!(
                    "{},{},{},{}",
                    percent(crop.x),
                    percent(crop.y),
                    percent(crop.width),
                    percent(crop.height)
                ));
            },
            onchange: move |next: Files| {
                if let Some(file) = next.one() {
                    spawn(async move {
                        let bytes = file.read_bytes().await.unwrap_or_default();
                        // A PNG's IHDR: width and height, big-endian, from byte 16.
                        let size = |at: usize| {
                            bytes.get(at..at + 4).map_or(0, |b| u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
                        };
                        out.set(format!(
                            "{} {} {}x{}",
                            file.name(),
                            file.content_type().unwrap_or_default(),
                            size(16),
                            size(20)
                        ));
                    });
                }
                files.set(next);
            },
        }
        p { id: "rect", "{rect}" }
        p { id: "out", "{out}" }
    }
}

/// Todo 1514: Browse's pick (the faked dialog) waits in the crop dialog, and
/// Apply cuts it in Rust (`image_crop::native`) before `onchange` gets it.
#[test]
fn a_picked_image_is_cropped_natively_before_the_field_takes_it() {
    const APPLY: &str = "[role=dialog] button[data-state~=filled]";
    let mut page = mount(cropping);
    fake_file_dialog::answer(vec![FileData::new(Picture)]);
    page.click("[data-slot=browse]");
    let ready = page.wait_for(|page| page.exists(APPLY) && page.attr(APPLY, "disabled").is_none());
    assert!(ready, "no crop dialog to apply: {}", page.tree());
    page.click(APPLY);
    let done = page.wait_for(|page| !page.text("#out").is_empty());
    assert!(done, "the field got no file: {}", page.tree());
    assert_eq!(page.text("#out"), "photo.png image/png 16x16");
    // The opening box: a 16px square centred in the 40 x 20 picture.
    assert_eq!(page.text("#rect"), "30,10,40,80");
    assert!(!page.exists("[role=dialog]"), "the dialog stayed open");
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
