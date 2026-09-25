/// An `Audio` player's strings. `{time}` and `{duration}` are clock times such as `1:05`.
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
    };
}
