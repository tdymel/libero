use std::rc::Rc;

use dioxus::prelude::MountedData;

use super::PlatformError;

/// What an `<audio>` or `<video>` reports, sent whole on each of its events.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MediaState {
    pub paused: bool,
    pub ended: bool,
    pub current_time: f64,
    /// `None` until the metadata loads, and for a live stream.
    pub duration: Option<f64>,
    pub volume: f64,
    pub muted: bool,
    pub rate: f64,
    /// Playing was asked for, but the data is not there yet.
    pub buffering: bool,
    /// `MediaError.code`: 1 aborted, 2 network, 3 decode, 4 source not supported.
    pub error: Option<u16>,
}

impl Default for MediaState {
    fn default() -> Self {
        Self {
            paused: true,
            ended: false,
            current_time: 0.0,
            duration: None,
            volume: 1.0,
            muted: false,
            rate: 1.0,
            buffering: false,
            error: None,
        }
    }
}

/// Events that change a [`MediaState`]; `timeupdate` is the only frequent one.
#[cfg(any(target_arch = "wasm32", not(feature = "native")))]
pub(crate) const MEDIA_EVENTS: &[&str] = &[
    "play",
    "pause",
    "playing",
    "waiting",
    "stalled",
    "seeking",
    "seeked",
    "timeupdate",
    "durationchange",
    "loadedmetadata",
    "canplay",
    "volumechange",
    "ratechange",
    "ended",
    "emptied",
    "error",
];

/// Dropping it stops the callback.
pub(crate) trait MediaSubscription {}

/// Commands and state of one `<audio>`/`<video>`. Commands are queued where
/// they cannot run at once; the state follows from the element's events.
pub(crate) trait MediaApi {
    fn play(&self) -> Result<(), PlatformError>;
    fn pause(&self) -> Result<(), PlatformError>;
    fn seek(&self, seconds: f64) -> Result<(), PlatformError>;
    fn set_volume(&self, volume: f64) -> Result<(), PlatformError>;
    fn set_muted(&self, muted: bool) -> Result<(), PlatformError>;
    fn set_rate(&self, rate: f64) -> Result<(), PlatformError>;
    /// Shows text track `index` (its order among the element's tracks) and hides
    /// the other captions and subtitles; `None` hides them all.
    fn show_captions(&self, index: Option<usize>) -> Result<(), PlatformError>;
    /// Picks the source again: browsers read `<source>` children only on load.
    fn load(&self) -> Result<(), PlatformError>;
    /// Calls `callback` with the state now (async on a WebView) and after each event.
    fn watch(&self, callback: Box<dyn Fn(MediaState)>) -> Box<dyn MediaSubscription>;
}

/// `None` where nothing plays media (Blitz, a server) or `mounted` is no media
/// element. `tag` is the element's `ElementHandle` tag, which a WebView finds it by.
pub(crate) fn media(mounted: &Rc<MountedData>, tag: Option<u64>) -> Option<Box<dyn MediaApi>> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = tag;
        web::media(mounted)
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "native")))]
    {
        let _ = mounted;
        super::backend::webview_media(tag?)
    }
    // Blitz has no media element: nothing decodes or plays.
    #[cfg(all(not(target_arch = "wasm32"), feature = "native"))]
    {
        let _ = (mounted, tag);
        None
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use std::rc::Rc;

    use dioxus::prelude::MountedData;
    use js_sys::Reflect;
    use wasm_bindgen::{JsCast, prelude::Closure};
    use web_sys::HtmlMediaElement;

    use super::{MEDIA_EVENTS, MediaApi, MediaState, MediaSubscription};
    use crate::platform::PlatformError;

    pub(super) fn media(mounted: &Rc<MountedData>) -> Option<Box<dyn MediaApi>> {
        let element = mounted
            .downcast::<web_sys::Element>()?
            .clone()
            .dyn_into::<HtmlMediaElement>()
            .ok()?;
        Some(Box::new(WebMedia { element }))
    }

    struct WebMedia {
        element: HtmlMediaElement,
    }

    /// `HAVE_FUTURE_DATA`: below it a playing element waits for data.
    const HAVE_FUTURE_DATA: u16 = 3;

    fn state(element: &HtmlMediaElement) -> MediaState {
        let duration = element.duration();
        let paused = element.paused();
        MediaState {
            paused,
            ended: element.ended(),
            current_time: element.current_time(),
            duration: duration.is_finite().then_some(duration),
            volume: element.volume(),
            muted: element.muted(),
            rate: element.playback_rate(),
            buffering: !paused && element.ready_state() < HAVE_FUTURE_DATA,
            error: element.error().map(|error| error.code()),
        }
    }

    impl MediaApi for WebMedia {
        fn play(&self) -> Result<(), PlatformError> {
            let promise = self.element.play().map_err(|_| PlatformError::Denied)?;
            // A refused play (autoplay policy) only shows as staying paused.
            wasm_bindgen_futures::spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            });
            Ok(())
        }

        fn pause(&self) -> Result<(), PlatformError> {
            self.element.pause().map_err(|_| PlatformError::Denied)
        }

        fn seek(&self, seconds: f64) -> Result<(), PlatformError> {
            self.element.set_current_time(seconds);
            Ok(())
        }

        fn set_volume(&self, volume: f64) -> Result<(), PlatformError> {
            self.element.set_volume(volume.clamp(0.0, 1.0));
            Ok(())
        }

        fn set_muted(&self, muted: bool) -> Result<(), PlatformError> {
            self.element.set_muted(muted);
            Ok(())
        }

        fn set_rate(&self, rate: f64) -> Result<(), PlatformError> {
            self.element.set_playback_rate(rate);
            Ok(())
        }

        // Through `Reflect`: the `TextTrack` bindings would grow every app's bundle.
        fn show_captions(&self, index: Option<usize>) -> Result<(), PlatformError> {
            let tracks = Reflect::get(&self.element, &"textTracks".into())
                .map_err(|_| PlatformError::Unsupported)?;
            let length = Reflect::get(&tracks, &"length".into())
                .ok()
                .and_then(|length| length.as_f64())
                .unwrap_or(0.0) as u32;
            for at in 0..length {
                let Ok(track) = Reflect::get_u32(&tracks, at) else {
                    continue;
                };
                let kind = Reflect::get(&track, &"kind".into())
                    .ok()
                    .and_then(|kind| kind.as_string());
                let mode = match (index == Some(at as usize), kind.as_deref()) {
                    (true, _) => "showing",
                    (false, Some("captions" | "subtitles")) => "hidden",
                    _ => continue,
                };
                let _ = Reflect::set(&track, &"mode".into(), &mode.into());
            }
            Ok(())
        }

        fn load(&self) -> Result<(), PlatformError> {
            self.element.load();
            Ok(())
        }

        fn watch(&self, callback: Box<dyn Fn(MediaState)>) -> Box<dyn MediaSubscription> {
            callback(state(&self.element));
            let element = self.element.clone();
            let closure = Closure::<dyn FnMut()>::new(move || callback(state(&element)));
            for event in MEDIA_EVENTS {
                let _ = self
                    .element
                    .add_event_listener_with_callback(event, closure.as_ref().unchecked_ref());
            }
            Box::new(WebMediaSubscription {
                element: self.element.clone(),
                closure,
            })
        }
    }

    struct WebMediaSubscription {
        element: HtmlMediaElement,
        closure: Closure<dyn FnMut()>,
    }

    impl MediaSubscription for WebMediaSubscription {}

    impl Drop for WebMediaSubscription {
        fn drop(&mut self) {
            for event in MEDIA_EVENTS {
                let _ = self.element.remove_event_listener_with_callback(
                    event,
                    self.closure.as_ref().unchecked_ref(),
                );
            }
        }
    }
}
