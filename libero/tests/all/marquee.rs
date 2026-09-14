//! `Marquee`'s rendered contract: `repeat` copies of which only the first is
//! announced or reachable, a pause toggle, and a reduced-motion arm that sits
//! where it can win.

use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Marquee};

fn marquee_class(html: &str) -> String {
    classes_of(&body(html), "div")
        .first()
        .expect("a class on the marquee")
        .clone()
}

fn groups(html: &str) -> Vec<&str> {
    html.split(r#"<div data-slot="group""#).skip(1).collect()
}

/// Four copies by default. Every one after the first is hidden from the
/// accessibility tree *and* inert - `aria-hidden` alone would leave a link in
/// it tabbable.
#[test]
fn only_the_first_copy_is_announced_or_reachable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { a { href: "#", "Link" } } }
        }
    }

    let html = body(&render(app));
    let groups = groups(&html);
    assert_eq!(groups.len(), 4, "{html}");
    assert!(groups[0].starts_with('>'), "{html}");
    for copy in &groups[1..] {
        assert!(
            copy.starts_with(r#" aria-hidden="true" inert=true>"#),
            "{html}"
        );
    }
    assert_eq!(html.matches(">Link</a>").count(), 4, "{html}");
    assert!(
        html.contains(r#"data-state="horizontal""#),
        "horizontal and running by default: {html}"
    );
}

/// One copy has nothing to restart onto, and zero would divide by zero in
/// the shift - so two is the floor.
#[test]
fn repeat_has_a_floor_of_two() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Marquee { repeat: 0u8, "x" }
                Marquee { repeat: 1u8, "y" }
                Marquee { repeat: 6u8, "z" }
            }
        }
    }

    let html = body(&render(app));
    assert_eq!(html.matches(">x</div>").count(), 2, "{html}");
    assert_eq!(html.matches(">y</div>").count(), 2, "{html}");
    assert_eq!(html.matches(">z</div>").count(), 6, "{html}");
    assert!(html.contains("--lsx-marquee-repeat:2;"), "{html}");
    assert!(html.contains("--lsx-marquee-repeat:6;"), "{html}");
}

/// The toggle's name stays put and `aria-pressed` carries the state, so it is
/// never read as "Play, pressed".
#[test]
fn the_pause_toggle_is_a_pressed_button_with_a_fixed_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { "x" } }
        }
    }

    let attributes = attributes_of(&body(&render(app)), "button");
    assert_eq!(attributes["type"], "button", "{attributes:?}");
    assert_eq!(attributes["aria-label"], "Pause", "{attributes:?}");
    assert_eq!(attributes["aria-pressed"], "false", "{attributes:?}");
    assert_eq!(attributes["data-slot"], "pause", "{attributes:?}");
}

/// A controlled `paused` drives both the animation and the toggle.
#[test]
fn a_controlled_pause_reaches_the_state_and_the_toggle() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { paused: true, onpausechange: move |_| {}, "x" } }
        }
    }

    let html = body(&render(app));
    assert!(html.contains(r#"data-state="horizontal paused""#), "{html}");
    assert_eq!(attributes_of(&html, "button")["aria-pressed"], "true");
}

#[test]
fn the_pause_control_can_be_turned_off() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { pause_control: false, paused: true, "x" } }
        }
    }

    let html = body(&render(app));
    assert!(!html.contains("<button"), "{html}");
    assert!(html.contains("paused"), "{html}");
}

/// The shift reads the per-instance gap and copy count, so the three can
/// never disagree. A duration prop writes the override twin; the theme's
/// own sits on `:root`.
#[test]
fn the_shift_is_one_copy_plus_one_gap() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Marquee { orientation: "vertical", gap: "lg", duration: 25_000u32, "x" }
            }
        }
    }

    let html = render(app);
    let class = marquee_class(&html);
    let style = attributes_of(&body(&html), "div")["style"].clone();
    assert!(
        style.contains("--lsx-marquee-gap:var(--lsx-spacing-lg);"),
        "{style}"
    );
    assert!(
        style.contains("--lsx-marquee-duration-override:25000ms;"),
        "{style}"
    );
    assert!(html.contains("--lsx-marquee-duration:40000ms;"), "{html}");
    assert!(
        html.contains(&format!(
            r#".{class}[data-state~="vertical"]{{overflow-y:clip;min-height:0;--lsx-marquee-shift:translateY(calc((-100% - var(--lsx-marquee-gap)) / var(--lsx-marquee-repeat)));}}"#
        )),
        "{html}"
    );
    assert!(
        html.contains("@keyframes lsx-marquee{to{transform:var(--lsx-marquee-shift);}}"),
        "{html}"
    );
}

/// Each guard is the selector it undoes, inside the media block: a bare
/// `@media` rule would lose on specificity. What is left is one copy in a
/// strip the reader scrolls, with no fade over it and no toggle for a
/// motion that is not there.
#[test]
fn reduced_motion_leaves_one_scrollable_copy() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { fade_edges: true, "x" } }
        }
    }

    let html = render(app);
    let class = marquee_class(&html);
    let media = "@media (prefers-reduced-motion: reduce){";
    for rule in [
        format!(r#".{class}[data-state~="horizontal"]{{overflow-x:auto;}}"#),
        format!(".{class} > [data-slot='track']{{animation:none;}}"),
        format!(
            ".{class} > [data-slot='track'] > [data-slot='group']:not(:first-child){{display:none;}}"
        ),
        format!(".{class} > [data-slot='pause']{{display:none;}}"),
        format!(
            r#".{class}[data-state~="fade-edges"]::before, .{class}[data-state~="fade-edges"]::after{{display:none;}}"#
        ),
    ] {
        assert!(html.contains(&format!("{media}{rule}")), "{rule} in {html}");
    }
}

/// The fade is `Paper`'s surface colour, not a literal white.
#[test]
fn the_fade_reads_the_paper_token() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { fade_edges: true, "x" } }
        }
    }

    let html = render(app);
    assert!(
        html.contains(
            "background:linear-gradient(to right, var(--lsx-paper-background), transparent);"
        ),
        "{html}"
    );
    assert!(body(&html).contains(r#"data-state="horizontal fade-edges""#));
}

/// Keyboard focus inside the content stops it at its start, with no opt-in: a
/// focused link must not drift away or freeze out of view. Only the track's
/// focus - the toggle is outside it.
#[test]
fn focus_inside_the_track_stops_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { Marquee { a { href: "#", "Link" } } }
        }
    }

    let html = render(app);
    let class = marquee_class(&html);
    let rule = format!(".{class} > [data-slot='track']:focus-within{{animation:none;}}");
    assert!(html.contains(&rule), "{rule} in {html}");
}
