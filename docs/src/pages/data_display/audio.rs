use crate::components::{Control, Demo, DemoValues, DocPage, a11y, prop, props};
use dioxus::prelude::*;
use libero::{
    components::{Audio, AudioPart, Code, Input, Text},
    theme::Size,
};

#[component]
pub fn AudioPage() -> Element {
    rsx! {
        DocPage {
            title: "Audio",
            source: "libero/src/components/data_display/audio.rs",
            markdown: "/md/audio.md",
            properties: vec![
                props("Audio", vec![
                    prop("src", "String").default("required").doc("The file's URL."),
                    prop("sources", "Vec<MediaSource>")
                        .default("[]")
                        .doc("The file in other formats, `MediaSource::new(src, mime)`, tried in order before `src`. `src` stays the fallback and the download link."),
                    prop("label", "String")
                        .default("required")
                        .doc("Names the player, e.g. the track's title."),
                    prop("media", "Option<MediaHandle>")
                        .default("None")
                        .doc("A handle from `use_media()`, to drive or read the player from outside."),
                    prop("autoplay", "bool")
                        .default("false")
                        .doc("Starts on load. Browsers refuse it with sound: pair it with `muted`; a debug build warns otherwise."),
                    prop("muted", "bool").default("false").doc("Starts muted."),
                    prop("looping", "bool").default("false").doc("Starts again at the end."),
                    prop("preload", "MediaPreload")
                        .default("metadata")
                        .doc("How much loads before a press: `none`, `metadata` or `auto`."),
                    prop("size", "Size").default("theme").doc("Of the buttons and sliders."),
                    prop("onplay", "EventHandler<()>").default("None").doc("Playing started."),
                    prop("onpause", "EventHandler<()>").default("None").doc("Playing paused."),
                    prop("onended", "EventHandler<()>").default("None").doc("Playing reached the end."),
                    prop("onerror", "EventHandler<MediaError>")
                        .default("None")
                        .doc("The source failed: `Aborted`, `Network`, `Decode` or `SourceNotSupported`."),
                    prop("children", "Element")
                        .default("a sentence and a link")
                        .doc("Shown instead of the controls where nothing plays media (Blitz)."),
                ])
                .parts("AudioPart", vec![
                    (AudioPart::Controls, "The row of controls, a `Toolbar`."),
                    (AudioPart::Time, "The elapsed and total time."),
                    (AudioPart::Seek, "The seek slider's wrapper."),
                    (AudioPart::Volume, "The volume slider's wrapper."),
                    (AudioPart::Message, "The error text, or the fallback where nothing plays media."),
                ]),
            ],
            accessibility: a11y()
                .key(["K"], "Plays or pauses, with focus anywhere in the player.")
                .key(["Space"], "On a slider: plays or pauses. On a button: presses it.")
                .key(["J", "L"], "Jumps 10 seconds back or ahead.")
                .key(["M"], "Mutes or unmutes.")
                .key(["Left", "Right"], "On the seek slider: 1 second; on the volume slider: 5%.")
                .handles([
                    "The player is a `group` named by `label`; its buttons are one `toolbar` stop, each slider its own.",
                    "The play and mute buttons change their names (Play/Pause, Mute/Unmute) rather than using `aria-pressed`.",
                    "Below 22rem the volume slider hides and the mute button stays, so the row fits at 320px (WCAG 1.4.10).",
                    "The seek slider's `aria-valuetext` reads \"1:05 of 4:56\" (the localization's `media.position`).",
                    "A polite status says \"Loading\" while playing waits for data; a failed source shows an alert.",
                    "No autoplay unless asked, and a debug warning for autoplay with sound (WCAG 1.4.2).",
                ])
                .must([
                    "Give each player a `label` that says what plays.",
                    "Offer a transcript for speech (WCAG 1.2.1): libero cannot write it.",
                ])
                .limits([
                    "Blitz plays no media: the controls give way to `children`, by default a link to the file.",
                    "On a WebView each command and state change crosses the IPC, so the time trails by a moment.",
                    "With `sources`, a file changed after mount is not loaded: the browser reads `<source>` once, so remount the player to swap files.",
                ]),
            lead: rsx! {
                Text {
                    "An "
                    Code { source: "<audio>" }
                    " with libero's own controls: play, seek, time, mute and volume, in the theme's look on "
                    "every platform that plays media. "
                    Code { source: "use_media()" }
                    " drives the same engine for a layout of your own: spread its "
                    Code { source: "attributes()" }
                    " and "
                    Code { source: "mount()" }
                    " on a bare element and read "
                    Code { source: "paused()" }
                    ", "
                    Code { source: "current_time()" }
                    " and friends."
                }
            },
            Demo {
                component: "Audio",
                children_text: "",
                fixed: vec![
                    format!("src: {:?}", crate::site::SAMPLE_AUDIO),
                    r#"label: "Wikipedia guitar solo""#.to_string(),
                ],
                controls: vec![
                    Control::slider("size", ["xs", "sm", "md", "lg", "xl"]).default("md"),
                    Control::switch("muted"),
                    Control::switch("looping"),
                ],
                render: move |values: DemoValues| rsx! {
                    Audio {
                        src: crate::site::SAMPLE_AUDIO,
                        label: "Wikipedia guitar solo",
                        size: Input::from(Size::from(values.str("size").as_str())),
                        muted: values.str("muted") == "true",
                        looping: values.str("looping") == "true",
                    }
                },
            }
        }
    }
}
