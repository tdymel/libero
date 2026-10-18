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

    /// `None` when no provider is in reach. A URL that is not absolute `http(s)` fails the fetch.
    pub(super) fn fetch(url: &str) -> Option<Fetched> {
        let provider = dioxus::prelude::try_consume_context::<Arc<dyn NetProvider>>()?;
        let Ok(url) = Url::parse(url) else {
            crate::utils::warn(&format!("Not fetched: `{url}` is not an absolute URL."));
            return Some(Box::pin(async { None }));
        };
        if !super::is_web_scheme(url.scheme()) {
            crate::utils::warn(&format!("Not fetched: `{url}` is not an http(s) URL."));
            return Some(Box::pin(async { None }));
        }
        let (sender, receiver) = oneshot::channel();
        // The id only picks the document to redraw; the task's own wake re-renders.
        provider.fetch(0, Request::get(url), Box::new(Reply(sender)));
        Some(Box::pin(async move {
            String::from_utf8(receiver.await.ok()?.to_vec()).ok()
        }))
    }
}

/// Whether the native provider may fetch `scheme`: `file:` and the like would read the disk
/// for a URL that came from user data.
#[cfg(any(test, all(not(target_arch = "wasm32"), feature = "native")))]
fn is_web_scheme(scheme: &str) -> bool {
    matches!(scheme, "http" | "https")
}

/// Protocols the page's `fetch` may load for an absolute URL: media may be `data:` or `blob:`,
/// while `javascript:`, `file:` and the like stay refused. A relative URL is always fetched.
const WEB_PROTOCOLS: [&str; 4] = ["http:", "https:", "data:", "blob:"];

fn web_fetch_script() -> String {
    let allowed = WEB_PROTOCOLS
        .map(|protocol| format!("'{protocol}'"))
        .join(", ");
    format!(
        "const url = await dioxus.recv();
        try {{
            const absolute = (() => {{ try {{ return new URL(url); }} catch (error) {{ return null; }} }})();
            if (absolute && ![{allowed}].includes(absolute.protocol)) return null;
            const response = await fetch(url);
            return response.ok ? await response.text() : null;
        }} catch (error) {{
            return null;
        }}"
    )
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
    // The web and the WebView: the page's own `fetch`. Without JS the eval fails, which reads as a failed fetch.
    let eval = dioxus::document::eval(&web_fetch_script());
    let _ = eval.send(url);
    Box::pin(async move { eval.join::<Option<String>>().await.ok().flatten() })
}

#[cfg(test)]
mod tests {
    use super::{is_web_scheme, web_fetch_script};

    #[test]
    fn the_web_arm_allows_data_and_blob_but_refuses_javascript() {
        let script = web_fetch_script();
        assert!(
            script.contains("['http:', 'https:', 'data:', 'blob:']"),
            "{script}"
        );
        for protocol in ["javascript:", "vbscript:", "file:"] {
            assert!(!script.contains(protocol), "{protocol}");
        }
    }

    #[test]
    fn only_http_and_https_are_fetched_natively() {
        assert!(is_web_scheme("http") && is_web_scheme("https"));
        for scheme in ["file", "ftp", "data", "javascript", "HTTP", ""] {
            assert!(!is_web_scheme(scheme), "{scheme}");
        }
    }
}
