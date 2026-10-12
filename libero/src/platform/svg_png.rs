//! An SVG drawn into a PNG by the page's canvas: a chart's export.

use std::{future::Future, pin::Pin};

pub(crate) type Rasterized = Pin<Box<dyn Future<Output = Option<Vec<u8>>>>>;

/// `raster(svg, width, height, scale)` resolving to a PNG `Blob`, run by a browser and a
/// WebView alike. The SVG loads as an image, so it must be standalone (`xmlns`, sizes).
#[cfg(not(feature = "native"))]
pub(crate) const SVG_PNG_SCRIPT: &str = "const raster = async (svg, width, height, scale) => {
    const image = new Image();
    image.src = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svg);
    await Promise.race([image.decode(), new Promise((_, fail) => setTimeout(fail, 5000))]);
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(width * scale));
    canvas.height = Math.max(1, Math.round(height * scale));
    canvas.getContext('2d').drawImage(image, 0, 0, canvas.width, canvas.height);
    return await new Promise((done, fail) =>
        canvas.toBlob((blob) => (blob ? done(blob) : fail()), 'image/png'));
};";

#[cfg(target_arch = "wasm32")]
fn web(svg: String, width: f32, height: f32, scale: f32) -> Rasterized {
    use js_sys::{Array, Function, Promise, Uint8Array};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    Box::pin(async move {
        let script = Function::new_with_args(
            "svg, width, height, scale",
            &format!(
                "{SVG_PNG_SCRIPT}
                return (async () => {{
                    const blob = await raster(svg, width, height, scale);
                    return new Uint8Array(await blob.arrayBuffer());
                }})();"
            ),
        );
        let args: Array = [
            JsValue::from_str(&svg),
            JsValue::from_f64(f64::from(width)),
            JsValue::from_f64(f64::from(height)),
            JsValue::from_f64(f64::from(scale)),
        ]
        .into_iter()
        .collect();
        let promise = script
            .apply(&JsValue::NULL, &args)
            .ok()?
            .dyn_into::<Promise>()
            .ok()?;
        let bytes: Uint8Array = JsFuture::from(promise).await.ok()?.dyn_into().ok()?;
        Some(bytes.to_vec())
    })
}

/// `svg`, `width` by `height` px, as PNG bytes at `scale` times its size. `None` where no
/// page draws it (a server render, Blitz) or the page fails to.
pub(crate) fn svg_png(svg: String, width: f32, height: f32, scale: f32) -> Rasterized {
    #[cfg(target_arch = "wasm32")]
    return web(svg, width, height, scale);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_svg_png(svg, width, height, scale);
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        let _ = (svg, width, height, scale);
        Box::pin(std::future::ready(None))
    }
}
