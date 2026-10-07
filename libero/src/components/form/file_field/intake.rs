use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::{
    components::form::{FormScope, Setter},
    hooks::{ElementHandle, current_localization, use_form_owner},
    localization::{FileFieldLabels, fill},
    platform::{ElementApi, next_task},
};

use super::accept::{accepts, picker_accepts};
use super::crop::croppable;
use super::files::{FileRejection, Files, RejectReason};
use super::rows::FocusDebt;

/// What the two paths that add files, pick and drop, need.
pub(super) struct Taking {
    pub(super) onchange: Option<EventHandler<Files>>,
    pub(super) setter: Option<Setter<Files>>,
    pub(super) onreject: Option<EventHandler<Vec<FileRejection>>>,
    /// What the chip announcer says of the last refusal, numbered per refusal.
    pub(super) refused: Signal<Option<(u64, String)>>,
    pub(super) accept: String,
    pub(super) multiple: bool,
    pub(super) editable: bool,
    /// Set with `crop`: a croppable pick waits here for the crop dialog.
    pub(super) crop: Option<Signal<Option<FileData>>>,
}

/// The one writer, and the focus debt every edit through it leaves.
#[derive(Clone, Copy)]
pub(super) struct Intake {
    pub(super) emit: Callback<Files>,
    /// What the next render owes the keyboard, after a removal or a pick
    /// destroyed the element focus was on.
    pub(super) owed: Signal<Option<FocusDebt>>,
    /// A drop.
    pub(super) take: Callback<Vec<FileData>>,
    /// A pick, which the picker filtered by `accept` already.
    pub(super) pick: Callback<Vec<FileData>>,
}

pub(super) fn use_file_intake(taking: Taking) -> Intake {
    let Taking {
        onchange,
        setter,
        onreject,
        mut refused,
        accept,
        multiple,
        editable,
        crop,
    } = taking;

    let emit = use_callback(move |files: Files| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(files),
        (None, Some(setter)) => setter.set(files),
        (None, None) => {}
    });

    let mut owed = use_signal(|| None::<FocusDebt>);

    let intake = use_callback(move |(files, picked): (Vec<FileData>, bool)| {
        if !editable {
            return;
        }
        let (kept, rejected) = keep_accepted(files, &accept, multiple, picked);
        if !rejected.is_empty() {
            let note = rejection_note(&rejected, &current_localization().file_field);
            let count = refused.peek().as_ref().map_or(0, |(count, _)| *count);
            refused.set(Some((count + 1, note)));
            if let Some(onreject) = &onreject {
                onreject.call(rejected);
            }
        }
        if let Some(mut pending) = crop
            && let Some(file) = croppable(&kept)
        {
            pending.set(Some(file));
        } else if !kept.is_empty() {
            owed.set(Some(FocusDebt::Took));
            emit.call(kept);
        }
    });
    let take = use_callback(move |files| intake.call((files, false)));
    let pick = use_callback(move |files| intake.call((files, true)));

    Intake {
        emit,
        owed,
        take,
        pick,
    }
}

/// The files a pick or a drop leaves after `accept` and `multiple`, and the
/// rest. A drop skips the picker's own `accept`, so it is applied here.
fn keep_accepted(
    files: Vec<FileData>,
    accept: &str,
    multiple: bool,
    picked: bool,
) -> (Files, Vec<FileRejection>) {
    let check = match picked {
        true => picker_accepts,
        false => accepts,
    };
    let (mut kept, mut rejected) = (Vec::new(), Vec::new());
    for file in files {
        let reason = if !check(accept, &file.name(), file.content_type().as_deref()) {
            Some(RejectReason::Type)
        } else if !multiple && !kept.is_empty() {
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
fn rejection_note(rejected: &[FileRejection], words: &FileFieldLabels) -> String {
    let names = |reason: RejectReason| {
        rejected
            .iter()
            .filter(|rejection| rejection.reason == reason)
            .map(|rejection| rejection.file.name())
            .collect::<Vec<_>>()
            .join(", ")
    };
    [
        (RejectReason::Type, words.rejected_type),
        (RejectReason::TooMany, words.rejected_many),
    ]
    .into_iter()
    .map(|(reason, template)| (names(reason), template))
    .filter(|(names, _)| !names.is_empty())
    .map(|(names, template)| fill(template, &[("names", &names)]))
    .collect::<Vec<_>>()
    .join(" ")
}

/// Writes the caller's value back into the input's `FileList`, what a form
/// posts, whenever the two disagree.
pub(super) fn use_input_mirror(input_element: ElementHandle, synced: Files) {
    let mut mirrored = use_signal(Mirrored::default);
    // The token, not `is_mounted`: a `variant` switch mounts a fresh, empty
    // input that must be rewritten too.
    let mount = input_element.mount_token();
    // A native reset empties the input while a value of the caller's own stays.
    let scope = try_use_context::<FormScope>();
    let owner = use_form_owner(input_element, scope.is_none());
    let resets = scope.map_or(0, |form| form.resets()) + owner().unwrap_or(0);
    use_effect(use_reactive!(|(synced, mount, resets)| {
        let next = Mirrored {
            mount,
            resets,
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
    files: Files,
}
