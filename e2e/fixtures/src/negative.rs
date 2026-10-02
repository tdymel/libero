//! Deliberately broken fixtures, each violating exactly one thing its pass claims to catch.
//! Plain HTML on purpose: the subject is the harness, not the library (`tests/all/negative.rs`).

use dioxus::prelude::*;

use crate::Routes;

pub const ROUTES: Routes = &[
    ("/broken/focus-ring", || rsx! { NoFocusRing {} }),
    ("/broken/translucent-ring", || rsx! { TranslucentRing {} }),
    ("/broken/clipped-ring", || rsx! { ClippedRing {} }),
    ("/broken/covered-focus", || rsx! { CoveredFocus {} }),
    ("/broken/reflow", || rsx! { NoReflow {} }),
    ("/broken/target-size", || rsx! { TinyTarget {} }),
    ("/broken/target-spacing", || rsx! { CrampedTargets {} }),
    ("/broken/contrast", || rsx! { LowContrast {} }),
    ("/broken/console", || rsx! { ConsoleError {} }),
    ("/broken/faint-field-ring", || rsx! { FaintFieldRing {} }),
    ("/broken/console-warning", || rsx! { ConsoleWarning {} }),
    ("/broken/roving", || rsx! { ManyTabStops {} }),
    ("/broken/focus-return", || rsx! { NoFocusReturn {} }),
    ("/broken/dismissal", || rsx! { PhantomPanel {} }),
    (
        "/broken/activedescendant",
        || rsx! { DanglingActiveDescendant {} },
    ),
    (
        "/broken/activedescendant-missing",
        || rsx! { MissingActiveDescendant {} },
    ),
    ("/broken/static-highlight", || rsx! { StaticHighlight {} }),
    ("/broken/faint-boundary", || rsx! { FaintBoundary {} }),
    ("/broken/layered-text", || rsx! { LayeredText {} }),
];

/// WCAG 1.4.11: a field whose only boundary is a border near the page's white.
#[component]
pub fn FaintBoundary() -> Element {
    rsx! {
        input {
            id: "faint-boundary",
            "aria-label": "Faint boundary",
            style: "border: 1px solid #e8e8e8; background: transparent; padding: 8px; outline-offset: 2px;",
        }
    }
}

/// WCAG 1.4.3 over a positioned layer that covers part of the text: axe leaves
/// it undecided ("partially obscured"), the sweep measures it.
#[component]
pub fn LayeredText() -> Element {
    rsx! {
        div { style: "position: relative; padding: 8px; max-width: 240px;",
            span { style: "position: absolute; inset: 0 30% 0 0; background: #f4f4f4;" }
            span { id: "layered", style: "position: relative; color: #c0c0c0;", "Faint text over a layer" }
        }
    }
}

/// WCAG 2.4.7 Focus Visible: a control that removes its own indicator, and one whose
/// ring is on a child not marked `data-ring`, which the pass must not credit.
#[component]
pub fn NoFocusRing() -> Element {
    rsx! {
        document::Style {
            "#no-ring:focus, #no-ring:focus-visible, #unmarked-ring:focus-visible {{ outline: none; box-shadow: none; }} \
             #unmarked-ring:focus-visible > div {{ outline: 2px solid #000; }} \
             #zero-ring:focus, #zero-ring:focus-visible {{ outline: solid 0px #000; box-shadow: none; }}"
        }
        button { id: "no-ring", style: "padding: 8px 16px; border: 1px solid #888;", "No ring" }
        div { id: "unmarked-ring", tabindex: "0", role: "button",
            div { "Unmarked ring" }
        }
        // A solid outline 0px wide: the shorthand changes, nothing is drawn (todo 1797).
        button { id: "zero-ring", style: "padding: 8px 16px; border: 1px solid #888;", "Zero ring" }
    }
}

/// WCAG 1.4.11: a black ring at 10% alpha, a faint grey as it shows on white (todo 1796).
#[component]
pub fn TranslucentRing() -> Element {
    rsx! {
        document::Style {
            "#translucent-ring:focus-visible {{ outline: 2px solid rgba(0, 0, 0, 0.1); outline-offset: 2px; }}"
        }
        button { id: "translucent-ring", style: "padding: 8px 16px; border: 1px solid #888;", "Translucent ring" }
    }
}

/// WCAG 1.4.10: a fixed 360px box scrolls a 320px page sideways; the wide grid is 2D content
/// a test exempts (todo 1794).
#[component]
pub fn NoReflow() -> Element {
    rsx! {
        div { id: "too-wide", style: "width: 400px; height: 24px; background: #ddd;" }
        div { id: "wide-grid", style: "width: 600px; height: 24px; background: #eee;" }
    }
}

/// WCAG 2.4.7: an offset ring cut by its `overflow: hidden` parent (todo 1793).
#[component]
pub fn ClippedRing() -> Element {
    rsx! {
        document::Style { "#clipped-ring:focus-visible {{ outline: 2px solid #000; outline-offset: 2px; }}" }
        div { style: "overflow: hidden; display: inline-block; margin: 24px;",
            button { id: "clipped-ring", style: "display: block; padding: 8px 16px; border: 1px solid #888;", "Clipped ring" }
        }
    }
}

/// WCAG 2.4.11: a sticky bar pulled over the start of its scroller's content, where the
/// target sits; focus does not scroll a target already in the scrollport (todo 1793).
#[component]
pub fn CoveredFocus() -> Element {
    rsx! {
        document::Style { "#covered-focus:focus-visible {{ outline: 2px solid #000; outline-offset: 2px; }}" }
        div { style: "height: 160px; width: 240px; overflow: auto; border: 1px solid #888;",
            div { style: "position: sticky; top: 0; height: 64px; margin-bottom: -64px; background: #333; color: #fff;",
                "Sticky bar"
            }
            button { id: "covered-focus", style: "margin: 16px; padding: 8px 16px;", "Under the bar" }
            div { style: "height: 400px;" }
        }
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

/// An uncaught exception during mount: the window a recorder attached after navigation would miss.
#[component]
pub fn ConsoleError() -> Element {
    rsx! {
        document::Script { "throw new Error('fixture: deliberate mount-time error');" }
        p { "This fixture throws while mounting." }
    }
}

/// `use_field_frame`'s shape with a faint `[data-ring]` sibling but a strong `:focus-within` border.
/// The pass used to read only the input's ancestors and passed this (review 7, E4).
#[component]
pub fn FaintFieldRing() -> Element {
    rsx! {
        style { r#"
            #faint-frame {{ position: relative; border: 1px solid #888; background: #fff; padding: 8px; max-width: 240px; }}
            #faint-frame:focus-within {{ border-color: #1a1a8c; }}
            #faint-frame input {{ border: none; outline: none; font: inherit; }}
            #faint-frame [data-ring] {{ position: absolute; inset: -1px; pointer-events: none; }}
            #faint-frame input:focus-visible ~ [data-ring] {{ outline: 2px solid #eeeeee; outline-offset: 2px; }}
        "# }
        div { id: "faint-frame",
            input { id: "faint-input", "aria-label": "Faint ring" }
            span { "data-ring": true, "aria-hidden": "true" }
        }
    }
}

/// A page that warns while mounting. The console pass used to drop every
/// warning, so dioxus's scope warning (todo 283) could never fail a run.
#[component]
pub fn ConsoleWarning() -> Element {
    rsx! {
        document::Script { "console.warn('fixture: deliberate mount-time warning');" }
        p { "This fixture warns while mounting." }
    }
}

/// WCAG 2.5.8's spacing exception violated: two 20x20 targets edge to edge, centres 20px apart.
/// Undersized alone conforms when the 24px circles clear their neighbours.
#[component]
pub fn CrampedTargets() -> Element {
    rsx! {
        div { style: "display: flex; gap: 0;",
            button {
                id: "cramped-a",
                "aria-label": "Cramped A",
                style: "box-sizing: border-box; width: 20px; height: 20px; padding: 0; margin: 0; border: 1px solid #555;",
            }
            button {
                id: "cramped-b",
                "aria-label": "Cramped B",
                style: "box-sizing: border-box; width: 20px; height: 20px; padding: 0; margin: 0; border: 1px solid #555;",
            }
        }
        // Centres 65px apart, yet the bar lies inside the small button's circle (todo 1792).
        div { style: "display: flex; gap: 0; margin-top: 48px;",
            button {
                id: "beside-bar",
                "aria-label": "Beside a bar",
                style: "box-sizing: border-box; width: 10px; height: 10px; padding: 0; margin: 0; border: 1px solid #555;",
            }
            button {
                id: "bar",
                "aria-label": "Bar",
                style: "box-sizing: border-box; width: 120px; height: 12px; padding: 0; margin: 0; border: 1px solid #555;",
            }
        }
    }
}

/// APG roving tabindex violated: every item is its own tab stop.
/// Arrows, Home and End work, so the tab-stop count is the only thing `RovingTabindex` can object to.
#[component]
pub fn ManyTabStops() -> Element {
    const TABS: [&str; 3] = ["One", "Two", "Three"];
    let mut nodes: Signal<Vec<Option<std::rc::Rc<MountedData>>>> =
        use_signal(|| vec![None; TABS.len()]);

    rsx! {
        div { role: "tablist", "aria-label": "Broken strip",
            for (index , label) in TABS.iter().enumerate() {
                button {
                    key: "{index}",
                    role: "tab",
                    tabindex: "0",
                    onmounted: move |event: Event<MountedData>| {
                        nodes.write()[index] = Some(event.data());
                    },
                    onkeydown: move |event: Event<KeyboardData>| {
                        let target = match event.key() {
                            Key::ArrowRight => (index + 1) % TABS.len(),
                            Key::ArrowLeft => (index + TABS.len() - 1) % TABS.len(),
                            Key::Home => 0,
                            Key::End => TABS.len() - 1,
                            _ => return,
                        };
                        event.prevent_default();
                        let node = nodes.read().get(target).cloned().flatten();
                        if let Some(node) = node {
                            spawn(async move {
                                let _ = node.set_focus(true).await;
                            });
                        }
                    },
                    "{label}"
                }
            }
        }
    }
}

/// APG dialog: closes, but drops focus to `<body>` instead of returning it.
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

/// A dismissed panel hidden with `opacity: 0` only: still announced and tabbable.
/// An animated close without `inert` or `visibility: hidden` looks like this.
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

/// `aria-activedescendant` naming an element not in the DOM (todo 101).
/// Starts closed like a real combobox, or the contract fails earlier and proves nothing.
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

/// A combobox that opens but never sets `aria-activedescendant` (review 7, E2).
#[component]
pub fn MissingActiveDescendant() -> Element {
    rsx! { FakeCombobox { id: "missing", highlight: None } }
}

/// A combobox whose highlight names a real option and never moves.
#[component]
pub fn StaticHighlight() -> Element {
    rsx! { FakeCombobox { id: "static", highlight: Some("static-option-0") } }
}

/// A real combobox in every respect but its highlight, so the contract reaches the highlight checks.
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
