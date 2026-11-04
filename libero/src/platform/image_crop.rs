use std::{future::Future, pin::Pin};

use dioxus::html::{FileData, bytes::Bytes};

/// A crop as fractions of the image: `[x, y, width, height]`, 0-1 inside the picture and
/// past it for an overhang. Not the component's rect type: this layer sits below the components.
pub(crate) type Fractions = [f64; 4];

/// The cropped file, `None` when the image would not decode or encode.
pub(crate) type Cropped = Pin<Box<dyn Future<Output = Option<FileData>>>>;

/// Whether the picture has see-through pixels, `None` when it would not decode.
pub(crate) type Probed = Pin<Box<dyn Future<Output = Option<bool>>>>;

/// The picture's width and height, `None` when it would not decode.
pub(crate) type Measured = Pin<Box<dyn Future<Output = Option<(u32, u32)>>>>;

/// A URL's bytes and content type, `None` on any failure.
pub(crate) type Loaded = Pin<Box<dyn Future<Output = Option<(Bytes, String)>>>>;

/// The mask over the image outside the crop, and the shape [`crop_image`] keeps.
/// The crop itself stays a rectangle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CropShape {
    #[default]
    Rect,
    /// An ellipse inscribed in the box: a circle with a square `aspect`.
    Circle,
}

/// The file type [`crop_image`] and [`on_square`] write.
///
/// ```rust
/// # use libero::platform::image::ImageType;
/// assert_eq!(ImageType::default(), ImageType::Keep);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ImageType {
    /// A PNG, JPEG or WebP keeps its type, anything else becomes a PNG.
    #[default]
    Keep,
    Png,
    /// No transparency: what would be see-through comes out white or the backdrop's colour.
    Jpeg,
    Webp,
}

impl ImageType {
    fn mime(self) -> Option<&'static str> {
        match self {
            Self::Keep => None,
            Self::Png => Some("image/png"),
            Self::Jpeg => Some("image/jpeg"),
            Self::Webp => Some("image/webp"),
        }
    }
}

/// How [`crop_image`] and [`on_square`] cut, besides the rect.
///
/// ```rust
/// # use libero::platform::image::{Backdrop, CropShape, CutOptions, ImageType};
/// // A round 256 px avatar, transparent past the circle.
/// let avatar = CutOptions {
///     max_size: Some(256),
///     shape: CropShape::Circle,
///     output: ImageType::Png,
///     ..Default::default()
/// };
/// assert_eq!(avatar.backdrop, Backdrop::Checkerboard);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CutOptions {
    /// The longer side is scaled down to at most this, in px.
    pub max_size: Option<u32>,
    /// What fills past the picture's edge and, with a `Circle`, past the circle.
    pub backdrop: Backdrop,
    /// `Circle` fills everything outside the ellipse inscribed in the cut with `backdrop`.
    pub shape: CropShape,
    pub output: ImageType,
}

/// A picture to probe: a picked file or a URL.
///
/// A URL loads through the page's `fetch` on the web and in a WebView, so a cross-origin one
/// needs CORS; Blitz decodes a `data:` URL itself and fetches `http(s)` through its network
/// provider.
/// `javascript:`, `file:` and the like load nothing.
#[derive(Clone, Copy)]
pub enum ImageSource<'a> {
    File(&'a FileData),
    Url(&'a str),
}

impl<'a> From<&'a FileData> for ImageSource<'a> {
    fn from(file: &'a FileData) -> Self {
        Self::File(file)
    }
}

impl<'a> From<&'a str> for ImageSource<'a> {
    fn from(url: &'a str) -> Self {
        Self::Url(url)
    }
}

impl<'a> From<&'a String> for ImageSource<'a> {
    fn from(url: &'a String) -> Self {
        Self::Url(url)
    }
}

/// What shows past a picture's edge, in an `ImageCropper` with `overhang` and in the cut.
///
/// ```rust
/// # use libero::components::Backdrop;
/// assert_eq!(Backdrop::default(), Backdrop::Checkerboard);
/// assert_eq!(Backdrop::rgb(255, 255, 255), Backdrop::Color([255, 255, 255, 255]));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Backdrop {
    /// A checkerboard in the cropper; the cut stays transparent (a JPEG flattens onto white).
    #[default]
    Checkerboard,
    /// A flat colour, as red, green, blue and alpha bytes, in the cropper and the cut.
    Color([u8; 4]),
}

impl Backdrop {
    /// An opaque colour.
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::Color([red, green, blue, 255])
    }

    /// The colour the cut fills with: `None` leaves the canvas transparent.
    pub(crate) fn fill(self) -> Option<[u8; 4]> {
        match self {
            Self::Checkerboard => None,
            Self::Color(rgba) => Some(rgba),
        }
    }
}

/// How a cut is made, besides the rect.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Cut {
    /// The longer side is scaled down to at most this, in px.
    pub(crate) max: Option<u32>,
    /// Painted under the picture; `None` leaves the canvas transparent (a JPEG gets white).
    pub(crate) fill: Option<[u8; 4]>,
    /// Ignores the rect: the picture centred on a square with this margin per side,
    /// a fraction of its longer side.
    pub(crate) square: Option<f64>,
    /// Fills outside the ellipse inscribed in the cut as past the picture's edge.
    pub(crate) circle: bool,
    /// The type written; `None` keeps the input's.
    pub(crate) output: Option<&'static str>,
}

impl From<CutOptions> for Cut {
    fn from(options: CutOptions) -> Self {
        Self {
            max: options.max_size,
            fill: options.backdrop.fill(),
            square: None,
            circle: options.shape == CropShape::Circle,
            output: options.output.mime(),
        }
    }
}

impl Cut {
    /// The cut as the JSON object the scripts read.
    #[cfg(not(feature = "native"))]
    pub(crate) fn json(self) -> String {
        let number =
            |value: Option<f64>| value.map_or("null".to_string(), |value| value.to_string());
        let fill = self.fill.map_or("null".to_string(), |[r, g, b, a]| {
            format!("[{r}, {g}, {b}, {a}]")
        });
        let output = self
            .output
            .map_or("null".to_string(), |kind| format!("\"{kind}\""));
        format!(
            "{{\"max\": {}, \"fill\": {fill}, \"square\": {}, \"circle\": {}, \"output\": {output}}}",
            number(self.max.map(f64::from)),
            number(self.square.filter(|margin| margin.is_finite())),
            self.circle
        )
    }
}

/// Cuts an image file to a crop, through the page's canvas or, on Blitz, the `image` crate.
pub(crate) trait ImageCropApi {
    /// `file` cut to `rect` as `cut` says. Keeps a PNG, JPEG or WebP's type unless
    /// `cut.output` names one; anything else comes back a PNG.
    fn crop(&self, file: FileData, rect: Fractions, cut: Cut) -> Cropped;

    /// Whether any pixel of the picture is not fully opaque.
    fn has_transparency(&self, bytes: Bytes, kind: String) -> Probed;

    /// The picture's width and height, turned as its EXIF says.
    fn size(&self, bytes: Bytes, kind: String) -> Measured;

    /// `url`'s bytes and type; [`fetchable`] already passed it.
    fn fetch(&self, url: String) -> Loaded;
}

/// Whether `url` may be fetched for a probe: relative, or a scheme in `schemes`.
pub(crate) fn fetchable(url: &str, schemes: &[&str]) -> bool {
    // A URL parser drops leading spaces and controls before it reads the scheme.
    let url = url.trim_start_matches(|ch: char| ch <= ' ');
    let scheme = url
        .split_once(':')
        .map(|(scheme, _)| scheme)
        .filter(|scheme| {
            scheme.starts_with(|ch: char| ch.is_ascii_alphabetic())
                && scheme
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || "+-.".contains(ch))
        });
    match scheme {
        Some(scheme) => schemes.contains(&scheme.to_ascii_lowercase().as_str()),
        None => true,
    }
}

/// `Some(false)` when the header alone says no pixel can be see-through, `None` when only the
/// pixels can tell: a JPEG, a PNG without alpha or `tRNS`, a lossy WebP without alpha.
pub(crate) fn opaque_by_header(bytes: &[u8]) -> Option<bool> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(false);
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        // IHDR comes first; its colour type sits at byte 25: 4 and 6 carry alpha.
        if bytes.get(12..16) != Some(b"IHDR") || matches!(bytes.get(25)?, 4 | 6) {
            return None;
        }
        let mut at = 8;
        while let Some(head) = bytes.get(at..at + 8) {
            let length = u32::from_be_bytes([head[0], head[1], head[2], head[3]]) as usize;
            match &head[4..8] {
                b"tRNS" => return None,
                b"IDAT" | b"IEND" => return Some(false),
                _ => at = at.checked_add(12 + length)?,
            }
        }
        return None;
    }
    // Lossless and extended WebP may hold alpha; only the pixels say.
    if bytes.get(..4) == Some(b"RIFF") && bytes.get(8..16) == Some(b"WEBPVP8 ") {
        return Some(false);
    }
    None
}

/// `crop(bytes, type, rect, cut)` resolving to a `Blob`, `transparent(bytes, type)` to a
/// bool, `measure(bytes, type)` to `[width, height]` and `load(url)` to `[type, bytes]`, run by a
/// browser and a WebView alike. `createImageBitmap` turns the picture as its EXIF says, as
/// `<img>` does; an SVG goes through an `<img>` first.
#[cfg(not(feature = "native"))]
pub(crate) const IMAGE_SCRIPT: &str =
    "const decode = async (bytes, type) => {
    const blob = new Blob([bytes], { type });
    try {
        return await createImageBitmap(blob);
    } catch (error) {
        // Chrome's createImageBitmap refuses an SVG; an <img> decodes it, at its own size.
        const url = URL.createObjectURL(blob);
        try {
            const image = new Image();
            image.src = url;
            await image.decode();
            if (!image.naturalWidth || !image.naturalHeight) throw error;
            return await createImageBitmap(image);
        } finally {
            URL.revokeObjectURL(url);
        }
    }
};
const crop = async (bytes, type, [x, y, w, h], { max, fill, square, circle, output }) => {
    const bitmap = await decode(bytes, type);
    const [bw, bh] = [bitmap.width, bitmap.height];
    let sx, sy, sw, sh;
    if (square != null) {
        sw = sh = Math.max(1, Math.round(Math.max(bw, bh) * (1 + 2 * square)));
        sx = Math.round((bw - sw) / 2);
        sy = Math.round((bh - sh) / 2);
    } else {
        sx = Math.round(x * bw);
        sy = Math.round(y * bh);
        sw = Math.max(1, Math.round(w * bw));
        sh = Math.max(1, Math.round(h * bh));
        // A rect inside the picture stays inside it, whatever the rounding.
        if (x >= 0 && y >= 0 && x + w <= 1 + 1e-9 && y + h <= 1 + 1e-9) {
            sw = Math.min(sw, Math.max(1, bw - sx));
            sh = Math.min(sh, Math.max(1, bh - sy));
        }
    }
    const scale = max ? Math.min(1, max / Math.max(sw, sh)) : 1;
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(sw * scale));
    canvas.height = Math.max(1, Math.round(sh * scale));
    const out = output
        || (['image/png', 'image/jpeg', 'image/webp'].includes(type) ? type : 'image/png');
    const context = canvas.getContext('2d');
    const paint = fill || (out === 'image/jpeg' ? [255, 255, 255, 255] : null);
    const painted = () => {
        context.fillStyle = `rgba(${paint[0]}, ${paint[1]}, ${paint[2]}, ${paint[3] / 255})`;
        context.fillRect(0, 0, canvas.width, canvas.height);
    };
    if (paint && !circle) painted();
    if (sx >= 0 && sy >= 0 && sx + sw <= bw && sy + sh <= bh) {
        context.drawImage(bitmap, sx, sy, sw, sh, 0, 0, canvas.width, canvas.height);
    } else {
        context.save();
        context.scale(canvas.width / sw, canvas.height / sh);
        context.drawImage(bitmap, -sx, -sy);
        context.restore();
    }
    bitmap.close();
    if (circle) {
        // Keep the inscribed ellipse, then paint under what is left.
        context.globalCompositeOperation = 'destination-in';
        context.beginPath();
        context.ellipse(canvas.width / 2, canvas.height / 2, canvas.width / 2, canvas.height / 2, 0, 0, 2 * Math.PI);
        context.fill();
        context.globalCompositeOperation = 'destination-over';
        if (paint) painted();
    }
    return await new Promise((done, fail) =>
        canvas.toBlob((blob) => (blob ? done(blob) : fail()), out, 0.92));
};
const transparent = async (bytes, type) => {
    const bitmap = await decode(bytes, type);
    const [width, height] = [bitmap.width, bitmap.height];
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext('2d', { willReadFrequently: true });
    context.drawImage(bitmap, 0, 0);
    bitmap.close();
    // Strips of about a million pixels, so a see-through one ends the read early. Every browser
    // is little-endian: alpha is a word's top byte.
    const rows = Math.max(1, Math.floor(1048576 / width));
    for (let top = 0; top < height; top += rows) {
        const strip = context.getImageData(0, top, width, Math.min(rows, height - top)).data;
        const words = new Uint32Array(strip.buffer, strip.byteOffset, strip.length / 4);
        for (let at = 0; at < words.length; at++) {
            if (words[at] < 0xff000000) return true;
        }
    }
    return false;
};
const measure = async (bytes, type) => {
    const bitmap = await decode(bytes, type);
    const size = [bitmap.width, bitmap.height];
    bitmap.close();
    return size;
};
const load = async (url) => {
    const response = await fetch(url);
    if (!response.ok) throw new Error(response.status);
    const blob = await response.blob();
    return [blob.type, new Uint8Array(await blob.arrayBuffer())];
};";

/// Schemes the page's `fetch` may load for a probe; a relative URL always loads.
#[cfg(not(feature = "native"))]
pub(crate) const PAGE_SCHEMES: [&str; 4] = ["http", "https", "data", "blob"];

/// The file's name for its new type: the extension swapped when the type changed.
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

    use super::{
        Cropped, Cut, Fractions, IMAGE_SCRIPT, ImageCropApi, Loaded, Measured, Probed, cropped_name,
    };

    pub(super) struct WebImageCrop;

    pub(super) static IMAGE_CROP: WebImageCrop = WebImageCrop;

    impl ImageCropApi for WebImageCrop {
        fn crop(&self, file: FileData, rect: Fractions, cut: Cut) -> Cropped {
            Box::pin(async move {
                let bytes = file.read_bytes().await.ok()?;
                let from = file.content_type().unwrap_or_default();
                let script = Function::new_with_args(
                    "bytes, type, rect, cut",
                    &format!(
                        "{IMAGE_SCRIPT}
                        return (async () => {{
                            const blob = await crop(bytes, type, rect, cut);
                            return [blob.type, new Uint8Array(await blob.arrayBuffer())];
                        }})();"
                    ),
                );
                let rect: Array = rect.iter().map(|edge| JsValue::from_f64(*edge)).collect();
                let args: Array = [
                    Uint8Array::from(bytes.as_ref()).into(),
                    JsValue::from_str(&from),
                    rect.into(),
                    js_sys::JSON::parse(&cut.json()).ok()?,
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

        fn has_transparency(&self, bytes: Bytes, kind: String) -> Probed {
            Box::pin(async move { run_on_bytes("transparent", &bytes, &kind).await?.as_bool() })
        }

        fn size(&self, bytes: Bytes, kind: String) -> Measured {
            Box::pin(async move {
                let size: Array = run_on_bytes("measure", &bytes, &kind)
                    .await?
                    .dyn_into()
                    .ok()?;
                Some((size.get(0).as_f64()? as u32, size.get(1).as_f64()? as u32))
            })
        }

        fn fetch(&self, url: String) -> Loaded {
            Box::pin(async move {
                let script =
                    Function::new_with_args("url", &format!("{IMAGE_SCRIPT} return load(url);"));
                let promise = script
                    .call1(&JsValue::NULL, &JsValue::from_str(&url))
                    .ok()?
                    .dyn_into::<Promise>()
                    .ok()?;
                let result: Array = JsFuture::from(promise).await.ok()?.dyn_into().ok()?;
                let kind = result.get(0).as_string()?;
                let bytes = result.get(1).dyn_into::<Uint8Array>().ok()?;
                Some((bytes.to_vec().into(), kind))
            })
        }
    }

    /// `IMAGE_SCRIPT`'s `function(bytes, type)`, awaited.
    async fn run_on_bytes(function: &str, bytes: &[u8], kind: &str) -> Option<JsValue> {
        let script = Function::new_with_args(
            "bytes, type",
            &format!("{IMAGE_SCRIPT} return {function}(bytes, type);"),
        );
        let promise = script
            .call2(
                &JsValue::NULL,
                &Uint8Array::from(bytes).into(),
                &JsValue::from_str(kind),
            )
            .ok()?
            .dyn_into::<Promise>()
            .ok()?;
        JsFuture::from(promise).await.ok()
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

    /// `blob` as a `FileData` whose bytes stay in the page until read.
    pub(crate) fn blob_file(
        name: &str,
        content_type: &str,
        blob: &web_sys::Blob,
    ) -> Option<FileData> {
        let options = web_sys::FilePropertyBag::new();
        options.set_type(content_type);
        let parts: Array = [JsValue::from(blob.clone())].into_iter().collect();
        let file =
            web_sys::File::new_with_blob_sequence_and_options(&parts, name, &options).ok()?;
        Some(FileData::new(PageFile(file)))
    }

    /// A browser `File` read on demand; a long recording never sits in wasm memory.
    struct PageFile(web_sys::File);

    unsafe impl Send for PageFile {}
    unsafe impl Sync for PageFile {}

    impl PageFile {
        fn read(&self) -> Pin<Box<dyn Future<Output = Chunk>>> {
            let promise = self.0.array_buffer();
            Box::pin(async move {
                let buffer = JsFuture::from(promise)
                    .await
                    .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
                Ok(Uint8Array::new(&buffer).to_vec().into())
            })
        }
    }

    /// One chunk, the whole file, once read.
    struct Later(Option<Pin<Box<dyn Future<Output = Chunk>>>>);

    // Wasm runs one thread.
    unsafe impl Send for Later {}

    impl futures_core::Stream for Later {
        type Item = Chunk;

        fn poll_next(
            mut self: Pin<&mut Self>,
            context: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Chunk>> {
            let Some(read) = self.0.as_mut() else {
                return std::task::Poll::Ready(None);
            };
            let chunk = std::task::ready!(read.as_mut().poll(context));
            self.0 = None;
            std::task::Poll::Ready(Some(chunk))
        }
    }

    impl NativeFileData for PageFile {
        fn name(&self) -> String {
            self.0.name()
        }

        fn size(&self) -> u64 {
            self.0.size() as u64
        }

        fn last_modified(&self) -> u64 {
            self.0.last_modified() as u64
        }

        fn path(&self) -> std::path::PathBuf {
            self.0.name().into()
        }

        fn content_type(&self) -> Option<String> {
            Some(self.0.type_()).filter(|kind| !kind.is_empty())
        }

        fn read_bytes(&self) -> Pin<Box<dyn Future<Output = Chunk>>> {
            self.read()
        }

        fn byte_stream(&self) -> Pin<Box<dyn futures_core::Stream<Item = Chunk> + Send>> {
            Box::pin(Later(Some(self.read())))
        }

        fn read_string(
            &self,
        ) -> Pin<Box<dyn Future<Output = Result<String, dioxus::CapturedError>>>> {
            let read = self.read();
            Box::pin(async move {
                String::from_utf8(read.await?.to_vec()).map_err(dioxus::CapturedError::from)
            })
        }

        fn inner(&self) -> &dyn std::any::Any {
            &self.0
        }
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

/// Blitz has no canvas: decode, cut and encode in Rust (`native` only).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod native {
    use std::io::Cursor;

    use dioxus::html::{FileData, bytes::Bytes};
    use image::{
        DynamicImage, ImageDecoder, ImageFormat, ImageReader, Rgba, RgbaImage,
        codecs::jpeg::JpegEncoder, imageops::FilterType, metadata::Orientation,
    };

    use super::{Cropped, Cut, Fractions, ImageCropApi, Loaded, Measured, Probed, cropped_name};
    use crate::platform::file_dialog::held_file;

    pub(super) struct NativeImageCrop;

    pub(super) static IMAGE_CROP: NativeImageCrop = NativeImageCrop;

    impl ImageCropApi for NativeImageCrop {
        fn crop(&self, file: FileData, rect: Fractions, cut: Cut) -> Cropped {
            Box::pin(async move {
                let bytes = file.read_bytes().await.ok()?;
                let from = file.content_type().unwrap_or_default();
                let (to, out) = crop_bytes(&bytes, &from, rect, cut)?;
                let name = cropped_name(&file.name(), &from, to);
                Some(held_file(name, to.to_string(), file.last_modified(), out))
            })
        }

        fn has_transparency(&self, bytes: Bytes, _: String) -> Probed {
            Box::pin(std::future::ready(transparent_bytes(&bytes)))
        }

        fn size(&self, bytes: Bytes, _: String) -> Measured {
            Box::pin(std::future::ready(size_bytes(&bytes)))
        }

        /// A `data:` URL here, `http(s)` through Blitz's network provider, `None` without one.
        fn fetch(&self, url: String) -> Loaded {
            if let Some(data) = data_url_bytes(&url) {
                return Box::pin(std::future::ready(data));
            }
            use std::sync::Arc;

            use blitz_traits::net::{NetHandler, NetProvider, Request, Url};
            use futures_channel::oneshot;

            /// A failed fetch drops it unanswered, which cancels the channel.
            struct Reply(oneshot::Sender<(String, Bytes)>);

            impl NetHandler for Reply {
                fn bytes(self: Box<Self>, kind: String, bytes: Bytes) {
                    let _ = self.0.send((kind, bytes));
                }
            }

            let provider = dioxus::prelude::try_consume_context::<Arc<dyn NetProvider>>();
            let url = Url::parse(&url).ok();
            let (Some(provider), Some(url)) = (provider, url) else {
                return Box::pin(std::future::ready(None));
            };
            let (sender, receiver) = oneshot::channel();
            provider.fetch(0, Request::get(url), Box::new(Reply(sender)));
            Box::pin(async move {
                let (_, bytes) = receiver.await.ok()?;
                // The provider's first argument is not the content type: sniff it.
                let kind = image::guess_format(&bytes).ok()?.to_mime_type().to_string();
                Some((bytes, kind))
            })
        }
    }

    /// A `data:` URL's bytes and type, `None` inside when it does not decode; `None` for another URL.
    #[allow(clippy::option_option)]
    pub(super) fn data_url_bytes(url: &str) -> Option<Option<(Bytes, String)>> {
        let url = url.trim();
        let rest = url
            .get(..5)
            .filter(|head| head.eq_ignore_ascii_case("data:"))
            .map(|_| &url[5..])?;
        let Some((head, body)) = rest.split_once(',') else {
            return Some(None);
        };
        let (kind, base64) = match head.strip_suffix(";base64") {
            Some(kind) => (kind, true),
            None => (head, false),
        };
        let bytes = match base64 {
            true => decode_base64(body),
            false => percent_decode(body),
        };
        Some(bytes.map(|bytes| {
            let kind = kind.split(';').next().unwrap_or_default();
            (Bytes::from(bytes), kind.to_ascii_lowercase())
        }))
    }

    /// Standard base64, padded or not; spaces skipped as a URL may wrap them in.
    fn decode_base64(text: &str) -> Option<Vec<u8>> {
        let (mut bytes, mut buffer, mut bits) = (Vec::with_capacity(text.len() * 3 / 4), 0u32, 0);
        for ch in text
            .bytes()
            .filter(|ch| !ch.is_ascii_whitespace() && *ch != b'=')
        {
            let value = match ch {
                b'A'..=b'Z' => ch - b'A',
                b'a'..=b'z' => ch - b'a' + 26,
                b'0'..=b'9' => ch - b'0' + 52,
                b'+' | b'-' => 62,
                b'/' | b'_' => 63,
                _ => return None,
            };
            buffer = (buffer << 6) | u32::from(value);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                bytes.push((buffer >> bits) as u8);
            }
        }
        Some(bytes)
    }

    fn percent_decode(text: &str) -> Option<Vec<u8>> {
        let (mut bytes, mut rest) = (Vec::with_capacity(text.len()), text.as_bytes());
        while let Some((&first, tail)) = rest.split_first() {
            if first == b'%' {
                let hex = std::str::from_utf8(tail.get(..2)?).ok()?;
                bytes.push(u8::from_str_radix(hex, 16).ok()?);
                rest = &tail[2..];
            } else {
                bytes.push(first);
                rest = tail;
            }
        }
        Some(bytes)
    }

    pub(super) fn transparent_bytes(bytes: &[u8]) -> Option<bool> {
        let image = decode(bytes)?;
        // The common layouts read in place, stopping at the first see-through pixel.
        Some(match &image {
            _ if !image.color().has_alpha() => false,
            DynamicImage::ImageRgba8(pixels) => pixels.pixels().any(|pixel| pixel.0[3] < 255),
            DynamicImage::ImageLumaA8(pixels) => pixels.pixels().any(|pixel| pixel.0[1] < 255),
            DynamicImage::ImageRgba16(pixels) => pixels.pixels().any(|pixel| pixel.0[3] < u16::MAX),
            DynamicImage::ImageLumaA16(pixels) => {
                pixels.pixels().any(|pixel| pixel.0[1] < u16::MAX)
            }
            _ => image.to_rgba8().pixels().any(|pixel| pixel.0[3] < 255),
        })
    }

    /// The size without decoding the pixels, swapped for a quarter turn.
    pub(super) fn size_bytes(bytes: &[u8]) -> Option<(u32, u32)> {
        let mut decoder = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .ok()?
            .into_decoder()
            .ok()?;
        let (width, height) = decoder.dimensions();
        let turned = matches!(
            decoder.orientation().ok(),
            Some(
                Orientation::Rotate90
                    | Orientation::Rotate270
                    | Orientation::Rotate90FlipH
                    | Orientation::Rotate270FlipH
            )
        );
        (width > 0 && height > 0).then_some(match turned {
            true => (height, width),
            false => (width, height),
        })
    }

    /// The picture turned as its EXIF says, as `createImageBitmap` does.
    fn decode(bytes: &[u8]) -> Option<DynamicImage> {
        let mut decoder = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .ok()?
            .into_decoder()
            .ok()?;
        let orientation = decoder.orientation().ok();
        let mut image = DynamicImage::from_decoder(decoder).ok()?;
        if let Some(orientation) = orientation {
            image.apply_orientation(orientation);
        }
        (image.width() > 0 && image.height() > 0).then_some(image)
    }

    /// The canvas script's cut in Rust: same rounding, same kept types, JPEG at 0.92.
    pub(super) fn crop_bytes(
        bytes: &[u8],
        from: &str,
        [x, y, w, h]: Fractions,
        cut: Cut,
    ) -> Option<(&'static str, Vec<u8>)> {
        let image = decode(bytes)?;
        let (bw, bh) = (i64::from(image.width()), i64::from(image.height()));
        let (width, height) = (bw as f64, bh as f64);
        let (sx, sy, sw, sh) = match cut.square.filter(|margin| margin.is_finite()) {
            Some(margin) => {
                let side = ((bw.max(bh) as f64) * (1.0 + 2.0 * margin))
                    .round()
                    .max(1.0) as i64;
                (
                    ((bw - side) as f64 / 2.0).round() as i64,
                    ((bh - side) as f64 / 2.0).round() as i64,
                    side,
                    side,
                )
            }
            None => {
                let (sx, sy) = ((x * width).round() as i64, (y * height).round() as i64);
                let mut sw = ((w * width).round() as i64).max(1);
                let mut sh = ((h * height).round() as i64).max(1);
                // A rect inside the picture stays inside it, whatever the rounding.
                if x >= 0.0 && y >= 0.0 && x + w <= 1.0 + 1e-9 && y + h <= 1.0 + 1e-9 {
                    sw = sw.min((bw - sx).max(1));
                    sh = sh.min((bh - sy).max(1));
                }
                (sx, sy, sw, sh)
            }
        };
        let scale = cut
            .max
            .map_or(1.0, |max| (f64::from(max) / sw.max(sh) as f64).min(1.0));
        let to = cut.output.unwrap_or(from);
        let inside = sx >= 0 && sy >= 0 && sx + sw <= bw && sy + sh <= bh;
        let paint = cut.fill.or((to == "image/jpeg").then_some([255; 4]));
        // A circle paints past it after the scale, as the canvas does.
        let under = paint.filter(|_| !cut.circle);
        let mut cut_image = match inside && under.is_none() {
            true => image.crop_imm(sx as u32, sy as u32, sw as u32, sh as u32),
            false => {
                let mut canvas =
                    RgbaImage::from_pixel(sw as u32, sh as u32, Rgba(under.unwrap_or([0; 4])));
                image::imageops::overlay(&mut canvas, &image.to_rgba8(), -sx, -sy);
                DynamicImage::ImageRgba8(canvas)
            }
        };
        if scale < 1.0 {
            let fit = |side: i64| ((side as f64 * scale).round() as u32).max(1);
            cut_image = cut_image.resize_exact(fit(sw), fit(sh), FilterType::Triangle);
        }
        if cut.circle {
            let mut pixels = cut_image.to_rgba8();
            let (width, height) = (f64::from(pixels.width()), f64::from(pixels.height()));
            for (x, y, pixel) in pixels.enumerate_pixels_mut() {
                let dx = (f64::from(x) + 0.5) / width * 2.0 - 1.0;
                let dy = (f64::from(y) + 0.5) / height * 2.0 - 1.0;
                if dx * dx + dy * dy > 1.0 {
                    *pixel = Rgba(paint.unwrap_or([0; 4]));
                } else if let Some(under) = paint.filter(|_| pixel.0[3] < 255) {
                    let mut blended = Rgba(under);
                    image::Pixel::blend(&mut blended, pixel);
                    *pixel = blended;
                }
            }
            cut_image = DynamicImage::ImageRgba8(pixels);
        }
        let cut = cut_image;
        let mut out = Cursor::new(Vec::new());
        let to = match to {
            "image/jpeg" => {
                let encoder = JpegEncoder::new_with_quality(&mut out, 92);
                DynamicImage::ImageRgb8(cut.to_rgb8())
                    .write_with_encoder(encoder)
                    .ok()?;
                "image/jpeg"
            }
            "image/webp" => {
                cut.to_rgba8().write_to(&mut out, ImageFormat::WebP).ok()?;
                "image/webp"
            }
            _ => {
                cut.write_to(&mut out, ImageFormat::Png).ok()?;
                "image/png"
            }
        };
        Some((to, out.into_inner()))
    }

    #[cfg(test)]
    mod tests {
        use std::io::Cursor;

        use image::{ImageFormat, Rgba, RgbaImage};

        use super::{crop_bytes, data_url_bytes, size_bytes, transparent_bytes};
        use crate::platform::image_crop::Cut;

        fn png(width: u32, height: u32) -> Vec<u8> {
            let mut out = Cursor::new(Vec::new());
            RgbaImage::new(width, height)
                .write_to(&mut out, ImageFormat::Png)
                .unwrap();
            out.into_inner()
        }

        fn opaque(width: u32, height: u32) -> Vec<u8> {
            let mut out = Cursor::new(Vec::new());
            RgbaImage::from_pixel(width, height, Rgba([200, 0, 0, 255]))
                .write_to(&mut out, ImageFormat::Png)
                .unwrap();
            out.into_inner()
        }

        fn size(bytes: &[u8]) -> (u32, u32) {
            let image = image::load_from_memory(bytes).unwrap();
            (image.width(), image.height())
        }

        fn pixel(bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
            image::load_from_memory(bytes)
                .unwrap()
                .to_rgba8()
                .get_pixel(x, y)
                .0
        }

        fn max(max: u32) -> Cut {
            Cut {
                max: Some(max),
                ..Cut::default()
            }
        }

        #[test]
        fn a_crop_cuts_the_fraction_and_keeps_the_type() {
            let (to, out) = crop_bytes(
                &png(200, 100),
                "image/png",
                [0.5, 0.0, 0.5, 0.5],
                Cut::default(),
            )
            .unwrap();
            assert_eq!(to, "image/png");
            assert_eq!(size(&out), (100, 50));
        }

        #[test]
        fn max_scales_the_longer_side_down() {
            let (_, out) =
                crop_bytes(&png(200, 100), "image/png", [0.0, 0.0, 1.0, 1.0], max(16)).unwrap();
            assert_eq!(size(&out), (16, 8));
        }

        #[test]
        fn a_jpeg_stays_a_jpeg_and_other_types_become_png() {
            let whole = [0.0, 0.0, 1.0, 1.0];
            let (to, out) = crop_bytes(&png(10, 10), "image/jpeg", whole, Cut::default()).unwrap();
            assert_eq!(to, "image/jpeg");
            assert_eq!(image::guess_format(&out).unwrap(), ImageFormat::Jpeg);
            let (to, _) = crop_bytes(&png(10, 10), "image/bmp", whole, Cut::default()).unwrap();
            assert_eq!(to, "image/png");
        }

        #[test]
        fn undecodable_bytes_crop_to_nothing() {
            let whole = [0.0, 0.0, 1.0, 1.0];
            assert!(crop_bytes(b"not an image", "image/png", whole, Cut::default()).is_none());
        }

        #[test]
        fn a_rect_past_the_edge_grows_the_canvas_and_leaves_it_transparent() {
            let (_, out) = crop_bytes(
                &opaque(100, 100),
                "image/png",
                [-0.25, -0.25, 1.5, 1.5],
                Cut::default(),
            )
            .unwrap();
            assert_eq!(size(&out), (150, 150));
            assert_eq!(pixel(&out, 0, 0)[3], 0);
            assert_eq!(pixel(&out, 75, 75), [200, 0, 0, 255]);
        }

        #[test]
        fn a_backdrop_colour_fills_past_the_edge_and_a_jpeg_gets_white() {
            let rect = [-0.5, 0.0, 2.0, 1.0];
            let blue = Cut {
                fill: Some([0, 0, 255, 255]),
                ..Cut::default()
            };
            let (_, out) = crop_bytes(&opaque(100, 100), "image/png", rect, blue).unwrap();
            assert_eq!(size(&out), (200, 100));
            assert_eq!(pixel(&out, 10, 50), [0, 0, 255, 255]);
            let (_, out) =
                crop_bytes(&opaque(100, 100), "image/jpeg", rect, Cut::default()).unwrap();
            let corner = pixel(&out, 10, 50);
            assert!(
                corner[..3].iter().all(|channel| *channel > 240),
                "{corner:?}"
            );
        }

        #[test]
        fn on_a_square_pads_the_shorter_side_and_the_margin() {
            let square = Cut {
                square: Some(0.1),
                ..Cut::default()
            };
            let (_, out) = crop_bytes(&opaque(200, 100), "image/png", [0.0; 4], square).unwrap();
            assert_eq!(size(&out), (240, 240));
            assert_eq!(pixel(&out, 120, 120), [200, 0, 0, 255]);
            assert_eq!(pixel(&out, 120, 20)[3], 0);
        }

        #[test]
        fn transparency_is_any_pixel_not_opaque() {
            assert_eq!(transparent_bytes(&png(4, 4)), Some(true));
            assert_eq!(transparent_bytes(&opaque(4, 4)), Some(false));
            assert_eq!(transparent_bytes(b"not an image"), None);
        }

        #[test]
        fn a_circle_fills_past_it_and_keeps_its_middle() {
            let circle = |fill| Cut {
                circle: true,
                fill,
                ..Cut::default()
            };
            let whole = [0.0, 0.0, 1.0, 1.0];
            let (_, out) = crop_bytes(&opaque(100, 100), "image/png", whole, circle(None)).unwrap();
            assert_eq!(pixel(&out, 0, 0)[3], 0);
            assert_eq!(pixel(&out, 50, 50), [200, 0, 0, 255]);
            let white = Some([255; 4]);
            let (_, out) = crop_bytes(&png(100, 100), "image/png", whole, circle(white)).unwrap();
            assert_eq!(pixel(&out, 0, 0), [255; 4]);
            // The see-through middle gets the backdrop under it, as the canvas paints it.
            assert_eq!(pixel(&out, 50, 50), [255; 4]);
        }

        #[test]
        fn an_output_type_overrides_the_input_and_a_jpeg_gets_white_under_alpha() {
            let jpeg = Cut {
                output: Some("image/jpeg"),
                ..Cut::default()
            };
            let (to, out) =
                crop_bytes(&png(10, 10), "image/png", [0.0, 0.0, 1.0, 1.0], jpeg).unwrap();
            assert_eq!(to, "image/jpeg");
            assert_eq!(image::guess_format(&out).unwrap(), ImageFormat::Jpeg);
            let middle = pixel(&out, 5, 5);
            assert!(
                middle[..3].iter().all(|channel| *channel > 240),
                "{middle:?}"
            );
            let png_out = Cut {
                output: Some("image/png"),
                ..Cut::default()
            };
            let (to, _) =
                crop_bytes(&opaque(10, 10), "image/jpeg", [0.0, 0.0, 1.0, 1.0], png_out).unwrap();
            assert_eq!(to, "image/png");
        }

        #[test]
        fn a_data_url_decodes_without_a_network() {
            let (bytes, kind) = data_url_bytes("data:image/png;base64,iVBORw0K")
                .unwrap()
                .unwrap();
            assert_eq!((&bytes[..4], kind.as_str()), (&b"\x89PNG"[..], "image/png"));
            let (bytes, _) = data_url_bytes("DATA:text/plain,a%20b").unwrap().unwrap();
            assert_eq!(&bytes[..], b"a b");
            assert_eq!(data_url_bytes("data:image/png;base64,!!"), Some(None));
            assert_eq!(data_url_bytes("https://a.b/c.png"), None);
        }

        #[test]
        fn the_size_reads_the_header() {
            assert_eq!(size_bytes(&png(7, 3)), Some((7, 3)));
            assert_eq!(size_bytes(b"not an image"), None);
        }
    }
}

/// `file` cut to `rect` (fractions of the picture, 0-1 inside it and past it for room round it)
/// as `options` say: the longest side, what fills past the picture or a circle, the type written.
/// `None` where no page can cut (a server render, so call it from an event or effect)
/// or the picture would not decode.
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::components::CropRect;
/// # use libero::platform::image::{Backdrop, CutOptions, crop_image};
/// # async fn demo(file: FileData) {
/// let rect = CropRect { x: -0.25, y: -0.25, width: 1.5, height: 1.5 };
/// let options = CutOptions {
///     max_size: Some(512),
///     backdrop: Backdrop::rgb(255, 255, 255),
///     ..Default::default()
/// };
/// let padded = crop_image(&file, rect, options).await;
/// # }
/// ```
pub async fn crop_image(
    file: &FileData,
    rect: impl Into<[f64; 4]>,
    options: CutOptions,
) -> Option<FileData> {
    image_crop()?
        .crop(file.clone(), rect.into(), options.into())
        .await
}

/// `file` centred on a square canvas, `margin` of its longer side free on every side
/// (`0.1` is 10%), cut as [`crop_image`] cuts. `None` where no page can cut or the picture
/// would not decode.
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::platform::image::{CropShape, CutOptions, ImageType, on_square};
/// # async fn demo(file: FileData) {
/// // A logo on a round, see-through avatar.
/// let options = CutOptions { shape: CropShape::Circle, output: ImageType::Png, ..Default::default() };
/// let square = on_square(&file, 0.1, options).await;
/// # }
/// ```
pub async fn on_square(file: &FileData, margin: f64, options: CutOptions) -> Option<FileData> {
    let cut = Cut {
        square: Some(margin.max(0.0)),
        ..options.into()
    };
    image_crop()?.crop(file.clone(), [0.0; 4], cut).await
}

/// Whether any pixel of a file or URL is not fully opaque, `None` where no page can decode it,
/// the URL would not load (see [`ImageSource`]) or the picture would not decode. A JPEG, or a
/// PNG or lossy WebP without alpha, answers from its header.
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::platform::image::has_transparency;
/// # async fn demo(file: FileData) {
/// if has_transparency(&file).await == Some(true) {
///     // Put it on a backdrop instead of a flat fill.
/// }
/// let logo = has_transparency("/logo.png").await;
/// # }
/// ```
pub async fn has_transparency<'a>(source: impl Into<ImageSource<'a>>) -> Option<bool> {
    let api = image_crop()?;
    let (bytes, kind) = load(api, source.into()).await?;
    if let Some(opaque) = opaque_by_header(&bytes) {
        return Some(opaque);
    }
    api.has_transparency(bytes, kind).await
}

/// A file's or URL's width and height in px, turned as its EXIF says; `None` as in
/// [`has_transparency`].
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::platform::image::image_size;
/// # async fn demo(file: FileData) {
/// if let Some((width, height)) = image_size(&file).await {
///     let square = width == height;
/// }
/// # }
/// ```
pub async fn image_size<'a>(source: impl Into<ImageSource<'a>>) -> Option<(u32, u32)> {
    let api = image_crop()?;
    let (bytes, kind) = load(api, source.into()).await?;
    api.size(bytes, kind).await
}

/// The source's bytes and type; a URL with a refused scheme loads nothing.
async fn load(api: &dyn ImageCropApi, source: ImageSource<'_>) -> Option<(Bytes, String)> {
    match source {
        ImageSource::File(file) => Some((
            file.read_bytes().await.ok()?,
            file.content_type().unwrap_or_default(),
        )),
        ImageSource::Url(url) => {
            #[cfg(not(feature = "native"))]
            let schemes = PAGE_SCHEMES.as_slice();
            #[cfg(feature = "native")]
            let schemes = ["http", "https", "data"].as_slice();
            if !fetchable(url, schemes) {
                crate::utils::warn(&format!(
                    "Not loaded: `{url}` is not a URL a picture may come from."
                ));
                return None;
            }
            api.fetch(url.to_string()).await
        }
    }
}

/// `None` where no page can crop: a server render. Call it after mount.
pub(crate) fn image_crop() -> Option<&'static dyn ImageCropApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::IMAGE_CROP);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_image_crop();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return Some(&native::IMAGE_CROP);
}

#[cfg(test)]
mod tests {
    use super::{cropped_name, fetchable, opaque_by_header};

    /// A PNG's chunks up to its first IDAT, CRCs zeroed: the sniff reads no further.
    fn png(colour: u8, transparency: bool) -> Vec<u8> {
        let chunk = |name: &[u8], data: &[u8]| {
            let mut out = (data.len() as u32).to_be_bytes().to_vec();
            out.extend_from_slice(name);
            out.extend_from_slice(data);
            out.extend_from_slice(&[0; 4]);
            out
        };
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        out.extend(chunk(
            b"IHDR",
            &[0, 0, 0, 2, 0, 0, 0, 1, 8, colour, 0, 0, 0],
        ));
        if transparency {
            out.extend(chunk(b"tRNS", &[0, 0, 0, 0, 0, 0]));
        }
        out.extend(chunk(b"IDAT", &[0; 8]));
        out
    }

    #[test]
    fn the_header_rules_out_alpha_only_where_none_can_be() {
        assert_eq!(opaque_by_header(&png(2, false)), Some(false));
        assert_eq!(opaque_by_header(&png(3, false)), Some(false));
        assert_eq!(opaque_by_header(&png(2, true)), None);
        assert_eq!(opaque_by_header(&png(6, false)), None);
        assert_eq!(opaque_by_header(&png(4, false)), None);
        assert_eq!(
            opaque_by_header(&[0xFF, 0xD8, 0xFF, 0xE0, 0, 0]),
            Some(false)
        );
        let webp = |kind: &[u8]| [b"RIFF\0\0\0\0WEBP".as_slice(), kind, &[0; 8]].concat();
        assert_eq!(opaque_by_header(&webp(b"VP8 ")), Some(false));
        assert_eq!(opaque_by_header(&webp(b"VP8L")), None);
        assert_eq!(opaque_by_header(&webp(b"VP8X")), None);
        assert_eq!(opaque_by_header(b"GIF89a"), None);
        assert_eq!(opaque_by_header(b""), None);
    }

    #[test]
    fn only_listed_schemes_and_relative_urls_load() {
        let schemes = ["http", "https", "data"];
        for url in [
            "/logo.png",
            "logo.png",
            "HTTPS://a.b/c.png",
            "data:image/png;base64,AA",
        ] {
            assert!(fetchable(url, &schemes), "{url}");
        }
        for url in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "blob:x",
            " vbscript:x",
        ] {
            assert!(!fetchable(url, &schemes), "{url}");
        }
    }

    #[test]
    fn a_changed_type_swaps_the_extension() {
        assert_eq!(cropped_name("a.jpg", "image/jpeg", "image/jpeg"), "a.jpg");
        assert_eq!(cropped_name("a.bmp", "image/bmp", "image/png"), "a.png");
        assert_eq!(cropped_name("photo", "", "image/png"), "photo.png");
    }
}
