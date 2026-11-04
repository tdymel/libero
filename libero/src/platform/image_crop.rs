use std::{future::Future, pin::Pin};

use dioxus::html::FileData;

/// A crop as fractions of the image: `[x, y, width, height]`, 0-1 inside the picture and
/// past it for an overhang. Not the component's rect type: this layer sits below the components.
pub(crate) type Fractions = [f64; 4];

/// The cropped file, `None` when the image would not decode or encode.
pub(crate) type Cropped = Pin<Box<dyn Future<Output = Option<FileData>>>>;

/// Whether the picture has see-through pixels, `None` when it would not decode.
pub(crate) type Probed = Pin<Box<dyn Future<Output = Option<bool>>>>;

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
        format!(
            "{{\"max\": {}, \"fill\": {fill}, \"square\": {}}}",
            number(self.max.map(f64::from)),
            number(self.square.filter(|margin| margin.is_finite()))
        )
    }
}

/// Cuts an image file to a crop, through the page's canvas or, on Blitz, the `image` crate.
pub(crate) trait ImageCropApi {
    /// `file` cut to `rect` as `cut` says. Keeps a PNG, JPEG or WebP's type;
    /// anything else comes back a PNG.
    fn crop(&self, file: FileData, rect: Fractions, cut: Cut) -> Cropped;

    /// Whether any pixel of `file` is not fully opaque.
    fn has_transparency(&self, file: FileData) -> Probed;
}

/// `crop(bytes, type, rect, cut)` resolving to a `Blob`, and `transparent(bytes, type)` to a
/// bool, run by a browser and a WebView alike. `createImageBitmap` turns the picture as its
/// EXIF says, as `<img>` does.
#[cfg(not(feature = "native"))]
pub(crate) const IMAGE_SCRIPT: &str =
    "const crop = async (bytes, type, [x, y, w, h], { max, fill, square }) => {
    const bitmap = await createImageBitmap(new Blob([bytes], { type }));
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
    const out = ['image/png', 'image/jpeg', 'image/webp'].includes(type) ? type : 'image/png';
    const context = canvas.getContext('2d');
    const paint = fill || (out === 'image/jpeg' ? [255, 255, 255, 255] : null);
    if (paint) {
        context.fillStyle = `rgba(${paint[0]}, ${paint[1]}, ${paint[2]}, ${paint[3] / 255})`;
        context.fillRect(0, 0, canvas.width, canvas.height);
    }
    if (sx >= 0 && sy >= 0 && sx + sw <= bw && sy + sh <= bh) {
        context.drawImage(bitmap, sx, sy, sw, sh, 0, 0, canvas.width, canvas.height);
    } else {
        context.scale(canvas.width / sw, canvas.height / sh);
        context.drawImage(bitmap, -sx, -sy);
    }
    bitmap.close();
    return await new Promise((done, fail) =>
        canvas.toBlob((blob) => (blob ? done(blob) : fail()), out, 0.92));
};
const transparent = async (bytes, type) => {
    const bitmap = await createImageBitmap(new Blob([bytes], { type }));
    const canvas = document.createElement('canvas');
    canvas.width = bitmap.width;
    canvas.height = bitmap.height;
    const context = canvas.getContext('2d', { willReadFrequently: true });
    context.drawImage(bitmap, 0, 0);
    bitmap.close();
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height).data;
    for (let at = 3; at < pixels.length; at += 4) {
        if (pixels[at] < 255) return true;
    }
    return false;
};";

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

    use super::{Cropped, Cut, Fractions, IMAGE_SCRIPT, ImageCropApi, Probed, cropped_name};

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

        fn has_transparency(&self, file: FileData) -> Probed {
            Box::pin(async move {
                let bytes = file.read_bytes().await.ok()?;
                let from = file.content_type().unwrap_or_default();
                let script = Function::new_with_args(
                    "bytes, type",
                    &format!("{IMAGE_SCRIPT} return transparent(bytes, type);"),
                );
                let args: Array = [
                    Uint8Array::from(bytes.as_ref()).into(),
                    JsValue::from_str(&from),
                ]
                .into_iter()
                .collect();
                let promise = script
                    .apply(&JsValue::NULL, &args)
                    .ok()?
                    .dyn_into::<Promise>()
                    .ok()?;
                JsFuture::from(promise).await.ok()?.as_bool()
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

    use dioxus::html::FileData;
    use image::{
        DynamicImage, ImageDecoder, ImageFormat, ImageReader, Rgba, RgbaImage,
        codecs::jpeg::JpegEncoder, imageops::FilterType,
    };

    use super::{Cropped, Cut, Fractions, ImageCropApi, Probed, cropped_name};
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

        fn has_transparency(&self, file: FileData) -> Probed {
            Box::pin(async move {
                let bytes = file.read_bytes().await.ok()?;
                transparent_bytes(&bytes)
            })
        }
    }

    pub(super) fn transparent_bytes(bytes: &[u8]) -> Option<bool> {
        let image = decode(bytes)?;
        Some(image.color().has_alpha() && image.to_rgba8().pixels().any(|pixel| pixel.0[3] < 255))
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
        let inside = sx >= 0 && sy >= 0 && sx + sw <= bw && sy + sh <= bh;
        let mut cut_image = match inside && cut.fill.is_none() {
            true => image.crop_imm(sx as u32, sy as u32, sw as u32, sh as u32),
            false => {
                let paint = cut.fill.or((from == "image/jpeg").then_some([255; 4]));
                let mut canvas =
                    RgbaImage::from_pixel(sw as u32, sh as u32, Rgba(paint.unwrap_or([0; 4])));
                image::imageops::overlay(&mut canvas, &image.to_rgba8(), -sx, -sy);
                DynamicImage::ImageRgba8(canvas)
            }
        };
        if scale < 1.0 {
            let fit = |side: i64| ((side as f64 * scale).round() as u32).max(1);
            cut_image = cut_image.resize_exact(fit(sw), fit(sh), FilterType::Triangle);
        }
        let cut = cut_image;
        let mut out = Cursor::new(Vec::new());
        let to = match from {
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

        use super::{crop_bytes, transparent_bytes};
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
    }
}

/// `file` cut to `rect` (fractions of the picture, 0-1 inside it and past it for room round it).
/// Keeps a PNG, JPEG or WebP's type, anything else comes back a PNG; `max_size` scales the
/// longer side down to at most that many px; `backdrop` fills what lies past the picture.
/// `None` where no page can cut (a server render, so call it from an event or effect)
/// or the picture would not decode.
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::components::CropRect;
/// # use libero::platform::image::{Backdrop, crop_image};
/// # async fn demo(file: FileData) {
/// let rect = CropRect { x: -0.25, y: -0.25, width: 1.5, height: 1.5 };
/// let padded = crop_image(&file, rect, Some(512), Backdrop::rgb(255, 255, 255)).await;
/// # }
/// ```
pub async fn crop_image(
    file: &FileData,
    rect: impl Into<[f64; 4]>,
    max_size: Option<u32>,
    backdrop: Backdrop,
) -> Option<FileData> {
    let cut = Cut {
        max: max_size,
        fill: backdrop.fill(),
        square: None,
    };
    image_crop()?.crop(file.clone(), rect.into(), cut).await
}

/// `file` centred on a square canvas, `margin` of its longer side free on every side
/// (`0.1` is 10%), the rest filled by `backdrop`. The file's type is kept as in [`crop_image`].
/// `None` where no page can cut or the picture would not decode.
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::platform::image::{Backdrop, on_square};
/// # async fn demo(file: FileData) {
/// let square = on_square(&file, 0.1, Backdrop::default()).await;
/// # }
/// ```
pub async fn on_square(file: &FileData, margin: f64, backdrop: Backdrop) -> Option<FileData> {
    let cut = Cut {
        max: None,
        fill: backdrop.fill(),
        square: Some(margin.max(0.0)),
    };
    image_crop()?.crop(file.clone(), [0.0; 4], cut).await
}

/// Whether any pixel of `file` is not fully opaque, `None` where no page can decode it
/// or the picture would not decode.
///
/// ```rust,no_run
/// # use dioxus::html::FileData;
/// # use libero::platform::image::has_transparency;
/// # async fn demo(file: FileData) {
/// if has_transparency(&file).await == Some(true) {
///     // Put it on a backdrop instead of a flat fill.
/// }
/// # }
/// ```
pub async fn has_transparency(file: &FileData) -> Option<bool> {
    image_crop()?.has_transparency(file.clone()).await
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
    use super::cropped_name;

    #[test]
    fn a_changed_type_swaps_the_extension() {
        assert_eq!(cropped_name("a.jpg", "image/jpeg", "image/jpeg"), "a.jpg");
        assert_eq!(cropped_name("a.bmp", "image/bmp", "image/png"), "a.png");
        assert_eq!(cropped_name("photo", "", "image/png"), "photo.png");
    }
}
