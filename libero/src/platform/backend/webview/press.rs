use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::json;

use super::{Slot, WebViewListener, eval_with, runs_scripts};
use crate::platform::{PRESS_MARKER_ATTR, PressApi, PressSubscription};

pub(crate) fn press() -> Option<&'static dyn PressApi> {
    runs_scripts().then_some(&PRESS as &'static dyn PressApi)
}

struct WebViewPress;

static PRESS: WebViewPress = WebViewPress;

/// Capture phase, so no handler that stops the press hides it. Sends every
/// marker from the target up to the root.
const ON_PRESS: &str = "const onPress = (event) => {
        const markers = [];
        for (let el = event.target instanceof Element ? event.target : null; el; el = el.parentElement) {
            const value = el.getAttribute(data);
            if (value) markers.push(...value.split(' ').map(Number));
        }
        dioxus.send(markers);
    };
    window.addEventListener('pointerdown', onPress, { capture: true });";

impl PressApi for WebViewPress {
    fn on_press(&self, callback: Box<dyn Fn(Vec<u64>)>) -> Box<dyn PressSubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!(PRESS_MARKER_ATTR),
            &format!(
                "{ON_PRESS}
                {}
                window.removeEventListener('pointerdown', onPress, {{ capture: true }});",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(markers) = script.recv::<Vec<u64>>().await {
                callback(markers);
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

impl PressSubscription for WebViewListener {}
