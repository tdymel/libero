use std::{future::Future, pin::Pin};

use super::PlatformError;

/// Resolves once the user clicked a pixel, to its color as `#rrggbb`. A
/// dismissed picker (Escape) is [`PlatformError::Denied`].
pub type Pick = Pin<Box<dyn Future<Output = Result<String, PlatformError>>>>;

/// Picks a color off the screen. Not element-scoped, hence its own capability
/// rather than a place on [`ElementApi`](super::ElementApi).
///
/// Answers a hex string, not a color type: the platform layer sits below the
/// components that own one.
pub trait EyeDropperApi {
    fn pick(&self) -> Pick;
}

#[cfg(target_arch = "wasm32")]
mod web {
    use js_sys::{Array, Function, Promise, Reflect};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;

    use super::{EyeDropperApi, Pick};
    use crate::platform::PlatformError;

    /// `window.EyeDropper`, reached by name: `web_sys` binds it only under
    /// `--cfg=web_sys_unstable_apis`, which would leak into every consumer's
    /// build.
    pub(super) fn constructor() -> Option<Function> {
        let window = web_sys::window()?;
        Reflect::get(&window, &JsValue::from_str("EyeDropper"))
            .ok()?
            .dyn_into::<Function>()
            .ok()
    }

    pub(super) struct WebEyeDropper;

    impl EyeDropperApi for WebEyeDropper {
        fn pick(&self) -> Pick {
            Box::pin(async move {
                let constructor = constructor().ok_or(PlatformError::Unsupported)?;
                let dropper = Reflect::construct(&constructor, &Array::new())
                    .map_err(|_| PlatformError::Unsupported)?;
                let open = Reflect::get(&dropper, &JsValue::from_str("open"))
                    .ok()
                    .and_then(|open| open.dyn_into::<Function>().ok())
                    .ok_or(PlatformError::Unsupported)?;
                let promise = open
                    .call0(&dropper)
                    .ok()
                    .and_then(|promise| promise.dyn_into::<Promise>().ok())
                    .ok_or(PlatformError::Unsupported)?;
                // Rejects with an `AbortError` when the user presses Escape.
                let result = JsFuture::from(promise)
                    .await
                    .map_err(|_| PlatformError::Denied)?;
                Reflect::get(&result, &JsValue::from_str("sRGBHex"))
                    .ok()
                    .and_then(|hex| hex.as_string())
                    .ok_or(PlatformError::Unsupported)
            })
        }
    }

    pub(super) static EYE_DROPPER: WebEyeDropper = WebEyeDropper;
}

/// `None` where the platform has no eyedropper - every native target, and
/// every browser but Chromium's today. An absent capability is a missing
/// accessor, never a stub that answers `Unsupported`.
///
/// Call it after mount, never while rendering: a server render answers `None`
/// and a hydrating client would answer otherwise.
pub(crate) fn eye_dropper() -> Option<&'static dyn EyeDropperApi> {
    #[cfg(target_arch = "wasm32")]
    return web::constructor().map(|_| &web::EYE_DROPPER as &'static dyn EyeDropperApi);
    #[cfg(not(target_arch = "wasm32"))]
    return None;
}
