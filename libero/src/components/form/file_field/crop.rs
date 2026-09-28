//! `FileField { crop }`: a picked image waits in a dialog for its crop before
//! the field takes it.

use dioxus::html::FileData;
use dioxus::prelude::*;

use super::files::Files;
use crate::{
    components::{
        buttons::Button,
        feedback::Alert,
        form::{CropOptions, CropRect, ImageCropper},
        layout::Flex,
        overlay::Dialog,
    },
    hooks::{ModalScope, use_localization, use_modal},
    platform,
    sx::sx,
};

/// The image types a canvas crops; an SVG or GIF passes through uncropped.
const CROPPED_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/webp",
    "image/bmp",
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
    src: String,
    alt: String,
    options: CropOptions,
}

/// Opens the crop dialog for each file `pending` receives, then emits the
/// cropped file, or the original where nothing can crop. Cancel drops it.
#[component]
pub(super) fn CropGate(
    pending: Signal<Option<FileData>>,
    options: CropOptions,
    emit: Callback<Files>,
    oncrop: Option<EventHandler<CropRect>>,
) -> Element {
    let dialog = use_modal(|scope: ModalScope<CropArgs, CropRect>| {
        rsx! {
            CropDialog {
                args: scope.args(),
                close: Callback::new(move |()| scope.close()),
                resolve: Callback::new(move |rect| scope.resolve(rect)),
            }
        }
    });

    use_effect(move || {
        let Some(file) = pending() else {
            return;
        };
        pending.set(None);
        spawn(async move {
            let Ok(bytes) = file.read_bytes().await else {
                emit.call(file.into());
                return;
            };
            let kind = file.content_type().unwrap_or_default();
            let args = CropArgs {
                src: platform::data_url(&kind, &bytes),
                alt: file.name(),
                options,
            };
            let Some(rect) = dialog.open_with(args).await else {
                return;
            };
            if let Some(oncrop) = &oncrop {
                oncrop.call(rect);
            }
            let fractions = [rect.x, rect.y, rect.width, rect.height];
            let cropped = match platform::image_crop() {
                Some(cropper) => {
                    cropper
                        .crop(file.clone(), fractions, options.max_size)
                        .await
                }
                None => None,
            };
            emit.call(cropped.unwrap_or(file).into());
        });
    });

    rsx! {}
}

#[component]
fn CropDialog(args: CropArgs, close: Callback<()>, resolve: Callback<CropRect>) -> Element {
    let words = use_localization().image_cropper;
    let CropArgs { src, alt, options } = args;
    let mut rect = use_signal(|| None::<CropRect>);
    let mut failed = use_signal(|| false);

    rsx! {
        Dialog { title: words.title,
            Flex { direction: "column", gap: "md", align: "center",
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
                if failed() {
                    Alert { color: "error", role: "alert", width: "100%", "{words.load_failed}" }
                }
                Flex { gap: "sm", justify: "flex-end", width: "100%",
                    Button { variant: "outlined", onclick: move |_| close.call(()), "{words.cancel}" }
                    Button {
                        disabled: failed() || rect().is_none(),
                        onclick: move |_| {
                            if let Some(rect) = rect() {
                                resolve.call(rect);
                            }
                        },
                        "{words.apply}"
                    }
                }
            }
        }
    }
}
