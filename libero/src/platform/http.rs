use std::{future::Future, pin::Pin};

/// A response's body, `None` on any failure: no network, a status outside 2xx.
pub(crate) type Fetched = Pin<Box<dyn Future<Output = Option<String>>>>;

/// Blitz runs no JS, so the renderer's own network provider fetches: dioxus-native
/// puts it in the root context (`native` only, and only with its `net` feature).
#[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
mod native {
    use std::sync::Arc;

    use blitz_traits::net::{Bytes, NetHandler, NetProvider, Request, Url};
    use futures_channel::oneshot;

    use super::Fetched;

    /// Passes the body on. A failed fetch drops it unanswered, which cancels the channel.
    struct Reply(oneshot::Sender<Bytes>);

    impl NetHandler for Reply {
        fn bytes(self: Box<Self>, _: String, bytes: Bytes) {
            let _ = self.0.send(bytes);
        }
    }

    /// `None` when no provider is in reach.
    pub(super) fn fetch(url: &str) -> Option<Fetched> {
        let provider = dioxus::prelude::try_consume_context::<Arc<dyn NetProvider>>()?;
        let url = Url::parse(url).ok()?;
        let (sender, receiver) = oneshot::channel();
        // The id only picks the document to redraw; the task's own wake re-renders.
        provider.fetch(0, Request::get(url), Box::new(Reply(sender)));
        Some(Box::pin(async move {
            String::from_utf8(receiver.await.ok()?.to_vec()).ok()
        }))
    }
}

/// GETs `url` and hands back its body. Call it in a component scope: the
/// native provider comes from the context.
pub(crate) fn fetch_text(url: &str) -> Fetched {
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    match native::fetch(url) {
        Some(fetched) => return fetched,
        None => crate::utils::warn(
            "No network provider in reach: dioxus-native fetches only with its `net` feature (on by default).",
        ),
    }
    // The web and the WebView: the page's own `fetch`. Without JS the eval
    // fails, which reads as a failed fetch.
    let eval = dioxus::document::eval(
        "const url = await dioxus.recv();
        try {
            const response = await fetch(url);
            return response.ok ? await response.text() : null;
        } catch (error) {
            return null;
        }",
    );
    let _ = eval.send(url);
    Box::pin(async move { eval.join::<Option<String>>().await.ok().flatten() })
}
