use std::{future::Future, pin::Pin};

/// A file's loudness per bar, `None` when it would not fetch or decode.
pub(crate) type Peaks = Pin<Box<dyn Future<Output = Option<Vec<f64>>>>>;

/// Decodes an audio file through the page's Web Audio, for a waveform.
pub(crate) trait WaveformApi {
    /// The RMS loudness of `src` in `bars` equal slices, unscaled.
    fn peaks(&self, src: &str, bars: usize) -> Peaks;
}

/// `peaks(src, bars)`, run by a browser and a WebView alike. An offline context
/// decodes without a user gesture and opens no output device; at 8 kHz, as bars
/// need no treble, the decoded samples take a fifth of the memory. A file over
/// 20 MB is not read: `null`, so the drawn bars stay. Bytes are counted too, as
/// a response may have no Content-Length. Chromium can hand the fetch the media
/// element's cache entry, cut off where the element stops loading (todo 2130): a
/// body short of its Content-Length is fetched again past the cache.
#[cfg(not(feature = "native"))]
pub(crate) const PEAKS_SCRIPT: &str = "const load = async (src, cache) => {
    const response = await fetch(src, { cache });
    if (!response.ok) return null;
    const expected = Number(response.headers.get('content-length'));
    if (expected > 20e6) {
        response.body?.cancel();
        return null;
    }
    const reader = response.body.getReader();
    const chunks = [];
    let size = 0;
    for (let read = await reader.read(); !read.done; read = await reader.read()) {
        size += read.value.length;
        if (size > 20e6) {
            reader.cancel();
            return null;
        }
        chunks.push(read.value);
    }
    const bytes = new Uint8Array(size);
    chunks.reduce((at, chunk) => (bytes.set(chunk, at), at + chunk.length), 0);
    return { bytes, short: expected > size };
};
const peaks = async (src, bars) => {
    let loaded = await load(src, 'default');
    if (loaded?.short) loaded = await load(src, 'no-store');
    if (!loaded) return null;
    const bytes = loaded.bytes;
    const Context = window.OfflineAudioContext ?? window.webkitOfflineAudioContext;
    const audio = await new Context(1, 1, 8000).decodeAudioData(bytes.buffer);
    const channels = Array.from({ length: audio.numberOfChannels }, (_, at) => audio.getChannelData(at));
    const slice = audio.length / bars;
    const stride = Math.max(1, Math.floor(slice / 2000));
    return Array.from({ length: bars }, (_, bar) => {
        const from = Math.floor(bar * slice), to = Math.floor((bar + 1) * slice);
        let sum = 0, count = 0;
        for (const data of channels) {
            for (let at = from; at < to; at += stride) { sum += data[at] * data[at]; count++; }
        }
        return count ? Math.sqrt(sum / count) : 0;
    });
};";

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Array, Function, Promise};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    use super::{PEAKS_SCRIPT, Peaks, WaveformApi};

    pub(super) struct WebWaveform;

    pub(super) static WAVEFORM: WebWaveform = WebWaveform;

    impl WaveformApi for WebWaveform {
        fn peaks(&self, src: &str, bars: usize) -> Peaks {
            let src = src.to_string();
            Box::pin(async move {
                let script = Function::new_with_args(
                    "src, bars",
                    &format!("{PEAKS_SCRIPT}\nreturn peaks(src, bars).catch(() => null);"),
                );
                let promise = script
                    .call2(
                        &JsValue::NULL,
                        &JsValue::from_str(&src),
                        &JsValue::from(bars as u32),
                    )
                    .ok()?
                    .dyn_into::<Promise>()
                    .ok()?;
                let found: Array = JsFuture::from(promise).await.ok()?.dyn_into().ok()?;
                found.iter().map(|peak| peak.as_f64()).collect()
            })
        }
    }
}

/// `None` where no page decodes: Blitz and a server render. Call it after mount.
pub(crate) fn waveform() -> Option<&'static dyn WaveformApi> {
    #[cfg(target_arch = "wasm32")]
    return Some(&web::WAVEFORM);
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_waveform();
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    return None;
}
