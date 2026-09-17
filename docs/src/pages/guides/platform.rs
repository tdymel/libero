use crate::components::{DocPage, DocSection};
use dioxus::prelude::*;
use libero::{
    components::{Code, CodeBlock, DataList, DataListItem, Text},
    platform::{KeyChord, keyboard},
};

/// Each row is `(accessor, methods, where it is None)`.
const ACCESSORS: [(&str, &str, &str); 5] = [
    (
        "timer() -> Option<&'static dyn TimerApi>",
        "after(delay, callback), every(interval, callback) - each returns a TimerSubscription.",
        "Some on every renderer; None only outside a dioxus runtime. Inert during a server render.",
    ),
    (
        "keyboard() -> Option<&'static dyn KeyboardApi>",
        "on_key(callback), on_key_unfiltered(callback) - each returns a KeySubscription. The callback gets a KeyChord and returns true to prevent the default.",
        "None off the web.",
    ),
    (
        "scroll() -> Option<&'static dyn ScrollApi>",
        "on_scroll(callback) - anything scrolling, not just the page. Returns a ScrollSubscription.",
        "None in a webview and a headless build. Natively it hears a wheel inside LiberoProvider and libero's own scroll_to/scroll_into_view.",
    ),
    (
        "document() -> Option<&'static dyn DocumentApi>",
        "active_element(), viewport().",
        "None where the renderer exposes no document: a webview, and any headless build.",
    ),
    (
        "clock() -> Option<&'static dyn ClockApi>",
        "today() - the local calendar day.",
        "Some on every renderer: JS Date on the web, the system clock and time zone off it. Call it after mount, never while rendering.",
    ),
];

const SHORTCUT: &str = r#"#[component]
fn GoCounter() -> Element {
    let presses = use_signal(|| 0);
    // Dropping the subscription unsubscribes, so the signal holds it for
    // as long as the component lives. Off the web it is None, and the
    // shortcut is simply not there.
    let mut shortcut = use_signal(move || {
        keyboard().map(|keyboard| {
            keyboard.on_key(Box::new(move |chord: KeyChord| {
                let hit = chord.key == Key::Character("g".into()) && !chord.repeat;
                if hit {
                    let mut presses = presses;
                    presses += 1;
                }
                hit
            }))
        })
    });
    use_drop(move || shortcut.set(None));

    rsx! {
        Text { "G pressed {presses} times" }
    }
}"#;

/// `SHORTCUT`, rendered. Kept in step with the snippet by hand.
#[component]
fn GoCounter() -> Element {
    let presses = use_signal(|| 0);
    let mut shortcut = use_signal(move || {
        keyboard().map(|keyboard| {
            keyboard.on_key(Box::new(move |chord: KeyChord| {
                let hit = chord.key == Key::Character("g".into()) && !chord.repeat;
                if hit {
                    let mut presses = presses;
                    presses += 1;
                }
                hit
            }))
        })
    });
    use_drop(move || shortcut.set(None));

    rsx! {
        Text { "G pressed {presses} times" }
    }
}

#[component]
pub fn PlatformPage() -> Element {
    rsx! {
        DocPage {
            title: "Platform",
            markdown: "/md/platform.md",
            lead: rsx! {
                Text {
                    "Everything that reaches the machine underneath dioxus goes through "
                    Code { source: "libero::platform" }
                    ": one trait per capability, and one accessor that returns "
                    Code { source: "Option<&'static dyn Api>" }
                    ". "
                    Code { source: "None" }
                    " means the running renderer cannot do it, so a caller branches once "
                    "and never unwraps: a feature that has no platform underneath it is "
                    "absent, not broken."
                }
                Text {
                    "The callback-shaped capabilities return a subscription, and dropping "
                    "it is how you stop: a dropped timer never fires, a dropped key "
                    "subscription hears nothing more. The callback runs outside every "
                    "scope, so what it writes must be a signal that outlives the moment."
                }
            },

            DocSection {
                title: "The accessors",
                DataList {
                    for (accessor, methods, none) in ACCESSORS {
                        DataListItem {
                            key: "{accessor}",
                            label: rsx! {
                                Code { source: accessor }
                            },
                            Text { "{methods}" }
                            Text { "{none}" }
                        }
                    }
                }
            }

            DocSection {
                title: "A subscription",
                Text {
                    "Press G anywhere on this page but in a text field - "
                    Code { source: "on_key" }
                    " never hears a key the user is typing. Never return "
                    Code { source: "true" }
                    " for Tab or Shift+Tab: the listener runs before everything else, and "
                    "keyboard focus would have nowhere to go."
                }
                GoCounter {}
                CodeBlock { source: SHORTCUT, language: "rust" }
                Text {
                    Code { source: "on_key_unfiltered" }
                    " hears the presses "
                    Code { source: "on_key" }
                    " drops, those inside a text field too. Use it for one chord, Escape "
                    "being the usual one, and return "
                    Code { source: "false" }
                    " to everything else."
                }
            }

            DocSection {
                title: "Elements",
                Text {
                    "An element has no accessor, because there is no portable way to name "
                    "one. "
                    Code { source: "use_element()" }
                    " returns a handle you mount with "
                    Code { source: "onmounted: handle.mount()" }
                    ", and the handle implements "
                    Code { source: "ElementApi" }
                    ". Commands - "
                    Code { source: "focus()" }
                    ", "
                    Code { source: "scroll_to()" }
                    ", "
                    Code { source: "set_pointer_capture()" }
                    " - return at once. Reads - "
                    Code { source: "dimensions()" }
                    ", "
                    Code { source: "client_offset()" }
                    ", "
                    Code { source: "scroll_offset()" }
                    " - are futures: start one in the event handler and await it in a "
                    Code { source: "spawn" }
                    ". What a renderer cannot serve answers "
                    Code { source: "PlatformError::Unsupported" }
                    " rather than being missing from the type."
                }
            }
        }
    }
}
