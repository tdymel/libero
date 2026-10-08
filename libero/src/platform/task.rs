/// Resolves once the platform has run the rest of its current task. On the web a
/// spawned task runs between `focusout` and `focusin`; natively this is instant.
pub(crate) async fn next_task() {
    #[cfg(target_arch = "wasm32")]
    web::next_task().await;
}

/// [`next_task`], and on a WebView one eval round trip: past a `focusout`'s `focusin`
/// there too (todo 2664). Not `next_task` itself: that slowed other WebView callers.
pub(crate) async fn page_task() {
    #[cfg(target_arch = "wasm32")]
    web::next_task().await;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    super::backend::webview_next_task().await;
}

/// Runs `run` once element reads can answer: at once, or natively, where a
/// render or a task holds Blitz's document, at the end of this poll.
pub(crate) fn when_free(run: impl FnOnce() + 'static) {
    super::backend::when_free(Box::new(run));
}

/// [`when_free`], once what this render mounted has a layout: natively the
/// next poll, since the shell lays out after effects run.
pub(crate) fn when_laid_out(run: impl FnOnce() + 'static) {
    super::backend::when_laid_out(Box::new(run));
}

#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    pub(super) async fn next_task() {
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            let scheduled = web_sys::window()
                .and_then(|window| window.set_timeout_with_callback(&resolve).ok())
                .is_some();
            // Without a timer, resolving now beats never resolving.
            if !scheduled {
                let _ = resolve.call0(&JsValue::NULL);
            }
        });
        let _ = JsFuture::from(promise).await;
    }
}
