//! `use_floating_window`'s rendered contract: nothing until opened, then a
//! fixed, non-modal `role="dialog"` named by its title, with a focusable
//! move handle, a close button and, only when asked, a resize separator.

mod common;

use common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider, components::FloatingWindowOptions, hooks::use_floating_window, sx::sx,
};

fn options() -> FloatingWindowOptions {
    FloatingWindowOptions {
        title: Some("Inspector".into()),
        ..Default::default()
    }
}

#[component]
fn Opened(options: FloatingWindowOptions) -> Element {
    let window = use_floating_window(options, |_| rsx! { "window body" });
    use_hook(|| window.open());
    rsx! {}
}

#[component]
fn Closed() -> Element {
    let _window = use_floating_window(options(), |_| rsx! { "window body" });
    rsx! {}
}

fn plain_app() -> Element {
    rsx! { LiberoProvider { Opened { options: options() } } }
}

fn resizable_app() -> Element {
    rsx! {
        LiberoProvider {
            Opened { options: FloatingWindowOptions { resizable: true, ..options() } }
        }
    }
}

fn pinned_app() -> Element {
    rsx! {
        LiberoProvider {
            Opened { options: FloatingWindowOptions { pinned: true, ..options() } }
        }
    }
}

fn closed_app() -> Element {
    rsx! { LiberoProvider { Closed {} } }
}

/// The open tag holding `needle`.
fn tag_with(html: &str, needle: &str) -> std::collections::BTreeMap<String, String> {
    let body = body(html);
    let at = body
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle}:\n{html}"));
    let open = body[..at].rfind('<').unwrap();
    let tag = &body[open + 1..][..body[open + 1..].find([' ', '>']).unwrap()];
    attributes_of(&body[open..], tag)
}

#[test]
fn nothing_renders_until_opened() {
    let html = render(closed_app);
    assert!(!body(&html).contains("window body"), "{html}");
    assert!(!body(&html).contains("role=\"dialog\""), "{html}");
}

#[test]
fn an_open_window_is_a_named_non_modal_dialog() {
    let html = render(plain_app);
    let dialog = tag_with(&html, "role=\"dialog\"");
    assert_eq!(dialog["tabindex"], "-1", "{dialog:?}");
    assert!(!dialog.contains_key("aria-modal"), "{dialog:?}");
    let heading = tag_with(&html, ">Inspector<");
    assert_eq!(dialog["aria-labelledby"], heading["id"], "{html}");
    assert!(body(&html).contains("window body"), "{html}");
}

#[test]
fn the_window_sits_in_a_fixed_float() {
    let html = render(plain_app);
    let body = body(&html);
    let dialog_at = body.find("role=\"dialog\"").unwrap();
    let float_open = body[..dialog_at].rfind("<div").unwrap();
    let float_open = body[..float_open].rfind("<div").unwrap();
    let float = attributes_of(&body[float_open..], "div");
    let tokens: Vec<_> = float["data-state"].split(' ').collect();
    assert!(tokens.contains(&"fixed"), "{float:?}");
    assert!(tokens.contains(&"vertical-center"), "{float:?}");
}

/// A caller's `max_width` replaces the window's own `max-width`, so the
/// viewport cap has to sit where no caller `sx` reaches: on the `Float`.
#[test]
fn the_viewport_cap_survives_a_callers_max_width() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Opened {
                    options: FloatingWindowOptions {
                        sx: sx().max_width("40rem").into(),
                        ..options()
                    },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let dialog_at = body.find("role=\"dialog\"").unwrap();
    let float_open = body[..dialog_at].rfind("<div").unwrap();
    let float_open = body[..float_open].rfind("<div").unwrap();
    let float = attributes_of(&body[float_open..], "div");
    let rules: String = float["class"]
        .split(' ')
        .filter_map(|class| {
            let start = html.find(&format!(".{class}{{"))?;
            let rule = &html[start..];
            Some(rule[..rule.find('}')?].to_owned())
        })
        .collect();
    for declaration in [
        "max-width:100dvw",
        "max-height:100dvh",
        "flex-direction:column",
    ] {
        assert!(
            rules.replace(' ', "").contains(declaration),
            "{declaration} in {rules}"
        );
    }
}

#[test]
fn the_title_bar_is_the_keyboard_move_handle() {
    let html = render(plain_app);
    let handle = tag_with(&html, "data-window-handle");
    assert_eq!(handle["tabindex"], "0", "{handle:?}");
    assert_eq!(handle["role"], "group", "{handle:?}");
    assert_eq!(handle["aria-label"], "Move window", "{handle:?}");
    let close = tag_with(&html, "aria-label=\"Close\"");
    assert_eq!(close["type"], "button", "{close:?}");
}

#[test]
fn a_pinned_window_has_no_move_handle_stop() {
    let html = render(pinned_app);
    let handle = tag_with(&html, "data-window-handle");
    assert!(!handle.contains_key("tabindex"), "{handle:?}");
    assert!(!handle.contains_key("role"), "{handle:?}");
}

#[test]
fn only_a_resizable_window_has_a_separator() {
    assert!(!body(&render(plain_app)).contains("role=\"separator\""));
    let html = render(resizable_app);
    let separator = tag_with(&html, "role=\"separator\"");
    assert_eq!(separator["tabindex"], "0", "{separator:?}");
    assert_eq!(separator["aria-label"], "Resize window", "{separator:?}");
    assert!(separator.contains_key("aria-valuetext"), "{separator:?}");
}
