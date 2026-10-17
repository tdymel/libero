use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::json;

use super::{Slot, WebViewListener, eval_with, runs_scripts};
use crate::platform::{MediaQueryApi, MediaQuerySubscription};

pub(crate) fn media_query() -> Option<&'static dyn MediaQueryApi> {
    runs_scripts().then_some(&MEDIA_QUERY as &'static dyn MediaQueryApi)
}

struct WebViewMediaQuery;

static MEDIA_QUERY: WebViewMediaQuery = WebViewMediaQuery;

impl MediaQueryApi for WebViewMediaQuery {
    /// The first answer crosses the IPC, so it arrives after the call returns.
    fn watch(&self, query: &str, callback: Box<dyn Fn(bool)>) -> Box<dyn MediaQuerySubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!(query),
            &format!(
                "const list = window.matchMedia(data);
                const send = () => dioxus.send(list.matches);
                send();
                list.addEventListener('change', send);
                {}
                list.removeEventListener('change', send);",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(matches) = script.recv::<bool>().await {
                callback(matches);
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

impl MediaQuerySubscription for WebViewListener {}
