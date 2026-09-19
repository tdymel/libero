//! `Carousel`: a looping strip opened on slide N shows slide N (todo 925).

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
