use serde_json::json;

use super::{decode_base64, eval_with};

/// The image at `src` fitted into a transparent `size` px square PNG, drawn by the page:
/// only the WebView reaches wry's asset URLs, and it decodes SVG. `None` if it fails, or if
/// `src` has a scheme other than http(s) and data: the load carries the WebView's cookies.
#[cfg(target_os = "android")]
pub(crate) fn square_png(src: &str, size: u32) -> impl Future<Output = Option<Vec<u8>>> + use<> {
    let script = eval_with(
        json!([src, size]),
        "const [src, size] = data;
        try {
            const scheme = src.replace(/[\\t\\n\\r]/g, '').trim().match(/^([a-z][a-z0-9+.-]*):/i);
            if (scheme && !['http', 'https', 'data'].includes(scheme[1].toLowerCase())) return null;
            const image = new Image();
            image.src = src;
            await Promise.race([image.decode(), new Promise((_, fail) => setTimeout(fail, 5000))]);
            const width = image.naturalWidth || size, height = image.naturalHeight || size;
            const scale = size / Math.max(width, height);
            const canvas = document.createElement('canvas');
            canvas.width = canvas.height = size;
            const [w, h] = [width * scale, height * scale];
            canvas.getContext('2d').drawImage(image, (size - w) / 2, (size - h) / 2, w, h);
            return canvas.toDataURL('image/png').split(',')[1] ?? null;
        } catch (error) {
            return null;
        }",
    );
    async move { decode_base64(&script.join::<Option<String>>().await.ok()??) }
}

/// [`svg_png`](crate::platform::svg_png) in the WebView, the PNG back as base64.
#[cfg(feature = "charts")]
pub(crate) fn svg_png(
    svg: String,
    width: f32,
    height: f32,
    scale: f32,
) -> crate::platform::svg_png::Rasterized {
    use crate::platform::svg_png::SVG_PNG_SCRIPT;

    let script = eval_with(
        json!([svg, width, height, scale]),
        &format!(
            "{SVG_PNG_SCRIPT}
            const [svg, width, height, scale] = data;
            try {{
                const blob = await raster(svg, width, height, scale);
                const out = new Uint8Array(await blob.arrayBuffer());
                let binary = '';
                for (let at = 0; at < out.length; at += 0x8000) {{
                    binary += String.fromCharCode(...out.subarray(at, at + 0x8000));
                }}
                return btoa(binary);
            }} catch (error) {{
                return null;
            }}"
        ),
    );
    Box::pin(async move { decode_base64(&script.join::<Option<String>>().await.ok()??) })
}
