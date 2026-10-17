use std::rc::Rc;

use dioxus::prelude::spawn;
use serde_json::{Value, json};

use super::{Slot, WebViewListener, eval, eval_with, runs_scripts};
use crate::platform::fullscreen::{FullscreenApi, FullscreenState, FullscreenSubscription};
use crate::platform::media::{MEDIA_EVENTS, MediaApi, MediaState, MediaSubscription};
use crate::platform::{OBSERVE_ATTR, PlatformError, Read};

pub(crate) fn media(tag: u64) -> Option<Box<dyn MediaApi>> {
    runs_scripts().then(|| Box::new(WebViewMedia { tag }) as Box<dyn MediaApi>)
}

/// The `<audio>`/`<video>` tagged `tag`, found per call: no handle reaches the DOM.
struct WebViewMedia {
    tag: u64,
}

/// `data` is `[attr, tag, ...]`; the script sees the element as `media`, maybe `null`.
const FIND_MEDIA: &str =
    "const media = document.querySelector('[' + data[0] + '=\"' + data[1] + '\"]');";

impl WebViewMedia {
    /// Queued behind the edits dioxus sent before it.
    fn command(&self, argument: Value, script: &'static str) -> Result<(), PlatformError> {
        eval_with(
            json!([OBSERVE_ATTR, self.tag.to_string(), argument]),
            &format!("{FIND_MEDIA}\nconst value = data[2];\nif (media) {{ {script} }}"),
        );
        Ok(())
    }
}

/// Sends the state on each media event, `timeupdate` at most every 250 ms: each
/// message crosses the IPC. Waits for the element if it is not there yet.
/// `exhausted`: every `<source>` failed, which sets no `MediaError` (4 stands in, as on the web).
const ON_MEDIA: &str = "let media = null;
    let last = 0;
    let exhausted = false;
    let alive = true;
    const send = (event) => {
        if (event?.type === 'emptied' || event?.type === 'loadstart') exhausted = false;
        const now = Date.now();
        if (event?.type === 'timeupdate' && now - last < 250) return;
        last = now;
        const duration = isFinite(media.duration) ? media.duration : null;
        const error = media.error?.code ?? (exhausted ? 4 : null);
        dioxus.send([media.paused, media.ended, media.currentTime, duration, media.volume,
            media.muted, media.playbackRate, !media.paused && media.readyState < 3 && error === null, error]);
    };
    const exhaust = () => { exhausted = true; send(); };
    // A `<source>`'s `error` does not bubble: the capture phase sees the last one fail.
    const onSource = (event) => {
        if (event.target?.tagName !== 'SOURCE') return;
        for (let next = event.target.nextElementSibling; next; next = next.nextElementSibling) {
            if (next.tagName === 'SOURCE') return;
        }
        exhaust();
    };
    const start = () => {
        media = document.querySelector('[' + data[0] + '=\"' + data[1] + '\"]');
        if (!media) return false;
        for (const name of data[2]) media.addEventListener(name, send);
        media.addEventListener('loadstart', send);
        media.addEventListener('error', onSource, true);
        send();
        // Found after every `<source>` failed: no `error` is left to see. A task later, as a
        // selection just begun also sits in NETWORK_NO_SOURCE until then.
        setTimeout(() => {
            if (alive && !exhausted && media.networkState === 3 && media.querySelector('source')) exhaust();
        }, 0);
        return true;
    };
    const waiting = start() ? null : new MutationObserver(() => {
        if (start()) waiting.disconnect();
    });
    waiting?.observe(document.documentElement, { childList: true, subtree: true, attributes: true, attributeFilter: [data[0]] });";

impl MediaApi for WebViewMedia {
    fn play(&self) -> Result<(), PlatformError> {
        // A refused play (autoplay policy) only shows as staying paused.
        self.command(Value::Null, "media.play().catch(() => {});")
    }

    fn pause(&self) -> Result<(), PlatformError> {
        self.command(Value::Null, "media.pause();")
    }

    fn seek(&self, seconds: f64) -> Result<(), PlatformError> {
        self.command(json!(seconds), "media.currentTime = value;")
    }

    fn set_volume(&self, volume: f64) -> Result<(), PlatformError> {
        self.command(json!(volume.clamp(0.0, 1.0)), "media.volume = value;")
    }

    fn set_muted(&self, muted: bool) -> Result<(), PlatformError> {
        self.command(json!(muted), "media.muted = value;")
    }

    fn set_rate(&self, rate: f64) -> Result<(), PlatformError> {
        self.command(json!(rate), "media.playbackRate = value;")
    }

    fn show_captions(&self, index: Option<usize>) -> Result<(), PlatformError> {
        self.command(
            json!(index),
            "Array.from(media.textTracks).forEach((track, at) => {
                if (at === value) track.mode = 'showing';
                else if (track.kind === 'captions' || track.kind === 'subtitles') track.mode = 'hidden';
            });",
        )
    }

    fn load(&self) -> Result<(), PlatformError> {
        self.command(Value::Null, "media.load();")
    }

    fn watch(&self, callback: Box<dyn Fn(MediaState)>) -> Box<dyn MediaSubscription> {
        type Sent = (
            bool,
            bool,
            f64,
            Option<f64>,
            f64,
            bool,
            f64,
            bool,
            Option<u16>,
        );
        let slot = Slot::new();
        let script = eval_with(
            json!([OBSERVE_ATTR, self.tag.to_string(), MEDIA_EVENTS]),
            &format!(
                "{ON_MEDIA}
                {}
                alive = false;
                waiting?.disconnect();
                if (media) {{
                    for (const name of data[2]) media.removeEventListener(name, send);
                    media.removeEventListener('loadstart', send);
                    media.removeEventListener('error', onSource, true);
                }}",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok(sent) = script.recv::<Sent>().await {
                let (paused, ended, current_time, duration, volume, muted, rate, buffering, error) =
                    sent;
                callback(MediaState {
                    paused,
                    ended,
                    current_time,
                    duration,
                    volume,
                    muted,
                    rate,
                    buffering,
                    error,
                });
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

pub(crate) fn fullscreen(tag: u64) -> Option<Box<dyn FullscreenApi>> {
    runs_scripts().then(|| Box::new(WebViewFullscreen { tag }) as Box<dyn FullscreenApi>)
}

/// The element tagged `tag`, found per call as [`WebViewMedia`] is.
struct WebViewFullscreen {
    tag: u64,
}

impl FullscreenApi for WebViewFullscreen {
    fn enter(&self) -> Read<()> {
        let read = eval_with(
            json!([OBSERVE_ATTR, self.tag.to_string()]),
            &format!(
                "{FIND_MEDIA}\ntry {{ await media.requestFullscreen(); return true; }} catch {{ return false; }}"
            ),
        );
        Box::pin(async move {
            match read.join::<bool>().await {
                Ok(true) => Ok(()),
                _ => Err(PlatformError::Denied),
            }
        })
    }

    fn exit(&self) -> Result<(), PlatformError> {
        eval("if (document.fullscreenElement) document.exitFullscreen().catch(() => {});");
        Ok(())
    }

    fn watch(&self, callback: Box<dyn Fn(FullscreenState)>) -> Box<dyn FullscreenSubscription> {
        let slot = Slot::new();
        let script = eval_with(
            json!([OBSERVE_ATTR, self.tag.to_string()]),
            &format!(
                "const selector = '[' + data[0] + '=\"' + data[1] + '\"]';
                const send = () => dioxus.send([document.fullscreenEnabled === true,
                    document.fullscreenElement?.matches(selector) === true]);
                // Android's WebView leaves fullscreen on Back only; Escape does as a browser's.
                const escape = (event) => {{
                    if (event.key !== 'Escape' || !document.fullscreenElement?.matches(selector)) return;
                    event.preventDefault();
                    event.stopImmediatePropagation();
                    document.exitFullscreen().catch(() => {{}});
                }};
                document.addEventListener('fullscreenchange', send);
                window.addEventListener('keydown', escape, true);
                send();
                {}
                document.removeEventListener('fullscreenchange', send);
                window.removeEventListener('keydown', escape, true);",
                slot.park("")
            ),
        );
        let task = spawn(async move {
            let mut script = script;
            while let Ok((available, active)) = script.recv::<(bool, bool)>().await {
                callback(FullscreenState { available, active });
            }
        });
        Box::new(WebViewListener {
            task,
            _slot: Rc::new(slot),
        })
    }
}

impl MediaSubscription for WebViewListener {}

impl FullscreenSubscription for WebViewListener {}
