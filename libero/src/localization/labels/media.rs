/// An `Audio` or `Video` player's strings. `{time}` and `{duration}` are clock times such as `1:05`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaLabels {
    pub controls: &'static str,
    pub play: &'static str,
    pub pause: &'static str,
    pub mute: &'static str,
    pub unmute: &'static str,
    pub seek: &'static str,
    pub volume: &'static str,
    /// The seek slider's value: `{time}` of `{duration}`.
    pub position: &'static str,
    pub buffering: &'static str,
    /// Shown, and announced, when the source fails.
    pub error: &'static str,
    /// Where nothing plays media, above a link to the file.
    pub unsupported: &'static str,
    pub download: &'static str,
    pub captions: &'static str,
    pub fullscreen: &'static str,
    pub exit_fullscreen: &'static str,
    /// The speed button's name; `{rate}` is its text, such as `1.5×`.
    pub speed: &'static str,
    /// Why a video's captions button is disabled.
    pub no_captions: &'static str,
    /// The track menu's item that hides every caption and subtitle track.
    pub captions_off: &'static str,
    /// A video's chapters menu button.
    pub chapters: &'static str,
    /// The rows of the `ShortcutHelp` that Shift+? opens in the player.
    pub shortcut_play: &'static str,
    pub shortcut_back: &'static str,
    pub shortcut_forward: &'static str,
    pub shortcut_mute: &'static str,
    pub shortcut_captions: &'static str,
    pub shortcut_fullscreen: &'static str,
    pub shortcut_next_chapter: &'static str,
    pub shortcut_previous_chapter: &'static str,
    pub shortcut_help: &'static str,
}

impl MediaLabels {
    pub const ENGLISH: Self = Self {
        controls: "Player controls",
        play: "Play",
        pause: "Pause",
        mute: "Mute",
        unmute: "Unmute",
        seek: "Seek",
        volume: "Volume",
        position: "{time} of {duration}",
        buffering: "Loading",
        error: "This media could not be played.",
        unsupported: "This app cannot play media here.",
        download: "Open the file",
        captions: "Captions",
        fullscreen: "Fullscreen",
        exit_fullscreen: "Exit fullscreen",
        speed: "Playback speed {rate}",
        no_captions: "No captions for this video",
        captions_off: "Off",
        chapters: "Chapters",
        shortcut_play: "Play or pause",
        shortcut_back: "Back 10 seconds",
        shortcut_forward: "Forward 10 seconds",
        shortcut_mute: "Mute or unmute",
        shortcut_captions: "Captions on or off",
        shortcut_fullscreen: "Fullscreen on or off",
        shortcut_next_chapter: "Next chapter",
        shortcut_previous_chapter: "Previous chapter",
        shortcut_help: "Show these shortcuts",
    };

    pub const GERMAN: Self = Self {
        controls: "Wiedergabesteuerung",
        play: "Abspielen",
        pause: "Pausieren",
        mute: "Stummschalten",
        unmute: "Ton an",
        seek: "Position",
        volume: "Lautstärke",
        position: "{time} von {duration}",
        buffering: "Lädt",
        error: "Dieses Medium kann nicht abgespielt werden.",
        unsupported: "Diese App kann hier keine Medien abspielen.",
        download: "Datei öffnen",
        captions: "Untertitel",
        fullscreen: "Vollbild",
        exit_fullscreen: "Vollbild beenden",
        speed: "Wiedergabegeschwindigkeit {rate}",
        no_captions: "Keine Untertitel für dieses Video",
        captions_off: "Aus",
        chapters: "Kapitel",
        shortcut_play: "Abspielen oder pausieren",
        shortcut_back: "10 Sekunden zurückspulen",
        shortcut_forward: "10 Sekunden vorspulen",
        shortcut_mute: "Stummschalten oder Ton an",
        shortcut_captions: "Untertitel ein oder aus",
        shortcut_fullscreen: "Vollbild ein oder aus",
        shortcut_next_chapter: "Nächstes Kapitel",
        shortcut_previous_chapter: "Vorheriges Kapitel",
        shortcut_help: "Tastenkürzel anzeigen",
    };
}
