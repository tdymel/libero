use std::{future::Future, pin::Pin};

use dioxus::html::FileData;

/// A crop as fractions 0-1 of the image: `[x, y, width, height]`. Not the
/// component's rect type: this layer sits below the components.
pub(crate) type Fractions = [f64; 4];

/// The cropped file, `None` when the image would not decode or encode.
pub(crate) type Cropped = Pin<Box<dyn Future<Output = Option<FileData>>>>;

/// Cuts an image file to a crop, through the page's canvas.
pub(crate) trait ImageCropApi {
    /// `file` cut to `rect`, its longer side scaled down to `max` px. Keeps
    /// a PNG, JPEG or WebP's type; anything else comes back a PNG.
    fn crop(&self, file: FileData, rect: Fractions, max: Option<u32>) -> Cropped;
}

/// `crop(bytes, type, rect, max)` resolving to a `Blob`, run by a browser and a
/// WebView alike. `createImageBitmap` turns the picture as its EXIF says, as `<img>` does.
#[cfg(not(feature = "native"))]
pub(crate) const CROP_SCRIPT: &str = "const crop = async (bytes, type, [x, y, w, h], max) => {
    const bitmap = await createImageBitmap(new Blob([bytes], { type }));
    const sx = Math.round(x * bitmap.width), sy = Math.round(y * bitmap.height);
    const sw = Math.max(1, Math.round(w * bitmap.width)), sh = Math.max(1, Math.round(h * bitmap.height));
    const scale = max ? Math.min(1, max / Math.max(sw, sh)) : 1;
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(sw * scale));
    canvas.height = Math.max(1, Math.round(sh * scale));
    canvas.getContext('2d').drawImage(bitmap, sx, sy, sw, sh, 0, 0, canvas.width, canvas.height);
    bitmap.close();
    const out = ['image/png', 'image/jpeg', 'image/webp'].includes(type) ? type : 'image/png';
    return await new Promise((done, fail) =>
        canvas.toBlob((blob) => (blob ? done(blob) : fail()), out, 0.92));
};";

/// The file's name for its new type: the extension swapped when the type changed.
#[cfg(any(test, not(feature = "native")))]
pub(crate) fn cropped_name(name: &str, from: &str, to: &str) -> String {
    if from == to {
        return name.to_string();
    }
    let extension = to.rsplit('/').next().unwrap_or("png");
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    format!("{stem}.{extension}")
}

pub(crate) use crate::utils::bytes_data_url as data_url;

#[cfg(target_arch = "wasm32")]
pub(super) mod web {
    use std::{future::Future, pin::Pin};

    use dioxus::html::{FileData, NativeFileData, bytes::Bytes};
    use js_sys::{Array, Function, Promise, Uint8Array};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    use super::{CROP_SCRIPT, Cropped, Fractions, ImageCropApi, cropped_name};

    pub(super) struct WebImageCrop;

    pub(super) static IMAGE_CROP: WebImageCrop = WebImageCrop;

    impl ImageCropApi for WebImageCrop {
        fn crop(&self, file: FileData, rect: Fractions, max: Option<u32>) -> Cropped {
            Box::pin(async move {
                let bytes = file.read_bytes().await.ok()?;
                let from = file.content_type().unwrap_or_default();
                let script = Function::new_with_args(
                    "bytes, type, rect, max",
                    &format!(
                        "{CROP_SCRIPT}
                        return (async () => {{
                            const blob = await crop(bytes, type, rect, max);
                            return [blob.type, new Uint8Array(await blob.arrayBuffer())];
                        }})();"
                    ),
                );
                let rect: Array = rect.iter().map(|edge| JsValue::from_f64(*edge)).collect();
                let max = max.map_or(JsValue::NULL, JsValue::from);
                let args: Array = [
                    Uint8Array::from(bytes.as_ref()).into(),
                    JsValue::from_str(&from),
                    rect.into(),
                    max,
                ]
                .into_iter()
                .collect();
                let promise = script
                    .apply(&JsValue::NULL, &args)
                    .ok()?
                    .dyn_into::<Promise>()
                    .ok()?;
                let result: Array = JsFuture::from(promise).await.ok()?.dyn_into().ok()?;
                let to = result.get(0).as_string()?;
                let bytes = result.get(1).dyn_into::<Uint8Array>().ok()?;
                bytes_file(&cropped_name(&file.name(), &from, &to), &to, &bytes)
            })
        }
    }

    /// `bytes` as a `FileData`, backed by a browser `File` a form can post.
    pub(crate) fn bytes_file(
        name: &str,
        content_type: &str,
        bytes: &Uint8Array,
    ) -> Option<FileData> {
        let options = web_sys::FilePropertyBag::new();
        options.set_type(content_type);
        let parts: Array = [JsValue::from(bytes.clone())].into_iter().collect();
        let file =
            web_sys::File::new_with_u8_array_sequence_and_options(&parts, name, &options).ok()?;
        Some(FileData::new(CroppedFile {
            bytes: bytes.to_vec().into(),
            file,
        }))
    }

    /// A crop's bytes, and the browser `File` a form posts them as.
    struct CroppedFile {
        bytes: Bytes,
        file: web_sys::File,
    }

    // Wasm runs one thread; dioxus-web's own file type does the same.
    unsafe impl Send for CroppedFile {}
    unsafe impl Sync for CroppedFile {}

    type Chunk = Result<Bytes, dioxus::CapturedError>;

    struct Whole(Option<Chunk>);

    impl futures_core::Stream for Whole {
        type Item = Chunk;

        fn poll_next(
            mut self: Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Chunk>> {
            std::task::Poll::Ready(self.0.take())
        }
    }

    impl NativeFileData for CroppedFile {
        fn name(&self) -> String {
            self.file.name()
        }

        fn size(&self) -> u64 {
            self.bytes.len() as u64
        }

        fn last_modified(&self) -> u64 {
            self.file.last_modified() as u64
        }

        fn path(&self) -> std::path::PathBuf {
            self.file.name().into()
        }

        fn content_type(&self) -> Option<String> {
            Some(self.file.type_()).filter(|kind| !kind.is_empty())
        }

        fn read_bytes(&self) -> Pin<Box<dyn Future<Output = Chunk>>> {
            Box::pin(std::future::ready(Ok(self.bytes.clone())))
        }

        fn byte_stream(&self) -> Pin<Box<dyn futures_core::Stream<Item = Chunk> + Send>> {
            Box::pin(Whole(Some(Ok(self.bytes.clone()))))
        }

        fn read_string(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<String, dioxus::CapturedError>>>> {
            let text = String::from_utf8(self.bytes.to_vec()).map_err(dioxus::CapturedError::from);
            Box::pin(std::future::ready(text))
        }

        fn inner(&self) -> &dyn std::any::Any {
            &self.file
        }
    }
}

/// `None` where no page can crop: Blitz and a server render. Call it after mount.
pub(crate) fn image_crop() -> Option<&'static dyn ImageCropApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::IMAGE_CROP);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_image_crop();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
}

#[cfg(test)]
mod tests {
    use super::cropped_name;

    #[test]
    fn a_changed_type_swaps_the_extension() {
        assert_eq!(cropped_name("a.jpg", "image/jpeg", "image/jpeg"), "a.jpg");
        assert_eq!(cropped_name("a.bmp", "image/bmp", "image/png"), "a.png");
        assert_eq!(cropped_name("photo", "", "image/png"), "photo.png");
    }
}
