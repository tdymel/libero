//! `Carousel`: a looping strip opened on slide N shows slide N (todo 925); a released
//! drag or a wheel rests on the nearest slide, as the web's scroll snap does.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Carousel, Text};

const TRACK: &str = "[aria-roledescription=carousel] [role=group][aria-describedby]";
const CURRENT: &str = "[aria-roledescription=slide][data-current]";

fn slides() -> Vec<Element> {
    (1..=5).map(|n| rsx! { Text { "Slide {n}" } }).collect()
}

fn one_up() -> Element {
    rsx! {
        div { style: "width: 400px; padding: 16px;",
            Carousel {
                aria_label: "Looping",
                r#loop: true,
                index: 3usize,
                slides: slides(),
            }
        }
    }
}

fn draggable() -> Element {
    rsx! {
        div { style: "padding: 16px 300px;",
            div { style: "width: 400px;",
                Carousel { aria_label: "Dragged", draggable: true, slides: slides() }
            }
        }
    }
}

fn reporting() -> Element {
    let mut reported = use_signal(|| None::<usize>);
    rsx! {
        div { style: "width: 400px; padding: 16px;",
            Carousel {
                aria_label: "Wheeled",
                slides: slides(),
                onindexchange: move |index| reported.set(Some(index)),
            }
            span { id: "reported", "{reported:?}" }
        }
    }
}

fn three_up() -> Element {
    rsx! {
        div { style: "width: 600px; padding: 16px;",
            Carousel {
                aria_label: "Looping",
                r#loop: true,
                per_view: 3.0,
                index: 4usize,
                slides: slides(),
            }
        }
    }
}

/// The current slide's left edge against the track's, once the placement's
/// retries have run.
fn offset(page: &mut Page) -> f64 {
    page.wait_for(|page| {
        page.exists(CURRENT) && (page.rect(CURRENT).0 - page.rect(TRACK).0).abs() < 1.0
    });
    page.rect(CURRENT).0 - page.rect(TRACK).0
}

#[test]
fn a_looping_strip_opens_on_its_index() {
    let mut page = mount(one_up);
    assert_eq!(page.text(CURRENT), "Slide 4");
    let offset = offset(&mut page);
    assert!(
        offset.abs() < 1.0,
        "slide 4 sits {offset}px off the track:\n{}",
        page.tree()
    );
}

#[test]
fn a_three_up_looping_strip_opens_on_its_last_slide() {
    let mut page = mount(three_up);
    assert_eq!(page.text(CURRENT), "Slide 5");
    page.wait_for(|page| {
        let (slide, track) = (page.rect(CURRENT), page.rect(TRACK));
        slide.0 >= track.0 - 1.0 && slide.0 + slide.2 <= track.0 + track.2 + 1.0
    });
    let (slide, track) = (page.rect(CURRENT), page.rect(TRACK));
    assert!(
        slide.0 >= track.0 - 1.0 && slide.0 + slide.2 <= track.0 + track.2 + 1.0,
        "slide 5 at {slide:?} is outside the track {track:?}:\n{}",
        page.tree()
    );
}

/// Blitz has no scroll snap: without a settle of libero's own the strip stayed
/// wherever the pointer let go, between two slides.
#[test]
fn a_released_drag_rests_on_the_nearest_slide() {
    let mut page = mount(draggable);
    page.drag(TRACK, -300.0, 0.0);
    page.wait_for(|page| page.text(CURRENT) == "Slide 2");
    let off = offset(&mut page);
    assert!(
        off.abs() < 1.0,
        "a drag past half a slide left slide 2 {off}px off the track:\n{}",
        page.tree()
    );

    page.drag(TRACK, 100.0, 0.0);
    page.wait_for(|page| (page.rect(CURRENT).0 - page.rect(TRACK).0).abs() < 1.0);
    assert_eq!(page.text(CURRENT), "Slide 2", "a short drag moved on");
    let off = offset(&mut page);
    assert!(
        off.abs() < 1.0,
        "a short drag left slide 2 {off}px off the track:\n{}",
        page.tree()
    );
}

/// Blitz fires no `scrollend` (todo 949): a wheeled strip stayed between two
/// slides and never told `onindexchange`.
#[test]
fn a_wheeled_strip_rests_on_a_slide_and_reports_it() {
    let mut page = mount(reporting);
    page.hover(TRACK);
    page.wheel_x(TRACK, 250.0);
    assert!(
        page.wait_for(|page| page.text("#reported") == "Some(1)"),
        "the wheel reported {}:\n{}",
        page.text("#reported"),
        page.tree()
    );
    assert_eq!(page.text(CURRENT), "Slide 2");
    let off = offset(&mut page);
    assert!(
        off.abs() < 1.0,
        "a wheel past half a slide left slide 2 {off}px off the track:\n{}",
        page.tree()
    );
}

fn autoplaying() -> Element {
    rsx! {
        button { id: "before", "Before" }
        Carousel {
            aria_label: "Photos",
            autoplay: true,
            autoplay_delay: 60_000,
            slides: slides(),
        }
    }
}

const TOGGLE: &str = "[aria-pressed]";

fn pressed(page: &Page) -> Option<String> {
    page.attr(TOGGLE, "aria-pressed")
}

/// Todo 2394: a Play click whose focus entry Blitz reports late is not paused again by it.
#[test]
fn a_play_click_from_outside_the_carousel_keeps_it_playing() {
    let mut page = mount(autoplaying);
    page.click("#before");
    page.click(TOGGLE);
    assert_eq!(
        pressed(&page).as_deref(),
        Some("true"),
        "Pause did not pause"
    );
    page.click("#before");
    assert!(
        page.is_focused("#before"),
        "focus is on {}",
        page.focus_owner()
    );

    page.click(TOGGLE);
    assert_eq!(
        pressed(&page).as_deref(),
        Some("false"),
        "Play was paused again, focus on {}",
        page.focus_owner()
    );
}
