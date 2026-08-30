/// Resolves once the platform has run the rest of its current task.
///
/// A spawned dioxus task runs as a microtask on the web, which is *between*
/// `focusout` and `focusin`: a check there sees focus nowhere. Awaiting this
/// first sees where it landed. Native renderers dispatch focus changes as one
/// step, so there it resolves at once.
pub(crate) async fn next_task() {
    #[cfg(target_arch = "wasm32")]
    web::next_task().await;
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
