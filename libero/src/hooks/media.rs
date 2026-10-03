use dioxus::core::Attribute;
use dioxus::prelude::*;

use crate::hooks::{ElementHandle, use_element, use_subscription_slot};
use crate::platform::{self, MediaApi, MediaState, MediaSubscription, PlatformError};

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

    /// Pauses at the current time.
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
    /// A non-finite `seconds` is ignored.
    pub fn seek(&self, seconds: f64) {
        let Some(seconds) = finite(seconds) else {
            return;
        };
        let end = self.duration.peek().unwrap_or(f64::INFINITY);
        // Shown at once: the element's answer crosses the IPC on a WebView.
        self.command(
            self.current_time,
            seconds.clamp(0.0, end),
            |api, seconds| api.seek(seconds),
        );
    }

    /// `0.0` (silent) to `1.0` (full). NaN is ignored.
    pub fn set_volume(&self, volume: f64) {
        if !volume.is_nan() {
            self.command(self.volume, volume.clamp(0.0, 1.0), |api, volume| {
                api.set_volume(volume)
            });
        }
    }

    /// Mutes or unmutes, keeping the volume.
    pub fn set_muted(&self, muted: bool) {
        self.command(self.muted, muted, |api, muted| api.set_muted(muted));
    }

    /// Playback speed: `1.0` is normal. Zero, negative or non-finite is ignored.
    pub fn set_rate(&self, rate: f64) {
        if let Some(rate) = finite(rate).filter(|rate| *rate > 0.0) {
            self.command(self.rate, rate, |api, rate| api.set_rate(rate));
        }
    }

    fn command<T: Copy + 'static>(
        &self,
        signal: Signal<T>,
        value: T,
        send: impl FnOnce(&dyn MediaApi, T) -> Result<(), PlatformError>,
    ) {
        if let Some(api) = self.api() {
            command(&*api, signal, value, send);
        }
    }

    /// Shows text track `index` of the element's, hiding other captions; `None` hides all.
    pub(crate) fn show_captions(&self, index: Option<usize>) {
        if let Some(api) = self.api() {
            let _ = api.show_captions(index);
        }
    }

    /// Loads the element's source again, after its `<source>` children changed.
    pub(crate) fn reload(&self) {
        if let Some(api) = self.api() {
            let _ = api.load();
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

    /// Not playing: before the first play, after a pause, and once ended.
    pub fn paused(&self) -> bool {
        (self.paused)()
    }

    /// Played through to the end.
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

    /// `0.0` (silent) to `1.0` (full), muting aside.
    pub fn volume(&self) -> f64 {
        (self.volume)()
    }

    /// Muted, whatever the volume.
    pub fn muted(&self) -> bool {
        (self.muted)()
    }

    /// Playback speed: `1.0` is normal.
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

/// Runs `send` and shows `value` in `signal` once the element took it.
fn command<T: Copy + 'static>(
    api: &dyn MediaApi,
    mut signal: Signal<T>,
    value: T,
    send: impl FnOnce(&dyn MediaApi, T) -> Result<(), PlatformError>,
) {
    if send(api, value).is_ok() {
        signal.set(value);
    }
}

fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
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
    let slot = use_subscription_slot::<dyn MediaSubscription>();
    use_effect(move || {
        let Some(_) = handle.element.mount_token() else {
            return;
        };
        // Dropped first, so a remount never runs two listeners.
        slot.clear();
        let api = handle.api();
        let mut supported = handle.supported;
        supported.set(Some(api.is_some()));
        slot.set(api.map(|api| api.watch(Box::new(move |state| handle.apply(state)))));
    });
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Refusing;

    impl MediaApi for Refusing {
        fn play(&self) -> Result<(), PlatformError> {
            Err(PlatformError::Unsupported)
        }
        fn pause(&self) -> Result<(), PlatformError> {
            Err(PlatformError::Unsupported)
        }
        fn seek(&self, _: f64) -> Result<(), PlatformError> {
            Err(PlatformError::Unsupported)
        }
        fn set_volume(&self, _: f64) -> Result<(), PlatformError> {
            Ok(())
        }
        fn set_muted(&self, _: bool) -> Result<(), PlatformError> {
            Err(PlatformError::Unsupported)
        }
        fn set_rate(&self, _: f64) -> Result<(), PlatformError> {
            Ok(())
        }
        fn show_captions(&self, _: Option<usize>) -> Result<(), PlatformError> {
            Err(PlatformError::Unsupported)
        }
        fn load(&self) -> Result<(), PlatformError> {
            Err(PlatformError::Unsupported)
        }
        fn watch(&self, _: Box<dyn Fn(MediaState)>) -> Box<dyn MediaSubscription> {
            unreachable!()
        }
    }

    #[test]
    fn a_command_shows_its_value_only_once_the_element_took_it() {
        let dom = VirtualDom::new(|| rsx! {});
        dom.in_runtime(|| {
            let time = Signal::new_in_scope(0.0, ScopeId::ROOT);
            let rate = Signal::new_in_scope(1.0, ScopeId::ROOT);
            command(&Refusing, time, 12.0, |api, seconds| api.seek(seconds));
            command(&Refusing, rate, 2.0, |api, rate| api.set_rate(rate));
            assert_eq!(*time.peek(), 0.0);
            assert_eq!(*rate.peek(), 2.0);
        });
    }

    #[test]
    fn non_finite_input_is_dropped() {
        assert_eq!(finite(f64::NAN), None);
        assert_eq!(finite(f64::INFINITY), None);
        assert_eq!(finite(3.5), Some(3.5));
    }
}
