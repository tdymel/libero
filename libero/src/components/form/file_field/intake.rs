use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::{
    components::form::{Edited, FormScope, Setter},
    hooks::{ElementHandle, current_formats, current_localization, use_form_owner},
    localization::{FileFieldLabels, Formats, fill},
    platform::{ElementApi, next_task},
};

use super::accept::{accepts, picker_accepts};
#[cfg(feature = "image-cropper")]
use super::crop::{croppable, crops};
use super::files::{FileRejection, Files, RejectReason, format_size};
use super::rows::FocusDebt;

/// What the two paths that add files, pick and drop, need.
pub(super) struct Taking {
    pub(super) onchange: Option<EventHandler<Files>>,
    pub(super) setter: Option<Setter<Files>>,
    pub(super) onreject: Option<EventHandler<Vec<FileRejection>>>,
    /// What the chip announcer says of the last refusal, numbered per refusal.
    pub(super) refused: Signal<Option<(u64, String)>>,
    /// Marked on every emit, so the chip announcer says the field's own edits only.
    pub(super) edited: Edited,
    pub(super) accept: String,
    pub(super) limits: Limits,
    pub(super) editable: bool,
    /// Set with `crop`: a croppable pick waits here for the crop dialog.
    #[cfg(feature = "image-cropper")]
    pub(super) crop: Option<Signal<Option<FileData>>>,
}

/// The one writer, and the focus debt every edit through it leaves.
#[derive(Clone, Copy)]
pub(super) struct Intake {
    pub(super) emit: Callback<Files>,
    /// What the next render owes the keyboard, after a removal or a pick
    /// destroyed the element focus was on.
    pub(super) owed: Signal<Option<FocusDebt>>,
    /// Counts the picks and drops, kept or not: each one may have left the
    /// input's `FileList` out of step with the value.
    pub(super) intakes: Signal<u32>,
    /// A drop.
    pub(super) take: Callback<Vec<FileData>>,
    /// A pick, which the picker filtered by `accept` already.
    pub(super) pick: Callback<Vec<FileData>>,
    /// The crop dialog's output, held to `max_size` before the field takes it.
    #[cfg(feature = "image-cropper")]
    pub(super) cropped: Callback<Files>,
}

pub(super) fn use_file_intake(taking: Taking) -> Intake {
    let Taking {
        onchange,
        setter,
        onreject,
        mut refused,
        edited,
        accept,
        limits,
        editable,
        #[cfg(feature = "image-cropper")]
        crop,
    } = taking;

    let emit = use_callback(move |files: Files| {
        edited.mark();
        match (&onchange, &setter) {
            (Some(onchange), _) => onchange.call(files),
            (None, Some(setter)) => setter.set(files),
            (None, None) => {}
        }
    });

    let mut owed = use_signal(|| None::<FocusDebt>);
    let mut intakes = use_signal(|| 0u32);

    let refuse = use_callback(move |rejected: Vec<FileRejection>| {
        if rejected.is_empty() {
            return;
        }
        let words = &current_localization().file_field;
        let note = rejection_note(&rejected, limits, words, current_formats());
        let count = refused.peek().as_ref().map_or(0, |(count, _)| *count);
        refused.set(Some((count + 1, note)));
        if let Some(onreject) = &onreject {
            onreject.call(rejected);
        }
    });

    // The crop's output is what the field keeps, so its size is checked then.
    #[cfg(feature = "image-cropper")]
    let cropped = use_callback(move |files: Files| {
        let (kept, rejected) = keep_accepted(files.into_vec(), "", limits, true, |_| false);
        refuse.call(rejected);
        if !kept.is_empty() {
            owed.set(Some(FocusDebt::Took));
            emit.call(kept);
        }
    });

    let intake = use_callback(move |(files, picked): (Vec<FileData>, bool)| {
        intakes.with_mut(|count| *count = count.wrapping_add(1));
        if !editable {
            return;
        }
        #[cfg(feature = "image-cropper")]
        let awaits_crop = move |file: &FileData| crop.is_some() && crops(file);
        #[cfg(not(feature = "image-cropper"))]
        let awaits_crop = |_: &FileData| false;
        let (kept, rejected) = keep_accepted(files, &accept, limits, picked, awaits_crop);
        refuse.call(rejected);
        #[cfg(feature = "image-cropper")]
        if let Some(mut pending) = crop
            && let Some(file) = croppable(&kept)
        {
            pending.set(Some(file));
            return;
        }
        if !kept.is_empty() {
            owed.set(Some(FocusDebt::Took));
            emit.call(kept);
        }
    });
    let take = use_callback(move |files| intake.call((files, false)));
    let pick = use_callback(move |files| intake.call((files, true)));

    Intake {
        emit,
        owed,
        intakes,
        take,
        pick,
        #[cfg(feature = "image-cropper")]
        cropped,
    }
}

/// How many files a pick or a drop may leave, and how large each.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct Limits {
    pub(super) multiple: bool,
    pub(super) max_files: Option<usize>,
    pub(super) max_size: Option<u64>,
}

impl Limits {
    /// One without `multiple`, else `max_files`.
    fn count(&self) -> Option<usize> {
        match self.multiple {
            true => self.max_files,
            false => Some(1),
        }
    }
}

/// The files a pick or a drop leaves after `accept` and the limits, and the
/// rest. A drop skips the picker's own `accept`, so it is applied here. A file
/// `awaits_crop` skips `max_size`: its size is the cut's.
fn keep_accepted(
    files: Vec<FileData>,
    accept: &str,
    limits: Limits,
    picked: bool,
    awaits_crop: impl Fn(&FileData) -> bool,
) -> (Files, Vec<FileRejection>) {
    let check = match picked {
        true => picker_accepts,
        false => accepts,
    };
    let (mut kept, mut rejected) = (Vec::new(), Vec::new());
    for file in files {
        let reason = if !check(accept, &file.name(), file.content_type().as_deref()) {
            Some(RejectReason::Type)
        } else if limits.max_size.is_some_and(|max| file.size() > max) && !awaits_crop(&file) {
            Some(RejectReason::TooLarge)
        } else if limits.count().is_some_and(|max| kept.len() >= max) {
            Some(RejectReason::TooMany)
        } else {
            None
        };
        match reason {
            Some(reason) => rejected.push(FileRejection { file, reason }),
            None => kept.push(file),
        }
    }
    (kept.into(), rejected)
}

/// One sentence per reason, naming the refused files.
fn rejection_note(
    rejected: &[FileRejection],
    limits: Limits,
    words: &FileFieldLabels,
    formats: &Formats,
) -> String {
    let names = |reason: RejectReason| {
        rejected
            .iter()
            .filter(|rejection| rejection.reason == reason)
            .map(|rejection| rejection.file.name())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let (many, max_files) = match limits.count() {
        Some(max) if max > 1 => (words.rejected_max, max.to_string()),
        _ => (words.rejected_many, String::new()),
    };
    let max_size = limits
        .max_size
        .map(|max| format_size(max, words, formats))
        .unwrap_or_default();
    [
        (RejectReason::Type, words.rejected_type, String::new()),
        (RejectReason::TooLarge, words.rejected_large, max_size),
        (RejectReason::TooMany, many, max_files),
    ]
    .into_iter()
    .map(|(reason, template, max)| (names(reason), template, max))
    .filter(|(names, _, _)| !names.is_empty())
    .map(|(names, template, max)| fill(template, &[("names", &names), ("max", &max)]))
    .collect::<Vec<_>>()
    .join(" ")
}

/// Writes the caller's value back into the input's `FileList`, what a form
/// posts, whenever the two disagree: also after a pick that changed no value.
pub(super) fn use_input_mirror(input_element: ElementHandle, synced: Files, intakes: u32) {
    let mut mirrored = use_signal(Mirrored::default);
    // The token, not `is_mounted`: a `variant` switch mounts a fresh, empty
    // input that must be rewritten too.
    let mount = input_element.mount_token();
    // A native reset empties the input while a value of the caller's own stays.
    let scope = try_use_context::<FormScope>();
    let owner = use_form_owner(input_element, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    use_effect(use_reactive!(|(synced, mount, resets, intakes)| {
        let next = Mirrored {
            mount,
            resets,
            intakes,
            files: synced,
        };
        if *mirrored.peek() == next {
            return;
        }
        let mut write = move |next: Mirrored| {
            if input_element.set_files(&next.files).is_ok() {
                mirrored.set(next);
            }
        };
        // Read first: the scrutinee's borrow would outlive `write`'s `set`.
        let after_reset = mirrored.peek().resets != resets;
        match after_reset {
            false => write(next),
            // The `reset` event comes before the browser empties the input.
            true => {
                spawn(async move {
                    next_task().await;
                    write(next);
                });
            }
        }
    }));
}

/// What the input's `FileList` was last written from.
#[derive(Clone, Default, PartialEq)]
struct Mirrored {
    mount: Option<usize>,
    resets: u32,
    intakes: u32,
    files: Files,
}

#[cfg(test)]
mod tests {
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
        ) -> Pin<Box<dyn futures_core::Stream<Item = Result<Bytes, CapturedError>> + Send>>
        {
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
}
