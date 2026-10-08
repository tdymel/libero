//! `FileField`, as a dropzone taking several files.

use dioxus::prelude::*;
use libero::components::{
    Button, CropOptions, CropRect, FileField, FileRejection, Files, Flex, Form, use_form,
};

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/file-field", || rsx! { FileFieldPage {} }),
    ("/file-field/input", || rsx! { FileInputPage {} }),
    ("/file-field/states", || rsx! { FileStatesPage {} }),
    ("/file-field/modes", || rsx! { FileModesPage {} }),
    ("/file-field/pick", || rsx! { FilePickPage {} }),
    ("/file-field/crop", || rsx! { FileCropPage {} }),
    ("/file-field/reject", || rsx! { FileRejectPage {} }),
    ("/file-field/reset", || rsx! { FileResetPage {} }),
    ("/file-field/single", || rsx! { FileSinglePage {} }),
];

/// Todos 2581, 2582: a single-file dropzone with an error that the button turns read-only
/// once the unit has dropped its file in.
#[component]
fn FileSinglePage() -> Element {
    let mut files = use_signal(Files::default);
    let mut readonly = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            Button { id: "to-readonly", onclick: move |_| readonly.set(true), "Read-only" }
            FileField {
                id: "single",
                label: "Contract",
                variant: "dropzone",
                status: "We cannot read that file.",
                readonly: readonly(),
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
        }
    }
}

/// Todo 2551: a field holding its own value, in a raw `<form>` and in a `Form`,
/// each with a native reset button.
#[component]
fn FileResetPage() -> Element {
    let mut raw = use_signal(Files::default);
    let mut held = use_signal(Files::default);
    let value = use_store(String::new);
    let form = use_form();

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            form {
                FileField { id: "raw", label: "Raw", value: raw(), onchange: move |next: Files| raw.set(next) }
                button { id: "raw-reset", r#type: "reset", "Reset raw" }
            }
            Form { value, form,
                FileField { id: "held", label: "Held", value: held(), onchange: move |next: Files| held.set(next) }
                button { id: "held-reset", r#type: "reset", "Reset held" }
            }
        }
    }
}

/// The next file chooser answers itself with a 40 x 20 PNG, as no driver answers
/// GTK's (1126): a detached input (the WebViews') at `click()`, an attached one at its event.
const STUB_PICKER: &str = "const click = HTMLInputElement.prototype.click;
    const answer = (input) => {
        HTMLInputElement.prototype.click = click;
        document.removeEventListener('click', onClick, true);
        const bytes = Uint8Array.from(atob('iVBORw0KGgoAAAANSUhEUgAAACgAAAAUCAIAAABwJOjsAAAAJ0lEQVR42mP4z8BANqJA63+GUYtHLR61eNTiUYtHLR61eNTikWMxAGDtHQ7V1k7XAAAAAElFTkSuQmCC'), (ch) => ch.charCodeAt(0));
        const files = new DataTransfer();
        files.items.add(new File([bytes], 'e2e-crop-picked.png', { type: 'image/png' }));
        input.files = files.files;
        input.dispatchEvent(new Event('input', { bubbles: true }));
        input.dispatchEvent(new Event('change', { bubbles: true }));
    };
    const onClick = (event) => {
        if (event.target instanceof HTMLInputElement && event.target.type === 'file') {
            event.preventDefault();
            answer(event.target);
        }
    };
    document.addEventListener('click', onClick, true);
    HTMLInputElement.prototype.click = function () {
        if (this.type !== 'file' || this.isConnected) return click.call(this);
        answer(this);
    };";

/// A square crop of a dropped image (1218): `#rect` reads the crop in whole
/// percent, `#out` the file `onchange` got, with its PNG size. `#stub-picker`
/// makes Browse pick the PNG, for the WebViews' crop round trip (1230).
#[component]
fn FileCropPage() -> Element {
    let mut files = use_signal(Files::default);
    let mut rect = use_signal(String::new);
    let mut out = use_signal(String::new);
    let mut stubbed = use_signal(|| false);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            Button {
                id: "stub-picker",
                onclick: move |_| async move {
                    let _ = document::eval(STUB_PICKER).await;
                    stubbed.set(true);
                },
                "Stub the picker"
            }
            FileField {
                id: "avatar",
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
            p { id: "stubbed", "{stubbed}" }
            p { id: "rect", "{rect}" }
            p { id: "out", "{out}" }
        }
    }
}

/// A single-file `image/*` field: `#rejected` lists what `onreject` got, as
/// `name:reason` (1819).
#[component]
fn FileRejectPage() -> Element {
    let mut files = use_signal(Files::default);
    let mut rejected = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "photo",
                label: "Photo",
                accept: "image/*",
                value: files(),
                onchange: move |next: Files| files.set(next),
                onreject: move |refused: Vec<FileRejection>| {
                    let named: Vec<String> = refused
                        .iter()
                        .map(|rejection| format!("{}:{:?}", rejection.file.name(), rejection.reason))
                        .collect();
                    rejected.set(named.join(","));
                },
            }
            p { id: "rejected", "{rejected}" }
        }
    }
}

/// One file from the chooser, whose text shows once read: the bytes arrived.
#[component]
fn FilePickPage() -> Element {
    let mut files = use_signal(Files::default);
    let mut read = use_signal(String::new);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "receipt",
                label: "Receipt",
                placeholder: "Pick a file",
                value: files(),
                onchange: move |next: Files| {
                    if let Some(file) = next.iter().next().cloned() {
                        spawn(async move {
                            read.set(file.read_string().await.unwrap_or_default());
                        });
                    }
                    files.set(next);
                },
            }
            p { id: "read", "{read}" }
        }
    }
}

/// One `multiple` field the buttons switch to read-only or disabled once the
/// unit has dropped its files in.
#[component]
fn FileModesPage() -> Element {
    let mut files = use_signal(Files::default);
    let mut mode = use_signal(|| "editable");

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "modes",
                label: "Papers",
                placeholder: "Pick files",
                multiple: true,
                readonly: mode() == "readonly",
                disabled: mode() == "disabled",
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
            Button { id: "to-readonly", onclick: move |_| mode.set("readonly"), "Read-only" }
            Button { id: "to-disabled", onclick: move |_| mode.set("disabled"), "Disabled" }
        }
    }
}

/// The `Input` variant bare (no placeholder, todo 520), and required with an
/// error.
#[component]
fn FileStatesPage() -> Element {
    let mut bare = use_signal(Files::default);
    let mut contract = use_signal(Files::default);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "bare",
                label: "Receipt",
                value: bare(),
                onchange: move |next: Files| bare.set(next),
            }
            FileField {
                id: "contract",
                label: "Contract",
                placeholder: "Pick a file",
                required: true,
                status: "We cannot read that file.",
                multiple: true,
                value: contract(),
                onchange: move |next: Files| contract.set(next),
            }
        }
    }
}

/// The `Input` variant: dropped files become chips the arrows walk.
#[component]
fn FileInputPage() -> Element {
    let mut files = use_signal(Files::default);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "attachments",
                label: "Attachments",
                placeholder: "Pick files",
                multiple: true,
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
        }
    }
}

/// Starts empty: a `FileData` only comes from a pick or a drop, so the unit
/// drops its files in.
#[component]
fn FileFieldPage() -> Element {
    let mut files = use_signal(Files::default);

    rsx! {
        Flex { direction: "column", gap: "md", max_width: "360px",
            FileField {
                id: "attachments",
                label: "Attachments",
                variant: "dropzone",
                multiple: true,
                value: files(),
                onchange: move |next: Files| files.set(next),
            }
        }
    }
}
