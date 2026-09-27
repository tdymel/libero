//! The controls `Audio` and `Video` share: play, time, seek, mute, volume and
//! speed, each its own Tab stop, their keys, and the error and buffering messages.

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::{ActionIcon, Button},
        common::{Glyph, Input},
        form::{Slider, SliderChangeEvent},
        overlay::{Menu, MenuItem, use_menu},
    },
    context::IconSlot,
    hooks::{
        ElementHandle, FullscreenHandle, Hotkey, MediaHandle, use_hotkeys, use_id, use_localization,
    },
    localization::fill,
    platform::key_taken,
    sx::{Sx, ThemeAwareValue, sx},
    theme::{PaperDefaults, Size, SizeCss},
};

/// The part slots both players name alike.
pub(super) const CONTROLS: &str = "controls";
pub(super) const TIME: &str = "time";
pub(super) const SEEK: &str = "seek";
pub(super) const VOLUME: &str = "volume";
pub(super) const MESSAGE: &str = "message";

/// The speeds the speed menu offers.
const RATES: [f64; 6] = [0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

/// The controls row: no wrap until the seek track is down to `2rem`, buttons
/// never shrink. Each player adds its own breakpoints and surface.
pub(super) fn controls_sx() -> Sx {
    sx().display("flex")
        .align_items("center")
        .gap(SizeCss::SPACING.value(Size::Xs))
        .flex_wrap("wrap")
        .min_width("0")
        .padding(SizeCss::SPACING.value(Size::Xs))
        // 3:1 against the page in both schemes (WCAG 1.4.11), so the bar reads as one.
        .border_style("solid")
        .border_width("1px")
        .border_color("muted.6")
        .border_radius(SizeCss::RADIUS.value(Size::Sm))
        // The surface it sits on, not a tint: `muted.1` left the speed label at 4.13:1.
        .and(PaperDefaults::background_sx())
        .selector("& > *", sx().flex_shrink("0"))
}

/// The seek wrapper: takes what the row leaves, down to `2rem`.
pub(super) fn seek_sx() -> Sx {
    sx().flex("1 1 0").min_width("2rem")
}

/// An `ActionIcon` size from the player's `size`.
pub(super) fn icon_size(size: &Input<Size>) -> Input<ThemeAwareValue> {
    match size.as_ref() {
        Some(size) => Input::Value(ThemeAwareValue::Size(*size)),
        None => Input::None,
    }
}

/// Mute and volume as one: silent is muted or at volume 0, and unmuting at 0
/// brings back the last audible volume, as native controls do.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Sound {
    media: MediaHandle,
    audible: Signal<f64>,
}

impl Sound {
    pub(super) fn silent(&self) -> bool {
        self.media.muted() || self.media.volume() == 0.0
    }

    pub(super) fn toggle(self) {
        let media = self.media;
        if !self.silent() {
            media.set_muted(true);
            return;
        }
        media.set_muted(false);
        if media.volume() == 0.0 {
            media.set_volume(*self.audible.peek());
        }
    }

    /// A volume above 0 unmutes.
    fn set_volume(self, volume: f64) {
        self.media.set_volume(volume);
        if volume > 0.0 && self.media.muted() {
            self.media.set_muted(false);
        }
    }
}

pub(super) fn use_sound(media: MediaHandle) -> Sound {
    let mut audible = use_signal(|| 1.0);
    use_effect(move || {
        let volume = media.volume();
        if volume > 0.0 {
            audible.set(volume);
        }
    });
    Sound { media, audible }
}

/// K plays and pauses, J and L jump 10 seconds, M mutes, all only with focus in
/// `player`; `more` adds a player's own.
pub(super) fn use_media_keys(
    media: MediaHandle,
    sound: Sound,
    player: ElementHandle,
    more: impl IntoIterator<Item = Hotkey>,
) {
    let jump = move |by: f64| media.seek(media.current_time() + by);
    use_hotkeys(
        [
            Hotkey::new("k", move || media.toggle()),
            Hotkey::new("j", move || jump(-10.0)),
            Hotkey::new("l", move || jump(10.0)),
            Hotkey::new("m", move || sound.toggle()),
        ]
        .into_iter()
        .chain(more)
        .map(|hotkey| hotkey.within(player)),
    );
}

/// A video's text track the captions button shows and hides.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct Captions {
    pub shown: Signal<bool>,
    /// Its index among the element's tracks; `None` disables the button.
    pub track: Option<usize>,
}

impl Captions {
    pub(super) fn toggle(mut self, media: MediaHandle) {
        let Some(track) = self.track else {
            return;
        };
        let on = !*self.shown.peek();
        media.show_captions(on.then_some(track));
        self.shown.set(on);
    }
}

/// The row of controls, a video's captions and fullscreen buttons at its end,
/// then the error and buffering messages. Those buttons come as data, not an
/// `Element`: a memoized child would keep handlers its parent's re-render dropped.
#[component]
pub(super) fn MediaControls(
    media: MediaHandle,
    sound: Sound,
    size: Input<Size>,
    #[props(default)] captions: Option<Captions>,
    #[props(default)] fullscreen: Option<FullscreenHandle>,
) -> Element {
    let labels = use_localization().media;
    let icon_size = icon_size(&size);
    // Space on a slider plays and pauses; a button keeps its own Space, so no hotkey.
    let space_toggles = move |event: KeyboardEvent| {
        if event.key() == Key::Character(" ".into()) && !key_taken(&event) {
            event.prevent_default();
            media.toggle();
        }
    };
    let silent = sound.silent();
    let no_captions = use_id();
    // Plain Tab stops in visual order, as native controls: a roving toolbar's
    // arrows clashed with the sliders' (todo 1328).
    rsx! {
        div { role: "group", "aria-label": labels.controls, "data-slot": CONTROLS,
            ActionIcon {
                aria_label: if media.paused() { labels.play } else { labels.pause },
                tooltip: true,
                shortcut: "k",
                size: icon_size.clone(),
                onclick: move |_| media.toggle(),
                if media.paused() {
                    Glyph { slot: IconSlot::Play, icon: lucide::play::outlined }
                } else {
                    Glyph { slot: IconSlot::Pause, icon: lucide::pause::outlined }
                }
            }
            MediaTime { media }
            div { "data-slot": SEEK, onkeydown: space_toggles,
                MediaSeek { media, size: size.clone() }
            }
            ActionIcon {
                aria_label: if silent { labels.unmute } else { labels.mute },
                tooltip: true,
                shortcut: "m",
                size: icon_size.clone(),
                onclick: move |_| sound.toggle(),
                if silent {
                    Glyph { slot: IconSlot::VolumeOff, icon: lucide::volume_x::outlined }
                } else {
                    Glyph { slot: IconSlot::Volume, icon: lucide::volume_2::outlined }
                }
            }
            div { "data-slot": VOLUME, onkeydown: space_toggles,
                Slider::<f64> {
                    value: media.volume() * 100.0,
                    min: 0.0,
                    max: 100.0,
                    step: 5.0,
                    size: size.clone(),
                    aria_label: labels.volume,
                    oninput: move |event: SliderChangeEvent| sound.set_volume(event.value() / 100.0),
                }
            }
            MediaSpeed { media, size }
            if let Some(captions) = captions {
                // Without a track: still a Tab stop, so its reason is heard (todo 1324).
                ActionIcon {
                    aria_label: labels.captions,
                    aria_pressed: if captions.track.is_some() && (captions.shown)() { "true" } else { "false" },
                    "aria-describedby": captions.track.is_none().then(|| no_captions.cloned()),
                    disabled: captions.track.is_none(),
                    focusable_when_disabled: true,
                    tooltip: true,
                    shortcut: "c",
                    size: icon_size.clone(),
                    onclick: move |_| captions.toggle(media),
                    Glyph { slot: IconSlot::Captions, icon: lucide::captions::outlined }
                }
                if captions.track.is_none() {
                    VisuallyHidden { id: "{no_captions}",{labels.no_captions} }
                }
            }
            if let Some(fullscreen) = fullscreen {
                ActionIcon {
                    aria_label: if fullscreen.is_fullscreen() { labels.exit_fullscreen } else { labels.fullscreen },
                    tooltip: true,
                    shortcut: "f",
                    size: icon_size.clone(),
                    onclick: move |_| fullscreen.toggle(),
                    if fullscreen.is_fullscreen() {
                        Glyph { slot: IconSlot::ExitFullscreen, icon: lucide::minimize::outlined }
                    } else {
                        Glyph { slot: IconSlot::Fullscreen, icon: lucide::maximize::outlined }
                    }
                }
            }
        }
        if media.error().is_some() {
            p { "data-slot": MESSAGE, role: "alert", {labels.error} }
        }
        // Mounted throughout, so a screen reader hears the change.
        VisuallyHidden { role: "status",
            if media.buffering() { {labels.buffering} }
        }
    }
}

/// Elapsed and total time; its own component, so a time tick re-renders only this.
#[component]
fn MediaTime(media: MediaHandle) -> Element {
    let time = clock(media.current_time());
    let total = media.duration().map_or_else(|| "--:--".to_string(), clock);
    rsx! {
        // The total in its own span, so a narrow player can drop it.
        span { "data-slot": TIME, "aria-hidden": "true", "{time}", span { " / {total}" } }
    }
}

/// The speed button, "1×", and its menu of [`RATES`].
#[component]
fn MediaSpeed(media: MediaHandle, size: Input<Size>) -> Element {
    let labels = use_localization().media;
    let menu = use_menu();
    let rate = media.rate();
    let items = RATES
        .into_iter()
        .map(|option| {
            MenuItem::new(times(option))
                .radio(option == rate)
                .onselect(move |_| media.set_rate(option))
                .into()
        })
        .collect();
    let shown = times(rate);
    let mut attributes = menu.a11y_attributes();
    attributes.push(Attribute::new(
        "aria-label",
        AttributeValue::Text(fill(labels.speed, &[("rate", &shown)])),
        None,
        false,
    ));
    rsx! {
        Menu { state: menu, items, size: size.clone(),
            // Narrow padding: the row must fit 320px with every button.
            Button {
                attributes,
                variant: "standard",
                size,
                sx: sx().padding_inline(SizeCss::SPACING.value(Size::Xs)),
                "{shown}"
            }
        }
    }
}

/// `1.5×`.
fn times(rate: f64) -> String {
    format!("{rate}×")
}

/// The seek slider. While dragged it shows the thumb's value, not the element's
/// older answer, which trails on a WebView.
#[component]
fn MediaSeek(media: MediaHandle, size: Input<Size>) -> Element {
    let labels = use_localization().media;
    let mut scrub = use_signal(|| None::<f64>);
    let duration = media.duration();
    let value = scrub().unwrap_or_else(|| media.current_time());
    let total = duration.map_or_else(|| "--:--".to_string(), clock);
    let format = use_callback(move |seconds: f64| {
        fill(
            labels.position,
            &[("time", &clock(seconds)), ("duration", &total)],
        )
    });
    rsx! {
        Slider::<f64> {
            value,
            min: 0.0,
            max: duration.unwrap_or(1.0).max(1.0),
            step: 1.0,
            size,
            disabled: duration.is_none(),
            aria_label: labels.seek,
            format,
            oninput: move |event: SliderChangeEvent| match event {
                SliderChangeEvent::Start(seconds) => scrub.set(Some(seconds)),
                SliderChangeEvent::Change(seconds) => {
                    if scrub.peek().is_some() {
                        scrub.set(Some(seconds));
                    }
                    media.seek(seconds);
                }
                SliderChangeEvent::End(_) => scrub.set(None),
            },
        }
    }
}

/// The fallback where nothing plays media: `children`, else a sentence and a link to `src`.
#[component]
pub(super) fn MediaFallback(src: String, children: Element) -> Element {
    let labels = use_localization().media;
    rsx! {
        div { "data-slot": MESSAGE,
            if children != VNode::empty() {
                {children}
            } else {
                p { {labels.unsupported} }
                a { href: src, {labels.download} }
            }
        }
    }
}

/// `m:ss`, or `h:mm:ss` from an hour.
fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0).floor() as u64;
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    match hours {
        0 => format!("{minutes}:{seconds:02}"),
        _ => format!("{hours}:{minutes:02}:{seconds:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clock_shows_hours_only_from_an_hour() {
        assert_eq!(clock(0.0), "0:00");
        assert_eq!(clock(65.9), "1:05");
        assert_eq!(clock(3599.0), "59:59");
        assert_eq!(clock(3725.0), "1:02:05");
        assert_eq!(clock(-3.0), "0:00");
    }

    #[test]
    fn a_rate_reads_without_trailing_zeros() {
        assert_eq!(
            RATES.map(times),
            ["0.5×", "0.75×", "1×", "1.25×", "1.5×", "2×"]
        );
    }
}
