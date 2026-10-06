//! `FileField { crop }`: a picked image waits in a dialog for its crop before
//! the field takes it.

use dioxus::html::FileData;
use dioxus::prelude::*;

use super::files::Files;
use super::rows::FocusDebt;
use crate::{
    components::{
        buttons::Button,
        feedback::Alert,
        form::{CropOptions, CropRect, ImageCropper},
        layout::Flex,
        overlay::Dialog,
    },
    hooks::{ModalScope, use_localization, use_modal},
    localization::fill,
    platform,
    sx::sx,
};

/// The image types a canvas crops; an SVG or GIF passes through uncropped, as
/// an AVIF does on Blitz, whose decoder lacks it.
const CROPPED_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/bmp",
    #[cfg(not(feature = "native"))]
    "image/avif",
];

/// The one picked file, when it is an image `crop` applies to.
pub(super) fn croppable(files: &Files) -> Option<FileData> {
    let file = files.iter().next()?;
    let kind = file.content_type()?;
    (files.len() == 1 && CROPPED_TYPES.contains(&kind.as_str())).then(|| file.clone())
}

#[derive(Clone, PartialEq)]
struct CropArgs {
    file: FileData,
    /// `None` when the file's bytes would not read.
    src: Option<String>,
    options: CropOptions,
}

/// Opens the crop dialog for each file `pending` receives, then emits the
/// cropped file. Cancel drops it; a failed crop keeps the dialog open (1611).
#[component]
pub(super) fn CropGate(
    pending: Signal<Option<FileData>>,
    options: CropOptions,
    emit: Callback<Files>,
    /// Owed after Apply: a single-file dropzone puts away the Browse button
    /// the dialog would hand the focus back to.
    mut owed: Signal<Option<FocusDebt>>,
    oncrop: Option<EventHandler<CropRect>>,
) -> Element {
    let dialog = use_modal(|scope: ModalScope<CropArgs, (CropRect, FileData)>| {
        rsx! {
            CropDialog {
                args: scope.args(),
                close: Callback::new(move |()| scope.close()),
                resolve: Callback::new(move |cropped| scope.resolve(cropped)),
            }
        }
    });

    use_effect(move || {
        let Some(file) = pending() else {
            return;
        };
        pending.set(None);
        spawn(async move {
            let kind = file.content_type().unwrap_or_default();
            let src = file
                .read_bytes()
                .await
                .ok()
                .map(|bytes| platform::data_url(&kind, &bytes));
            let args = CropArgs { file, src, options };
            let Some((rect, cropped)) = dialog.open_with(args).await else {
                return;
            };
            if let Some(oncrop) = &oncrop {
                oncrop.call(rect);
            }
            owed.set(Some(FocusDebt::Took));
            emit.call(cropped.into());
        });
    });

    rsx! {}
}

#[component]
fn CropDialog(
    args: CropArgs,
    close: Callback<()>,
    resolve: Callback<(CropRect, FileData)>,
) -> Element {
    let localization = use_localization();
    let words = localization.image_cropper;
    let CropArgs { file, src, options } = args;
    let alt = fill(
        localization.file_field.crop_image,
        &[("name", &file.name())],
    );
    let mut rect = use_signal(|| None::<CropRect>);
    let unreadable = src.is_none();
    let mut failed = use_signal(|| false);
    let mut crop_failed = use_signal(|| false);
    let mut cropping = use_signal(|| false);

    let apply = move |_| {
        let Some(rect) = rect() else {
            return;
        };
        if cropping() {
            return;
        }
        cropping.set(true);
        crop_failed.set(false);
        let file = file.clone();
        spawn(async move {
            let fractions = [rect.x, rect.y, rect.width, rect.height];
            let cropped = match platform::image_crop() {
                Some(cropper) => cropper.crop(file, fractions, options.max_size).await,
                None => None,
            };
            cropping.set(false);
            match cropped {
                Some(cropped) => resolve.call((rect, cropped)),
                None => crop_failed.set(true),
            }
        });
    };

    rsx! {
        Dialog { title: words.title,
            Flex { direction: "column", gap: "md", align: "center",
                if let Some(src) = src {
                    ImageCropper {
                        src,
                        alt,
                        aspect: options.aspect,
                        shape: options.shape,
                        pan: options.pan,
                        value: rect(),
                        onchange: move |next| rect.set(Some(next)),
                        onerror: move |()| failed.set(true),
                        // Tall photos fit the viewport; the box maps onto what shows.
                        sx: sx().selector("& > [data-slot='image']", sx().max_height("60vh")),
                    }
                }
                if unreadable || failed() {
                    Alert { color: "error", role: "alert", width: "100%", "{words.load_failed}" }
                } else if crop_failed() {
                    Alert { color: "error", role: "alert", width: "100%", "{words.crop_failed}" }
                }
                Flex { gap: "sm", justify: "flex-end", width: "100%",
                    Button { variant: "outlined", onclick: move |_| close.call(()), "{words.cancel}" }
                    Button {
                        disabled: unreadable || failed() || rect().is_none(),
                        onclick: apply,
                        "{words.apply}"
                    }
                }
            }
        }
    }
}
