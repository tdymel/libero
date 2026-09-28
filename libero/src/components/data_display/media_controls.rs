//! The controls `Audio` and `Video` share: play, time, seek, mute, volume and
//! speed, each its own Tab stop, their keys, and the error and buffering messages.

use dioxus::core::{Attribute, AttributeValue};
use dioxus::prelude::*;
use pictogram_icons_lucide as lucide;

use crate::{
    components::{
        accessibility::VisuallyHidden,
        buttons::{ActionIcon, Button},
        common::{Glyph, HtmlTag, Input},
        form::{Slider, SliderChangeEvent},
        layout::{paper_sx, use_box},
        overlay::{Menu, MenuItem, Shortcut, ShortcutHelp, use_menu},
    },
    context::IconSlot,
    hooks::{
        Align, ElementHandle, FullscreenHandle, Hotkey, MediaHandle, ModalScope, PopoverOptions,
        Side, owner_link, use_element, use_formats, use_hotkeys, use_id, use_localization,
        use_modal, use_popover, use_theme,
    },
    localization::fill,
    platform::{ElementApi, key_taken},
    sx::{StaticSx, Sx, ThemeAwareValue, sx},
    theme::{PaperDefaults, Size, SizeCss, Z_INDEX_POPOVER},
};

/// The part slots both players name alike.
pub(super) const CONTROLS: &str = "controls";
pub(super) const TIME: &str = "time";
pub(super) const SEEK: &str = "seek";
pub(super) const VOLUME: &str = "volume";
pub(super) const MESSAGE: &str = "message";

/// The volume menu: a slider on a card.
static VOLUME_MENU_SX: StaticSx = StaticSx::new(|| {
    paper_sx()
        .z_index(Z_INDEX_POPOVER.value())
        .padding(SizeCss::SPACING.value(Size::Sm))
});

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
    pub(super) fn set_volume(self, volume: f64) {
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

/// K plays and pauses, J and L jump 10 seconds, M mutes, Shift+? lists them in a
/// `ShortcutHelp`, all only with focus in `player`; `more` adds a player's own, with
/// its row when it applies. Call it below the player's own `PortalHost`, if any.
pub(super) fn use_media_keys(
    media: MediaHandle,
    sound: Sound,
    player: ElementHandle,
    more: impl IntoIterator<Item = (Hotkey, Option<Shortcut>)>,
) {
    let labels = use_localization().media;
    let (more, extra_rows): (Vec<_>, Vec<_>) = more.into_iter().unzip();
    let shortcuts: Vec<Shortcut> = [
        Shortcut::new("k", labels.shortcut_play),
        Shortcut::new("j", labels.shortcut_back),
        Shortcut::new("l", labels.shortcut_forward),
        Shortcut::new("m", labels.shortcut_mute),
    ]
    .into_iter()
    .chain(extra_rows.into_iter().flatten())
    .chain([Shortcut::new(HELP, labels.shortcut_help)])
    .collect();
    let help = use_modal(move |_: ModalScope<()>| {
        rsx! { ShortcutHelp { shortcuts: shortcuts.clone() } }
    });
    let jump = move |by: f64| media.seek(media.current_time() + by);
    use_hotkeys(
        [
            Hotkey::new("k", move || media.toggle()),
            Hotkey::new("j", move || jump(-10.0)),
            Hotkey::new("l", move || jump(10.0)),
            Hotkey::new("m", move || sound.toggle()),
            Hotkey::new(HELP, move || {
                help.open();
            }),
        ]
        .into_iter()
        .chain(more)
        .map(|hotkey| hotkey.within(player)),
    );
}

/// Matches the `?` a layout produces, Shift+/ or Shift+ß alike.
const HELP: &str = "shift+?";

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
///
/// `overlay` is a video's bar over the picture: the seek track a row of its own
/// on top, then play, mute with its volume menu and time, then captions, speed and fullscreen.
#[component]
pub(super) fn MediaControls(
    media: MediaHandle,
    sound: Sound,
    size: Input<Size>,
    #[props(default)] captions: Option<Captions>,
    #[props(default)] fullscreen: Option<FullscreenHandle>,
    #[props(default)] overlay: bool,
    /// The row's border-box height, each time it changes.
    #[props(default)]
    onheight: Option<EventHandler<f64>>,
) -> Element {
    let labels = use_localization().media;
    let icon_size = icon_size(&size);
    let space_toggles = space_toggles(media);
    let no_captions = use_id();
    let play = rsx! {
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
    };
    let seek = rsx! {
        div { "data-slot": SEEK, onkeydown: space_toggles,
            MediaSeek { media, size: size.clone() }
        }
    };
    let sound_controls = rsx! {
        MediaVolume { media, sound, size: size.clone() }
    };
    let speed = rsx! {
        MediaSpeed { media, size, overlay }
    };
    let captions_button = rsx! {
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
                // The pressed state's bar under the icon; `ActionIcon`'s own pseudo-elements are taken.
                span { "data-mark": "", "aria-hidden": "true" }
            }
            if captions.track.is_none() {
                VisuallyHidden { id: "{no_captions}",{labels.no_captions} }
            }
        }
    };
    let fullscreen_button = rsx! {
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
    };
    // Plain Tab stops in visual order, as native controls: a roving toolbar's
    // arrows clashed with the sliders' (todo 1328).
    rsx! {
        div {
            role: "group",
            "aria-label": labels.controls,
            "data-slot": CONTROLS,
            onresize: move |event: Event<ResizeData>| {
                if let (Some(onheight), Ok(size)) = (onheight, event.get_border_box_size()) {
                    onheight.call(size.height);
                }
            },
            if overlay {
                {seek}
                {play}
                {sound_controls}
                MediaTime { media }
                {captions_button}
                {speed}
                {fullscreen_button}
            } else {
                {play}
                MediaTime { media }
                {seek}
                {sound_controls}
                {speed}
                {captions_button}
                {fullscreen_button}
            }
        }
        MediaStatus { media }
    }
}

/// Space on a slider plays and pauses; a button keeps its own Space, so no hotkey.
pub(super) fn space_toggles(media: MediaHandle) -> impl FnMut(KeyboardEvent) + Copy {
    move |event: KeyboardEvent| {
        if event.key() == Key::Character(" ".into()) && !key_taken(&event) {
            event.prevent_default();
            media.toggle();
        }
    }
}

/// The error alert and the buffering status under a player's controls.
#[component]
pub(super) fn MediaStatus(media: MediaHandle) -> Element {
    let labels = use_localization().media;
    rsx! {
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
fn MediaSpeed(media: MediaHandle, size: Input<Size>, overlay: bool) -> Element {
    let labels = use_localization().media;
    let separator = use_formats().decimal_separator;
    let menu = use_menu();
    let rate = media.rate();
    let items = RATES
        .into_iter()
        .map(|option| {
            MenuItem::new(times(option, separator))
                .radio(option == rate)
                .onselect(move |_| media.set_rate(option))
                .into()
        })
        .collect();
    let shown = times(rate, separator);
    // Narrow padding: the row must fit 320px with every button. Over the
    // picture it takes the bar's light text, as the icons do.
    let mut look = sx().padding_inline(SizeCss::SPACING.value(Size::Xs));
    if overlay {
        look = look.color("inherit");
    }
    let mut attributes = menu.a11y_attributes();
    attributes.push(Attribute::new(
        "aria-label",
        AttributeValue::Text(fill(labels.speed, &[("rate", &shown)])),
        None,
        false,
    ));
    rsx! {
        Menu { state: menu, items, size: size.clone(),
            // "1×" stays "1×" in a right-to-left page, not "×1".
            Button { attributes, dir: "ltr", variant: "standard", size, sx: look, "{shown}" }
        }
    }
}

/// The speaker mutes and unmutes; the chevron beside it opens the volume slider
/// in a menu, which keeps the row narrow.
#[component]
pub(super) fn MediaVolume(media: MediaHandle, sound: Sound, size: Input<Size>) -> Element {
    let labels = use_localization().media;
    let theme = use_theme();
    let silent = sound.silent();
    let mut opened = use_signal(|| false);
    let anchor = use_element();
    let menu_id = use_id();
    let popover = use_popover(
        anchor,
        opened(),
        PopoverOptions::new(theme.popover.gap, theme.popover.padding)
            .side(Side::Top)
            .align(Align::Center)
            .dismiss(true),
    );
    popover.on_dismiss(move || opened.set(false));
    let floating = *popover.floating();
    let card = use_box()
        .framework_sx(&VOLUME_MENU_SX)
        .style(popover.style())
        .prepare();

    // Focus goes to the slider once the card is placed, once per opening.
    let mut entered = use_signal(|| false);
    use_effect(move || match (opened(), popover.placed()) {
        (true, true) if !*entered.peek() => {
            entered.set(true);
            let _ = floating
                .query_selector("[role='slider']")
                .and_then(|thumb| thumb.focus());
        }
        (false, _) => entered.set(false),
        _ => {}
    });

    popover.show(opened().then(|| {
        let mut attributes = popover.floating_events();
        attributes.extend(owner_link(&anchor));
        card.element(&floating)
            .attr("id", menu_id.cloned())
            .attr("role", "dialog")
            .attr("aria-label", labels.volume)
            // Tab leaves the card for its trigger, as a menu does.
            .event("onkeydown", move |event: KeyboardEvent| {
                if event.key() == Key::Tab {
                    event.prevent_default();
                    opened.set(false);
                    let _ = anchor.focus();
                } else {
                    let mut space_toggles = space_toggles(media);
                    space_toggles(event);
                }
            })
            .render(
                HtmlTag::Div,
                attributes,
                rsx! {
                    Slider::<f64> {
                        value: media.volume() * 100.0,
                        min: 0.0,
                        max: 100.0,
                        step: 5.0,
                        size: size.clone(),
                        aria_label: labels.volume,
                        // The card sizes to its content, which a `100%` track has none of.
                        sx: sx().width("8rem"),
                        oninput: move |event: SliderChangeEvent| sound.set_volume(event.value() / 100.0),
                    }
                },
            )
    }));

    rsx! {
        div { "data-slot": VOLUME,
            ActionIcon {
                aria_label: if silent { labels.unmute } else { labels.mute },
                tooltip: true,
                shortcut: "m",
                size: icon_size(&size),
                onclick: move |_| sound.toggle(),
                if silent {
                    Glyph { slot: IconSlot::VolumeOff, icon: lucide::volume_x::outlined }
                } else {
                    Glyph { slot: IconSlot::Volume, icon: lucide::volume_2::outlined }
                }
            }
            ActionIcon {
                aria_label: labels.volume,
                aria_haspopup: "dialog",
                aria_expanded: if opened() { "true" } else { "false" },
                "aria-controls": opened().then(|| menu_id.cloned()),
                size: icon_size(&size),
                // Half a button: it only opens the menu, so the speaker stays the target.
                sx: sx().min_width("1.25em").width("1.25em"),
                attributes: popover.anchor_events(),
                onmounted: anchor.mount(),
                onclick: move |_| opened.toggle(),
                Glyph { slot: IconSlot::ChevronUp, icon: lucide::chevron_up::outlined }
            }
        }
    }
}

/// `1.5×`, or `1,5×` with a comma `separator`.
pub(super) fn times(rate: f64, separator: &str) -> String {
    format!("{rate}×").replace('.', separator)
}

/// The seek slider. While dragged it shows the thumb's value, not the element's
/// older answer, which trails on a WebView.
#[component]
pub(super) fn MediaSeek(media: MediaHandle, size: Input<Size>) -> Element {
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
                // A press on the track seeks at once: a click sends no `Change`.
                SliderChangeEvent::Start(seconds) => {
                    scrub.set(Some(seconds));
                    media.seek(seconds);
                }
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
pub(super) fn clock(seconds: f64) -> String {
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
            RATES.map(|rate| times(rate, ".")),
            ["0.5×", "0.75×", "1×", "1.25×", "1.5×", "2×"]
        );
    }

    /// Todo 1352: German writes `1,5×`.
    #[test]
    fn a_rate_takes_the_locale_decimal_mark() {
        assert_eq!(times(1.5, ","), "1,5×");
        assert_eq!(times(2.0, ","), "2×");
    }
}
