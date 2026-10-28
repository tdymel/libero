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

/// Runs `run` once the press under way, if any, has ended and its `click` has run; at once
/// natively. A blur's reveal waits for it, or the error it shows moves the button pressed (4756).
pub(crate) fn after_press(run: impl FnOnce() + 'static) {
    #[cfg(target_arch = "wasm32")]
    dioxus::prelude::spawn(async move {
        web::press_ended().await;
        next_task().await;
        run();
    });
    // The WebView too: its render lands mid-press, past the IPC round trip (4796).
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    dioxus::prelude::spawn(async move {
        super::backend::webview_press_ended().await;
        run();
    });
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    run();
}

/// Starts tracking presses for [`after_press`], once per page; before the first press it waits on.
pub(crate) fn watch_presses() {
    #[cfg(target_arch = "wasm32")]
    web::watch_presses();
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    super::backend::webview_watch_presses();
}

/// `true` once the page's web fonts have loaded; `false` at once where there is no fonts
/// API (Blitz).
pub(crate) async fn fonts_ready() -> bool {
    #[cfg(target_arch = "wasm32")]
    return web::fonts_ready().await;
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    return super::backend::webview_fonts_ready().await;
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    false
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

/// [`when_laid_out`] for a retry that found no layout `failed` times: natively each
/// asks a frame longer, as the zero-delay wake may come before the shell's layout.
pub(crate) fn when_laid_out_retry(failed: u8, run: impl FnOnce() + 'static) {
    super::backend::when_laid_out_retry(failed, Box::new(run));
}

/// Whether a [`when_laid_out`] run still waits for its wake: a native harness
/// keeps polling until it ran, as a shell's next frame would.
#[doc(hidden)]
pub fn awaits_layout() -> bool {
    super::backend::awaits_layout()
}

/// A Tab move's focus scroll on Blitz, `scroll-padding` and `scroll-margin` honoured:
/// for a native harness that focuses by hand (2863). Call it inside the runtime.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
#[doc(hidden)]
pub fn reveal_in(doc: &mut blitz_dom::BaseDocument, node_id: dioxus_native_dom::NodeId) {
    super::backend::reveal(doc, node_id);
}

/// Before a native harness moves focus by hand, so subscribers hear the move as they
/// do for `focus()`. Call it inside the runtime.
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
#[doc(hidden)]
pub fn watch_focus_in(doc: &blitz_dom::BaseDocument) {
    super::backend::watch_focus(doc);
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::{Cell, RefCell};

    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use wasm_bindgen_futures::JsFuture;

    thread_local! {
        static WATCHING: Cell<bool> = const { Cell::new(false) };
        static PRESSED: Cell<bool> = const { Cell::new(false) };
        /// What `press_ended` resolves at the press's `pointerup` or `pointercancel`.
        static WAITING: RefCell<Vec<js_sys::Function>> = const { RefCell::new(Vec::new()) };
    }

    pub(super) fn watch_presses() {
        if WATCHING.replace(true) {
            return;
        }
        let Some(window) = web_sys::window() else {
            return;
        };
        let down = Closure::<dyn Fn()>::new(|| PRESSED.set(true));
        let up = Closure::<dyn Fn()>::new(|| {
            PRESSED.set(false);
            for resolve in WAITING.take() {
                let _ = resolve.call0(&JsValue::NULL);
            }
        });
        // Capture on the window: a control that stops its press still counts.
        let _ = window.add_event_listener_with_callback_and_bool(
            "pointerdown",
            down.as_ref().unchecked_ref(),
            true,
        );
        for kind in ["pointerup", "pointercancel"] {
            let _ = window.add_event_listener_with_callback_and_bool(
                kind,
                up.as_ref().unchecked_ref(),
                true,
            );
        }
        down.forget();
        up.forget();
    }

    pub(super) async fn press_ended() {
        if !PRESSED.get() {
            return;
        }
        let promise = js_sys::Promise::new(&mut |resolve, _| {
            WAITING.with_borrow_mut(|waiting| waiting.push(resolve));
        });
        let _ = JsFuture::from(promise).await;
    }

    pub(super) async fn fonts_ready() -> bool {
        let ready = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| js_sys::Reflect::get(&document, &JsValue::from_str("fonts")).ok())
            .and_then(|fonts| js_sys::Reflect::get(&fonts, &JsValue::from_str("ready")).ok())
            .and_then(|ready| ready.dyn_into::<js_sys::Promise>().ok());
        match ready {
            Some(ready) => JsFuture::from(ready).await.is_ok(),
            None => false,
        }
    }

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
