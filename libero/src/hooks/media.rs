use std::{cell::RefCell, rc::Rc};

use dioxus::core::Attribute;
use dioxus::prelude::*;

use crate::hooks::{ElementHandle, use_element};
use crate::platform::{self, MediaApi, MediaState, MediaSubscription};

/// Plays and reads one `<audio>` or `<video>`: commands go to the element, and
/// each read is its own signal, so a time tick re-renders only what shows time.
///
/// Spread [`attributes`](Self::attributes) and set [`mount`](Self::mount) on the
/// element. Where nothing plays media (Blitz, a server render) every command is
/// a no-op and [`is_supported`](Self::is_supported) stays `false`.
#[derive(Clone, Copy, PartialEq)]
pub struct MediaHandle {
    element: ElementHandle,
    supported: Signal<Option<bool>>,
    paused: Signal<bool>,
    ended: Signal<bool>,
    current_time: Signal<f64>,
    duration: Signal<Option<f64>>,
    volume: Signal<f64>,
    muted: Signal<bool>,
    rate: Signal<f64>,
    buffering: Signal<bool>,
    error: Signal<Option<u16>>,
}

impl MediaHandle {
    /// Spread on the media element (`..media.attributes()`), so a WebView finds it.
    pub fn attributes(&self) -> Vec<Attribute> {
        self.element.attributes()
    }

    /// The media element's `onmounted` handler.
    pub fn mount(self) -> impl FnMut(Event<MountedData>) + 'static {
        self.element.mount()
    }

    /// The element underneath, for focus or measuring.
    pub fn element(&self) -> ElementHandle {
        self.element
    }

    fn api(&self) -> Option<Box<dyn MediaApi>> {
        let mounted = self.element.mounted()?;
        platform::media(&mounted, self.element.tag())
    }

    /// Starts playing. A browser may refuse one not started by a press (autoplay
    /// policy): the media then stays [`paused`](Self::paused).
    pub fn play(&self) {
        if let Some(api) = self.api() {
            let _ = api.play();
        }
    }

    pub fn pause(&self) {
        if let Some(api) = self.api() {
            let _ = api.pause();
        }
    }

    /// Plays when paused, pauses when playing.
    pub fn toggle(&self) {
        match *self.paused.peek() {
            true => self.play(),
            false => self.pause(),
        }
    }

    /// Jumps to `seconds` from the start, clamped to the duration once known.
    pub fn seek(&self, seconds: f64) {
        let end = self.duration.peek().unwrap_or(f64::INFINITY);
        let seconds = seconds.clamp(0.0, end);
        if let Some(api) = self.api() {
            let _ = api.seek(seconds);
            // Shown at once: the element's answer crosses the IPC on a WebView.
            let mut current_time = self.current_time;
            current_time.set(seconds);
        }
    }

    /// `0.0` (silent) to `1.0` (full).
    pub fn set_volume(&self, volume: f64) {
        let volume = volume.clamp(0.0, 1.0);
        if let Some(api) = self.api() {
            let _ = api.set_volume(volume);
            let mut signal = self.volume;
            signal.set(volume);
        }
    }

    pub fn set_muted(&self, muted: bool) {
        if let Some(api) = self.api() {
            let _ = api.set_muted(muted);
            let mut signal = self.muted;
            signal.set(muted);
        }
    }

    /// Playback speed: `1.0` is normal.
    pub fn set_rate(&self, rate: f64) {
        if let Some(api) = self.api() {
            let _ = api.set_rate(rate);
        }
    }

    /// Shows text track `index` of the element's, hiding other captions; `None` hides all.
    pub(crate) fn show_captions(&self, index: Option<usize>) {
        if let Some(api) = self.api() {
            let _ = api.show_captions(index);
        }
    }

    /// Reactive, as every read below. `false` until the element mounts, and
    /// always where nothing plays media.
    pub fn is_supported(&self) -> bool {
        (self.supported)() == Some(true)
    }

    /// `Some(false)` once the element mounted where nothing plays media; `None` before.
    pub(crate) fn supported(&self) -> Option<bool> {
        (self.supported)()
    }

    pub fn paused(&self) -> bool {
        (self.paused)()
    }

    pub fn ended(&self) -> bool {
        (self.ended)()
    }

    /// Seconds from the start.
    pub fn current_time(&self) -> f64 {
        (self.current_time)()
    }

    /// Seconds; `None` until the metadata loads, and for a live stream.
    pub fn duration(&self) -> Option<f64> {
        (self.duration)()
    }

    pub fn volume(&self) -> f64 {
        (self.volume)()
    }

    pub fn muted(&self) -> bool {
        (self.muted)()
    }

    pub fn rate(&self) -> f64 {
        (self.rate)()
    }

    /// Playing was asked for, but the data has not arrived yet.
    pub fn buffering(&self) -> bool {
        (self.buffering)()
    }

    /// Why the source failed to load or play, if it did.
    pub fn error(&self) -> Option<MediaError> {
        (self.error)().map(MediaError::from_code)
    }

    /// Writes only what changed, so an unchanged read does not re-render.
    fn apply(mut self, state: MediaState) {
        fn put<T: PartialEq + 'static>(signal: &mut Signal<T>, value: T) {
            if *signal.peek() != value {
                signal.set(value);
            }
        }
        put(&mut self.paused, state.paused);
        put(&mut self.ended, state.ended);
        put(&mut self.current_time, state.current_time);
        put(&mut self.duration, state.duration);
        put(&mut self.volume, state.volume);
        put(&mut self.muted, state.muted);
        put(&mut self.rate, state.rate);
        put(&mut self.buffering, state.buffering);
        put(&mut self.error, state.error);
    }
}

/// Why media failed, from the element's `MediaError.code`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaError {
    /// The fetch was aborted, by the reader or the app.
    Aborted,
    /// A network error stopped the download.
    Network,
    /// The data could not be decoded.
    Decode,
    /// The source or its format is not supported.
    SourceNotSupported,
}

impl MediaError {
    fn from_code(code: u16) -> Self {
        match code {
            1 => Self::Aborted,
            2 => Self::Network,
            3 => Self::Decode,
            _ => Self::SourceNotSupported,
        }
    }
}

/// Plays and reads an `<audio>` or `<video>` you render yourself; `Audio` and
/// `Video` take one through their `media` prop.
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::hooks::use_media;
/// # fn app() -> Element {
/// let media = use_media();
///
/// rsx! {
///     audio { src: "/podcast.ogg", onmounted: media.mount(), ..media.attributes() }
///     button {
///         onclick: move |_| media.toggle(),
///         if media.paused() { "Play" } else { "Pause" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/hooks/use-media>
pub fn use_media() -> MediaHandle {
    let defaults = MediaState::default();
    let handle = MediaHandle {
        element: use_element(),
        supported: use_signal(|| None),
        paused: use_signal(|| defaults.paused),
        ended: use_signal(|| defaults.ended),
        current_time: use_signal(|| defaults.current_time),
        duration: use_signal(|| defaults.duration),
        volume: use_signal(|| defaults.volume),
        muted: use_signal(|| defaults.muted),
        rate: use_signal(|| defaults.rate),
        buffering: use_signal(|| defaults.buffering),
        error: use_signal(|| defaults.error),
    };
    let slot: Rc<RefCell<Option<Box<dyn MediaSubscription>>>> =
        use_hook(|| Rc::new(RefCell::new(None)));
    use_drop({
        let slot = slot.clone();
        move || drop(slot.borrow_mut().take())
    });
    use_effect(move || {
        let Some(_) = handle.element.mount_token() else {
            return;
        };
        // Dropped first, so a remount never runs two listeners.
        slot.borrow_mut().take();
        let api = handle.api();
        let mut supported = handle.supported;
        supported.set(Some(api.is_some()));
        *slot.borrow_mut() = api.map(|api| api.watch(Box::new(move |state| handle.apply(state))));
    });
    handle
}
