//! `Carousel`'s rendered contract.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Carousel};

/// The autoplay button is a toggle: one fixed name, with `aria-pressed` as the
/// state. A name that flipped to "Play" as well would read "Play, pressed".
#[test]
fn the_autoplay_toggle_has_a_fixed_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Offers",
                    autoplay: true,
                    slides: vec![rsx! { "a" }, rsx! { "b" }],
                }
            }
        }
    }

    let html = body(&render(app));
    assert!(
        html.contains(r#"aria-label="Pause slideshow" aria-pressed="false""#),
        "{html}"
    );
    assert!(!html.contains("Play slideshow"), "{html}");
}

/// The opening tag of the element whose markup contains `needle`, so an
/// attribute can be asserted on *that* element rather than anywhere in the page.
fn open_tag<'a>(html: &'a str, needle: &str) -> &'a str {
    let at = html
        .find(needle)
        .unwrap_or_else(|| panic!("no {needle} in {html}"));
    let start = html[..at].rfind('<').expect("an opening tag");
    let end = at + html[at..].find('>').expect("an unterminated tag");
    &html[start..=end]
}

/// The track's id: the first id in a carousel's markup is the track's.
fn track_id_of(html: &str) -> String {
    html.split_once(r#" id=""#)
        .map(|(_, rest)| rest.split('"').next().unwrap_or_default().to_string())
        .expect("the track carries an id")
}

fn six() -> Vec<Element> {
    (1..=6).map(|n| rsx! { "{n}" }).collect()
}

/// Mantine counts where the strip can rest, not slides: six three-up rest at
/// four places, so there are four dots and the status is out of four.
#[test]
fn the_status_and_the_dots_count_resting_positions() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel { aria_label: "Offers", per_view: 3.0, indicators: true, slides: six() }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("Slide 1 of 4"), "{html}");
    assert!(html.contains(r#"aria-label="Go to slide 4""#), "{html}");
    assert!(!html.contains(r#"aria-label="Go to slide 5""#), "{html}");
}

/// Every slide in view is one resting place, whichever slide the caller holds
/// - a lightbox's thumbnail strip opened on its last picture read "4 of 6".
#[test]
fn a_strip_whose_slides_all_fit_is_one_position() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Thumbnails",
                    per_view: 6.0,
                    align: "center",
                    index: Some(5),
                    controls: true,
                    indicators: true,
                    slides: six(),
                }
            }
        }
    }

    let html = body(&render(app));
    assert!(html.contains("Slide 1 of 1"), "{html}");
    assert!(html.contains(r#"aria-label="Go to slide 1""#), "{html}");
    assert!(!html.contains(r#"aria-label="Go to slide 2""#), "{html}");
}

/// The five-part DOM and the a11y wiring in one pass, because every part of it
/// is a projection of the same `(count, current)` pair.
#[test]
fn a_carousel_names_its_slides_and_points_its_controls_at_the_track() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    per_view: 3.0,
                    slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));
    let root = attributes_of(&html, "section");

    assert_eq!(root["role"], "region");
    assert_eq!(root["aria-roledescription"], "carousel");
    assert_eq!(root["aria-label"], "Photos");

    // Every slide is a named group, and none is hidden: in a real scroll
    // container an offscreen slide is still reachable.
    assert_eq!(html.matches(r#"aria-roledescription="slide""#).count(), 6);
    assert!(html.contains(r#"aria-label="1 of 6""#), "{html}");
    assert!(html.contains(r#"aria-label="6 of 6""#), "{html}");
    assert!(!html.contains("aria-hidden"), "{html}");

    // The controls name the element they scroll, and the track is the tab stop.
    // The track is the only element here that carries an id, and the controls
    // have to name that one rather than whatever `attributes_of` finds first.
    let track_id = track_id_of(&html);
    assert_eq!(
        html.matches(&format!(r#"aria-controls="{track_id}""#))
            .count(),
        2
    );
    // On the track's own tag: the current dot is a tab stop too, so a
    // page-wide `tabindex="0"` stayed green with the track taken out.
    let track = open_tag(&html, &format!(r#"id="{track_id}""#));
    assert!(track.contains(r#"tabindex="0""#), "{track}");
    // The track and the current dot, and nothing else.
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 2, "{html}");

    // Six slides three-up stop at index 3, so four dots, not six.
    assert_eq!(html.matches(r#"aria-label="Go to slide"#).count(), 4);
    // At the first slide the previous control is disabled but keeps its place
    // in the tab order.
    assert!(html.contains(r#"aria-disabled="true""#), "{html}");
    assert!(!html.contains("disabled=true"), "{html}");

    // The live region reads the settled position, politely, as one
    // utterance - out of the four places the strip can rest, as Mantine counts.
    assert!(html.contains(r#"role="status""#), "{html}");
    assert!(html.contains(r#"aria-live="polite""#), "{html}");
    assert!(html.contains("Slide 1 of 4"), "{html}");
}

/// Centred three-up over six slides reaches indices 1-4, not 0-3, so an index
/// seeded at 0 sits outside the window. Left there it strands the keyboard:
/// four dots and no tab stop among them, nothing marked current, and no scroll
/// at rest to correct any of it.
#[test]
fn an_index_outside_the_reachable_window_is_pulled_into_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    per_view: 3.0,
                    align: "center",
                    slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));

    // The track is the tab stop, and so is the current dot - on each one's
    // own tag. The track's alone kept a page-wide `tabindex="0"` green with
    // no tab stop among the dots, the very defect this test is named for.
    let track = open_tag(&html, &format!(r#"id="{}""#, track_id_of(&html)));
    assert!(track.contains(r#"tabindex="0""#), "{track}");
    let dot = open_tag(&html, r#"aria-current="true""#);
    assert!(
        dot.contains(r#"tabindex="0""#),
        "no tab stop in the strip: {dot}"
    );
    // Slide 1 is the one actually centred at rest, so it is the current one.
    let slide_one = html.find("slide 1").expect("slide 1");
    let current = html
        .find(r#"data-current="true""#)
        .expect("a current slide");
    let slide_two = html.find("slide 2").expect("slide 2");
    assert!(
        current < slide_one && slide_one < slide_two,
        "the current slide should be slide 1: {html}"
    );
}

/// The mount-time clamp speaks only to a controlled caller, which is the only
/// party that can be holding an index the component disagrees with.
/// `onindexchange` documents itself as a scroll, control, key, indicator or
/// autoplay event, and a clamp is none of those.
#[test]
fn an_uncontrolled_carousel_reports_no_index_change_on_mount() {
    #[component]
    fn Counted(controlled: bool) -> Element {
        let mut calls = use_signal(|| 0usize);

        rsx! {
            div { "calls: {calls}" }
            Carousel {
                aria_label: "Photos",
                per_view: 3.0,
                align: "center",
                index: controlled.then_some(0),
                onindexchange: move |_| calls += 1,
                slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
            }
        }
    }

    fn uncontrolled() -> Element {
        rsx! { LiberoProvider { Counted { controlled: false } } }
    }
    fn controlled() -> Element {
        rsx! { LiberoProvider { Counted { controlled: true } } }
    }

    // Both clamp - the window is 1..=4 either way.
    assert!(body(&render(uncontrolled)).contains("calls: 0"));
    assert!(body(&render(controlled)).contains("calls: 1"));
}

/// A controlled index that leaves the window *after* mount is clamped like one
/// that starts outside it, and the caller has to hear about it the same way -
/// or it goes on holding 5 while the carousel shows 4.
#[test]
fn a_controlled_index_pushed_out_of_the_window_later_is_reported_back() {
    #[component]
    fn Driven() -> Element {
        // In the window (1..=4) at mount, so the mount-time clamp is not what
        // this sees.
        let mut index = use_signal(|| 3usize);
        let mut reported = use_signal(Vec::<usize>::new);
        use_effect(move || index.set(5));

        rsx! {
            div { "holding: {index}, reported: {reported:?}" }
            Carousel {
                aria_label: "Photos",
                per_view: 3.0,
                align: "center",
                index: index(),
                onindexchange: move |next| {
                    reported.write().push(next);
                    index.set(next);
                },
                slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
            }
        }
    }

    fn app() -> Element {
        rsx! { LiberoProvider { Driven {} } }
    }

    // Its own dom rather than `render`, which stops after one pass: this
    // needs the effect, the re-render it causes, the carousel's answer and the
    // re-render after that.
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..6 {
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
    }
    let html = body(&dioxus_ssr::render(&dom));

    assert!(html.contains("holding: 4, reported: [4]"), "{html}");
}

/// The roving `tabindex` moves focus by looking the new dot up by id, so the
/// ids have to be on the dots and have to match what the lookup builds. If they
/// were not, focus would silently never move and only the comment would say
/// otherwise.
#[test]
fn every_indicator_carries_the_id_its_focus_lookup_targets() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    slides: (0..3).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));
    let track_id = track_id_of(&html);

    for index in 0..3 {
        let id = format!("{track_id}-indicator-{index}");
        assert!(html.contains(&format!(r#"id="{id}""#)), "no {id} in {html}");
    }
}

/// The clones are what a looping strip scrolls into past either edge. They
/// carry the same content, so they are hidden rather than announced twice.
#[test]
fn a_looping_carousel_clones_its_ends_and_hides_the_copies() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    r#loop: true,
                    slides: (0..4).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));

    // One clone at each end at one-up: six positions for four slides.
    assert_eq!(html.matches("<div>slide").count(), 6);
    assert_eq!(html.matches(r#"aria-hidden="true""#).count(), 2);
    // Only the four real slides are named and grouped.
    assert_eq!(html.matches(r#"aria-roledescription="slide""#).count(), 4);
    assert_eq!(html.matches(r#"aria-label="1 of 4""#).count(), 1);

    // The strip opens on the last slide (the leading clone) and the first
    // slide (the trailing one), in that order.
    let first = html.find("slide 3").expect("the leading clone");
    let second = html.find("slide 0").expect("the first real slide");
    assert!(first < second, "{html}");

    // No end to be at, so neither control is disabled and there is one dot per
    // real slide.
    assert!(!html.contains(r#"aria-disabled="true""#), "{html}");
    assert_eq!(html.matches(r#"aria-label="Go to slide"#).count(), 0);
}

/// A generic name beats none at all, so the theme's stands in - the warning is
/// what says to do better.
#[test]
fn an_unnamed_carousel_falls_back_to_the_theme_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel { slides: vec![rsx! { div { "one" } }] }
            }
        }
    }

    let root = attributes_of(&body(&render(app)), "section");

    assert_eq!(root["aria-label"], "Carousel");
}

/// No slides is not one slide: there is no position to announce, nothing to go
/// to and nothing to pause. The root stays so the caller's layout holds, but
/// it is neither a landmark nor a tab stop.
#[test]
fn an_empty_carousel_renders_no_chrome() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    indicators: true,
                    controls: true,
                    autoplay: true,
                    slides: vec![],
                }
            }
        }
    }

    let html = body(&render(app));

    assert!(html.contains("<section"), "{html}");
    assert!(!html.contains("Slide 1 of 1"), "{html}");
    assert!(!html.contains(r#"role="status""#), "{html}");
    assert!(!html.contains("Go to slide"), "{html}");
    assert!(!html.contains("Pause slideshow"), "{html}");
    assert!(!html.contains("Previous slide"), "{html}");
    assert!(!html.contains(r#"role="region""#), "{html}");
    assert!(!html.contains(r#"tabindex="0""#), "{html}");
}

/// `align` defaults to `Center` (Maintainer, 2026-09-19). Three-up over six
/// slides, centred rests on slides 1-4, so the one current at rest is the
/// second - start-aligned it would be the first.
#[test]
fn a_carousel_centres_its_slides_by_default() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Carousel {
                    aria_label: "Photos",
                    per_view: 3.0,
                    slides: (0..6).map(|i| rsx! { div { "slide {i}" } }).collect(),
                }
            }
        }
    }

    let html = body(&render(app));

    let current = open_tag(&html, r#"data-current="true""#);
    let after = &html[html.find(current).unwrap() + current.len()..];
    assert!(after.starts_with("<div>slide 1</div>"), "{html}");
}
