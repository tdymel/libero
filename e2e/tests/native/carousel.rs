//! `Carousel`: a looping strip opened on slide N shows slide N (todo 925); a released
//! drag or a wheel rests on the nearest slide, as the web's scroll snap does.

use dioxus::prelude::*;
use e2e::native::{Key, Page, mount};
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

/// Todo 3120: the instant placement's `seam` comes down once the strip rests, or every
/// later step jumps instead of scrolling.
#[test]
fn a_looping_strip_lowers_the_seam_once_placed() {
    let mut page = mount(one_up);
    let lowered = page.wait_for(|page| {
        !page
            .attr(TRACK, "data-state")
            .is_some_and(|state| state.split(' ').any(|s| s == "seam"))
    });
    assert!(
        lowered,
        "the seam stayed up: {:?}",
        page.attr(TRACK, "data-state")
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

/// Todo 5287: Blitz places the logical inset, at the strip's end and beside Next's column.
#[test]
fn the_toggle_sits_at_the_inline_end() {
    let page = mount(autoplaying);
    let (toggle_x, _, toggle_width, _) = page.rect(TOGGLE);
    let (root_x, _, root_width, _) = page.rect("[aria-roledescription=carousel]");
    let gap = root_x + root_width - (toggle_x + toggle_width);
    assert!(
        (0.0..root_width / 3.0).contains(&gap),
        "{gap}px off the end of a {root_width}px strip"
    );
}

/// Todo 5286: a finger's first tap pauses on Blitz too, whose focus comes after the click (2394).
#[test]
fn a_first_tap_from_outside_the_carousel_pauses() {
    let mut page = mount(autoplaying);
    page.click("#before");
    let (x, y) = page.touch_down(TOGGLE);
    page.touch_up(x, y);
    assert_eq!(
        pressed(&page).as_deref(),
        Some("true"),
        "the first tap did not pause, focus on {}",
        page.focus_owner()
    );
}

/// Todo 3118 on Blitz: a mouse press released off the toggle leaves the next entry an entry;
/// its pointer type has to read as a mouse for the leave to clear it (5286).
#[test]
fn a_press_released_off_the_toggle_leaves_the_next_entry_an_entry() {
    let mut page = mount(autoplaying);
    page.click("#before");
    let centre = |page: &Page, selector: &str| {
        let (x, y, width, height) = page.rect(selector);
        ((x + width / 2.0) as f32, (y + height / 2.0) as f32)
    };
    let (x, y) = centre(&page, TOGGLE);
    page.press_at(x, y);
    let (away_x, away_y) = centre(&page, "#before");
    page.move_to(away_x, away_y);
    page.release_at(away_x, away_y);
    assert_eq!(
        pressed(&page).as_deref(),
        Some("false"),
        "the press clicked"
    );

    page.press(Key::Named(NamedKey::Tab));
    assert!(
        page.is_focused(TOGGLE),
        "Tab did not reach the toggle, focus on {}",
        page.focus_owner()
    );
    assert_eq!(
        pressed(&page).as_deref(),
        Some("true"),
        "focus entering after the press did not pause"
    );
}

const STATUS: &str = "[aria-roledescription=carousel] [role=status]";

fn live(page: &Page) -> Option<String> {
    page.attr(STATUS, "aria-live")
}

/// Todo 4729: Blitz's mouse `pointerenter` pauses, its `pointerleave` resumes, and
/// the toggle never presses; a rotating carousel keeps its status quiet.
#[test]
fn a_mouse_hover_pauses_and_leaving_resumes() {
    let mut page = mount(autoplaying);
    assert_eq!(live(&page).as_deref(), Some("off"), "not rotating at rest");
    page.hover(TRACK);
    assert!(
        page.wait_for(|page| live(page).as_deref() == Some("polite")),
        "hover did not pause, status {:?}",
        live(&page)
    );
    page.hover("#before");
    assert!(
        page.wait_for(|page| live(page).as_deref() == Some("off")),
        "leaving did not resume, status {:?}",
        live(&page)
    );
    assert_eq!(
        pressed(&page).as_deref(),
        Some("false"),
        "hover pressed Pause"
    );
}

/// Todo 2634: a Tab from the Play toggle to the track is a move within the carousel, no
/// new focus entry that pauses the rotation again.
#[test]
fn a_tab_within_the_carousel_keeps_it_playing() {
    let mut page = mount(autoplaying);
    page.click("#before");
    page.click(TOGGLE);
    page.click(TOGGLE);
    assert_eq!(
        pressed(&page).as_deref(),
        Some("false"),
        "Play did not play"
    );

    page.press(Key::Named(NamedKey::Tab));
    assert!(
        page.is_focused(TRACK),
        "Tab did not reach the track, focus on {}",
        page.focus_owner()
    );
    assert_eq!(
        pressed(&page).as_deref(),
        Some("false"),
        "a Tab within the carousel paused it"
    );
}
