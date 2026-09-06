//! Deliberately broken fixtures, one per pass.
//!
//! These exist to answer the only question that matters about an assertion
//! library: **can it fail?** A pass that has never been seen to fail is
//! indistinguishable from a pass that cannot, and the second kind reports as
//! coverage forever.
//!
//! Each fixture here violates exactly one thing, in the way the matching pass
//! claims to catch. `tests/all/negative.rs` asserts the pass returns an error
//! for each. If one of these ever goes green, the pass guarding it has stopped
//! working and every component relying on it is unguarded too.
//!
//! Written in plain HTML rather than with `libero` components on purpose: the
//! subject here is the harness, not the library. A broken fixture built out of
//! library components would be testing that the library can be misused.

use dioxus::prelude::*;

/// WCAG 2.4.7 Focus Visible: a control that removes its own indicator.
#[component]
pub fn NoFocusRing() -> Element {
    rsx! {
        document::Style { "#no-ring:focus, #no-ring:focus-visible {{ outline: none; box-shadow: none; }}" }
        button { id: "no-ring", style: "padding: 8px 16px; border: 1px solid #888;", "No ring" }
    }
}

/// WCAG 2.5.8 Target Size (Minimum): 16x16, under the 24x24 floor.
#[component]
pub fn TinyTarget() -> Element {
    rsx! {
        button {
            id: "tiny",
            role: "slider",
            "aria-label": "Tiny",
            style: "width: 16px; height: 16px; padding: 0; border: 1px solid #888;",
        }
    }
}

/// WCAG 1.4.3 Contrast (Minimum): grey on white, far under 4.5:1.
#[component]
pub fn LowContrast() -> Element {
    rsx! {
        p {
            id: "faint",
            style: "color: #bbbbbb; background: #ffffff; font-size: 14px;",
            "This text does not meet 4.5:1."
        }
    }
}

/// An uncaught exception during mount, which the console recorder must see.
///
/// Mount time specifically: that is where the errors the recorder exists for
/// actually happen, and it is the window a recorder attached after navigation
/// would miss.
#[component]
pub fn ConsoleError() -> Element {
    rsx! {
        document::Script { "throw new Error('fixture: deliberate mount-time error');" }
        p { "This fixture throws while mounting." }
    }
}

/// APG roving tabindex violated: every item is its own tab stop.
///
/// The important part is that this **works**. Every tab is reachable and
/// nothing throws; it is simply three tab stops where the pattern allows one.
/// A naive "can a keyboard reach it" check passes here.
#[component]
pub fn ManyTabStops() -> Element {
    rsx! {
        div { role: "tablist", "aria-label": "Broken strip",
            button { role: "tab", tabindex: "0", "One" }
            button { role: "tab", tabindex: "0", "Two" }
            button { role: "tab", tabindex: "0", "Three" }
        }
    }
}

/// APG dialog: closes, but drops focus to `<body>` instead of returning it.
///
/// Nothing looks wrong on screen. The keyboard user is simply at the top of the
/// document with no idea where they are.
#[component]
pub fn NoFocusReturn() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        button {
            id: "open-broken",
            onclick: move |_| open.set(true),
            "Open"
        }
        if open() {
            div {
                role: "dialog",
                id: "broken-dialog",
                "aria-label": "Broken dialog",
                style: "border: 1px solid #888; padding: 16px;",
                tabindex: "-1",
                onmounted: |event: Event<MountedData>| async move {
                    let _ = event.data().set_focus(true).await;
                },
                onkeydown: move |event| {
                    if event.key() == Key::Escape {
                        // Closes, and deliberately returns focus nowhere.
                        open.set(false);
                    }
                },
                button { "Inside" }
            }
        }
    }
}

/// A dismissed panel that stays in the accessibility tree.
///
/// Hidden with `opacity: 0` only, which hides nothing from assistive
/// technology: the contents are still announced and the button inside is still
/// tabbable. This is the shape an animated close takes if nobody adds `inert`
/// or `visibility: hidden` for the duration.
#[component]
pub fn PhantomPanel() -> Element {
    let mut open = use_signal(|| true);

    rsx! {
        button { id: "dismiss", onclick: move |_| open.set(false), "Dismiss" }
        div {
            role: "dialog",
            id: "phantom",
            "aria-label": "Phantom",
            style: if open() { "opacity: 1;" } else { "opacity: 0;" },
            button { "Still reachable" }
        }
    }
}

/// `aria-activedescendant` naming an element that is not in the DOM.
///
/// This is todo 101's bug, reproduced: the attribute is present and
/// well-formed, and only its referent is missing. A screen reader following it
/// finds nothing.
///
/// It has to behave like a real combobox up to that point - start closed, open
/// on ArrowDown, expose a listbox - or the contract fails on an earlier step
/// and never reaches the check this fixture exists to exercise. The first
/// version of this fixture started open, so `Combobox` failed on its
/// "starts closed" assertion instead, and the negative test passed while
/// proving nothing about dangling references at all.
#[component]
pub fn DanglingActiveDescendant() -> Element {
    let mut open = use_signal(|| false);

    rsx! {
        input {
            id: "dangling",
            role: "combobox",
            "aria-label": "Dangling",
            "aria-expanded": if open() { "true" } else { "false" },
            "aria-controls": "dangling-listbox",
            "aria-activedescendant": if open() { "option-that-does-not-exist" } else { "" },
            readonly: true,
            onkeydown: move |event| match event.key() {
                Key::ArrowDown => open.set(true),
                Key::Escape => open.set(false),
                _ => {}
            },
        }
        if open() {
            ul {
                id: "dangling-listbox",
                role: "listbox",
                style: "border: 1px solid #888; margin: 0; padding: 4px;",
                li { role: "option", id: "a-real-option", "Real option" }
            }
        }
    }
}

/// A combobox that opens but never sets `aria-activedescendant` at all.
///
/// The contract used to treat an absent attribute as valid, so this passed
/// it (review 7, E2): a screen reader hears nothing while the arrows move.
#[component]
pub fn MissingActiveDescendant() -> Element {
    rsx! { FakeCombobox { id: "missing", highlight: None } }
}

/// A combobox whose highlight names a real option and never moves.
///
/// Every reference resolves, so a check that only asks "does it exist" passes
/// it. Only a check that the arrows move the highlight catches it.
#[component]
pub fn StaticHighlight() -> Element {
    rsx! { FakeCombobox { id: "static", highlight: Some("static-option-0") } }
}

/// Starts closed, opens on ArrowDown, closes on Escape, and draws three
/// options with ids - a real combobox in every respect but its highlight, so
/// the contract reaches the highlight checks instead of failing earlier.
#[component]
fn FakeCombobox(id: &'static str, highlight: Option<&'static str>) -> Element {
    let mut open = use_signal(|| false);
    let listbox = format!("{id}-listbox");

    rsx! {
        input {
            id,
            role: "combobox",
            "aria-label": "Fake combobox",
            "aria-expanded": if open() { "true" } else { "false" },
            "aria-controls": "{listbox}",
            "aria-activedescendant": highlight.filter(|_| open()),
            readonly: true,
            onkeydown: move |event| match event.key() {
                Key::ArrowDown => open.set(true),
                Key::Escape => open.set(false),
                _ => {}
            },
        }
        if open() {
            ul {
                id: "{listbox}",
                role: "listbox",
                style: "border: 1px solid #888; margin: 0; padding: 4px;",
                for row in 0..3 {
                    li { role: "option", id: "{id}-option-{row}", "Option {row}" }
                }
            }
        }
    }
}
