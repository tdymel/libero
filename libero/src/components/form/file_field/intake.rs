use dioxus::html::FileData;
use dioxus::prelude::*;

use crate::{components::form::Setter, hooks::ElementHandle, platform::ElementApi, utils::warn};

use super::accept::accepts;
use super::files::Files;
use super::rows::FocusDebt;

/// What the two paths that add files, pick and drop, need.
pub(super) struct Taking {
    pub(super) onchange: Option<EventHandler<Files>>,
    pub(super) setter: Option<Setter<Files>>,
    pub(super) accept: String,
    pub(super) multiple: bool,
    pub(super) editable: bool,
}

/// The one writer, and the focus debt every edit through it leaves.
#[derive(Clone, Copy)]
pub(super) struct Intake {
    pub(super) emit: Callback<Files>,
    /// What the next render owes the keyboard, after a removal or a pick
    /// destroyed the element focus was on.
    pub(super) owed: Signal<Option<FocusDebt>>,
    pub(super) take: Callback<Vec<FileData>>,
}

pub(super) fn use_file_intake(taking: Taking) -> Intake {
    let Taking {
        onchange,
        setter,
        accept,
        multiple,
        editable,
    } = taking;

    let emit = use_callback(move |files: Files| match (&onchange, &setter) {
        (Some(onchange), _) => onchange.call(files),
        (None, Some(setter)) => setter.set(files),
        (None, None) => {}
    });

    let mut owed = use_signal(|| None::<FocusDebt>);

    let take = use_callback(move |files: Vec<FileData>| {
        if !editable {
            return;
        }
        let kept = keep_accepted(files, &accept, multiple);
        if !kept.is_empty() {
            owed.set(Some(FocusDebt::Took));
            emit.call(kept);
        }
    });

    Intake { emit, owed, take }
}

/// The files a pick or a drop leaves after `accept` and `multiple`. A drop
/// skips the picker's own `accept`, so it is applied here, with a warning.
fn keep_accepted(files: Vec<FileData>, accept: &str, multiple: bool) -> Files {
    let picked = files.len();
    let kept: Files = files
        .into_iter()
        .filter(|file| accepts(accept, &file.name(), file.content_type().as_deref()))
        .collect::<Files>()
        .truncated(multiple);
    if kept.len() < picked {
        warn("FileField: dropped files that `accept` or `multiple` excludes.");
    }
    kept
}

/// Writes the caller's value back into the input's `FileList`, what a form
/// posts, whenever the two disagree.
pub(super) fn use_input_mirror(
    input_element: ElementHandle,
    mut mirrored: Signal<(Option<usize>, Files)>,
    synced: Files,
) {
    // The token, not `is_mounted`: a `variant` switch mounts a fresh, empty
    // input that must be rewritten too.
    let mount = input_element.mount_token();
    use_effect(use_reactive!(|(synced, mount)| {
        let (mirrored_mount, mirrored_files) = mirrored.peek().clone();
        if mirrored_mount == mount && mirrored_files == synced {
            return;
        }
        if input_element.set_files(&synced).is_ok() {
            mirrored.set((mount, synced));
        }
    }));
}
