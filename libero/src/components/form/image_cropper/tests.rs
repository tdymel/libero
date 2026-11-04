use super::crop::{CropRect, Grip, Reach};
use super::image_cropper::{min_side, press_message, slider_zoom, spoken_percent, value_text};
use super::{ImageCropper, ImageCropperControls};
use crate::localization::{Formats, ImageCropperLabels};
use crate::platform::image::Backdrop;

fn rect(x: f64, y: f64, width: f64, height: f64) -> CropRect {
    CropRect {
        x,
        y,
        width,
        height,
    }
}

fn close(a: CropRect, b: CropRect) -> bool {
    [
        (a.x, b.x),
        (a.y, b.y),
        (a.width, b.width),
        (a.height, b.height),
    ]
    .iter()
    .all(|(a, b)| (a - b).abs() < 1e-9)
}

#[track_caller]
fn assert_close(a: CropRect, b: CropRect) {
    assert!(close(a, b), "{a:?} != {b:?}");
}

#[test]
fn the_largest_box_is_centred_at_the_ratio() {
    assert_eq!(CropRect::largest(None), CropRect::FULL);
    assert_close(CropRect::largest(Some(2.0)), rect(0.0, 0.25, 1.0, 0.5));
    assert_close(CropRect::largest(Some(0.5)), rect(0.25, 0.0, 0.5, 1.0));
}

#[test]
fn a_pinch_scales_the_box_around_its_centre_inside_the_image() {
    let start = rect(0.2, 0.3, 0.4, 0.2);
    assert_close(start.scaled(1.5, 0.05), rect(0.1, 0.25, 0.6, 0.3));
    // Stopped at the image's size and shifted back in, the shape kept.
    assert_close(
        rect(0.6, 0.0, 0.4, 0.2).scaled(4.0, 0.05),
        rect(0.0, 0.0, 1.0, 0.5),
    );
    // Stopped at `min` on the shorter side.
    assert_close(start.scaled(0.01, 0.05), rect(0.35, 0.375, 0.1, 0.05));
}

/// Todo 2436: a non-finite edge read `aria-valuenow="NaN"`, so the box is not used.
#[test]
fn a_box_with_a_non_finite_edge_is_not_finite() {
    assert!(CropRect::FULL.is_finite());
    for bad in [f64::NAN, f64::INFINITY] {
        assert!(!rect(0.1, 0.1, bad, 0.5).is_finite());
        assert!(!rect(bad, 0.1, 0.5, 0.5).is_finite());
    }
}

#[test]
fn an_unset_box_starts_at_four_fifths_of_the_largest() {
    assert_close(CropRect::starting(None), rect(0.1, 0.1, 0.8, 0.8));
    assert_close(CropRect::starting(Some(0.5)), rect(0.3, 0.1, 0.4, 0.8));
    // A 4:3 crop of a 16:9 picture, in whole percent.
    assert_close(CropRect::starting(Some(0.75)), rect(0.2, 0.1, 0.6, 0.8));
}

#[test]
fn an_unset_locked_box_starts_on_its_ratio() {
    // 3:2 on an 800x600 picture: whole percent would give 0.80 by 0.71.
    let start = CropRect::starting(Some(1.125));
    assert!(
        (start.width / start.height - 1.125).abs() < 1e-9,
        "{start:?}"
    );
    assert!(
        (start.width - 0.8).abs() < 1e-9 && start.x == 0.1,
        "{start:?}"
    );
}

#[test]
fn a_move_stops_at_the_edges() {
    let start = rect(0.1, 0.1, 0.5, 0.5);
    assert_close(start.moved(0.2, -0.05), rect(0.3, 0.05, 0.5, 0.5));
    assert_close(start.moved(1.0, -1.0), rect(0.5, 0.0, 0.5, 0.5));
}

#[test]
fn a_free_corner_moves_its_two_edges_only() {
    let start = rect(0.2, 0.2, 0.4, 0.4);
    let grown = start.resized(Grip::SouthEast, 0.1, 0.2, None, 0.05);
    assert_close(grown, rect(0.2, 0.2, 0.5, 0.6));
    let shrunk = start.resized(Grip::NorthWest, 0.1, -0.1, None, 0.05);
    assert_close(shrunk, rect(0.3, 0.1, 0.3, 0.5));
}

#[test]
fn a_free_edge_stops_at_the_image_and_at_the_minimum() {
    let start = rect(0.2, 0.2, 0.4, 0.4);
    assert_close(
        start.resized(Grip::East, 1.0, 0.0, None, 0.05),
        rect(0.2, 0.2, 0.8, 0.4),
    );
    assert_close(
        start.resized(Grip::North, 0.0, 1.0, None, 0.05),
        rect(0.2, 0.55, 0.4, 0.05),
    );
}

#[test]
fn a_locked_corner_keeps_the_ratio_from_the_opposite_corner() {
    let start = rect(0.2, 0.2, 0.4, 0.2);
    let grown = start.resized(Grip::SouthEast, 0.2, 0.0, Some(2.0), 0.05);
    assert_close(grown, rect(0.2, 0.2, 0.6, 0.3));
    // A key moves one axis: the other must not veto it.
    let narrowed = start.resized(Grip::NorthWest, 0.0, 0.05, Some(2.0), 0.05);
    assert_close(narrowed, rect(0.3, 0.25, 0.3, 0.15));
}

#[test]
fn a_locked_box_stops_where_either_side_meets_the_image() {
    let start = rect(0.2, 0.6, 0.2, 0.2);
    let grown = start.resized(Grip::SouthEast, 0.5, 0.5, Some(1.0), 0.05);
    assert_close(grown, rect(0.2, 0.6, 0.4, 0.4));
}

#[test]
fn a_locked_edge_grows_centred_across_it() {
    let start = rect(0.4, 0.4, 0.2, 0.2);
    let grown = start.resized(Grip::East, 0.2, 0.0, Some(1.0), 0.05);
    assert_close(grown, rect(0.4, 0.3, 0.4, 0.4));
}

#[test]
fn pan_mode_holds_the_frame_and_scales_the_image_under_it() {
    let start = CropRect::starting(Some(1.0));
    assert_close(start.frame(), start);
    let (scale, x, y) = start.image_transform();
    assert!((scale - 1.0).abs() < 1e-9 && x.abs() < 1e-9 && y.abs() < 1e-9);
    // Half the crop: the image doubles, the crop's corner on the frame's.
    let half = rect(0.2, 0.3, 0.4, 0.4);
    let (scale, x, y) = half.image_transform();
    assert!((scale - 2.0).abs() < 1e-9, "{scale}");
    assert!((x - (0.1 - 0.4)).abs() < 1e-9 && (y - (0.1 - 0.6)).abs() < 1e-9);
}

#[test]
fn a_pan_moves_the_crop_against_the_drag_inside_the_image() {
    let crop = rect(0.2, 0.2, 0.4, 0.4);
    // The frame is 0.8 wide, the crop half that: the crop moves half the drag.
    assert_close(crop.panned(0.1, -0.2), rect(0.15, 0.3, 0.4, 0.4));
    assert_close(crop.panned(1.0, 1.0), rect(0.0, 0.0, 0.4, 0.4));
}

#[test]
fn a_zoom_keeps_the_pinched_point_under_the_fingers() {
    let crop = rect(0.2, 0.2, 0.4, 0.4);
    // Twice the zoom around the frame's centre: half the crop, same centre.
    assert_close(
        crop.zoomed(2.0, (0.5, 0.5), (0.5, 0.5), 0.05),
        rect(0.3, 0.3, 0.2, 0.2),
    );
    // Zoomed out past the image: the crop stops at its whole height.
    assert_close(
        crop.zoomed(0.1, (0.5, 0.5), (0.5, 0.5), 0.05),
        rect(0.0, 0.0, 1.0, 1.0),
    );
    // Around the frame's top-left corner: that image point stays put.
    assert_close(
        crop.zoomed(2.0, (0.1, 0.1), (0.1, 0.1), 0.05),
        rect(0.2, 0.2, 0.2, 0.2),
    );
}

#[test]
fn home_and_end_take_the_keyboard_value_to_its_ends() {
    let crop = rect(0.2, 0.3, 0.4, 0.4);
    // The box's value is x: it can travel to either edge of the image.
    assert_close(
        crop.extreme(Grip::Move, false, false, None, 0.05),
        rect(0.0, 0.3, 0.4, 0.4),
    );
    assert_close(
        crop.extreme(Grip::Move, true, false, None, 0.05),
        rect(0.6, 0.3, 0.4, 0.4),
    );
    // An east corner's value is the width, kept to `min` and the image.
    let (least, greatest) = crop.range(Grip::SouthEast, false, None, 0.05);
    assert!((least - 0.05).abs() < 1e-9 && (greatest - 0.8).abs() < 1e-9);
    // A locked ratio follows the width.
    assert_close(
        rect(0.2, 0.2, 0.4, 0.4).extreme(Grip::SouthEast, true, false, Some(1.0), 0.05),
        rect(0.2, 0.2, 0.8, 0.8),
    );
}

/// Todo 4321: every slider's ArrowRight raises its value and Home takes it to its least.
#[test]
fn arrow_right_raises_each_slider_value_and_home_is_its_least() {
    let crop = rect(0.2, 0.3, 0.4, 0.4);
    // The corners, the keyboard's sliders.
    for grip in Grip::HANDLES[..4].iter().copied() {
        for ratio in [None, Some(1.0)] {
            let value = crop.value(grip, false);
            let right = crop
                .resized(grip, 0.05, 0.0, ratio, 0.05)
                .value(grip, false);
            assert!(right > value, "{grip:?} {ratio:?}: {value} -> {right}");
            let (least, greatest) = crop.range(grip, false, ratio, 0.05);
            assert!(least < value && value < greatest, "{grip:?} {ratio:?}");
            let home = crop.extreme(grip, false, false, ratio, 0.05);
            assert!((home.value(grip, false) - least).abs() < 1e-9, "{grip:?}");
        }
    }
    // A west corner reports its left edge: Home is the image's edge, End leaves `min`.
    let (least, greatest) = crop.range(Grip::NorthWest, false, None, 0.05);
    assert!(least.abs() < 1e-9 && (greatest - 0.55).abs() < 1e-9);

    // Pan mode reports the image's offset, which ArrowRight drags right.
    let value = crop.value(Grip::Move, true);
    assert!(crop.panned(0.05, 0.0).value(Grip::Move, true) > value);
    let (least, greatest) = crop.range(Grip::Move, true, None, 0.05);
    assert!(
        least < value && value < greatest,
        "{least} {value} {greatest}"
    );
    assert_close(
        crop.extreme(Grip::Move, false, true, None, 0.05),
        rect(0.6, 0.3, 0.4, 0.4),
    );
}

/// Todo 4323: a `min_size` of 0 let the smallest box be empty, so the zoom slider topped out at 1x.
#[test]
fn a_zero_min_size_still_lets_the_zoom_go_past_one() {
    assert_eq!(min_side(None), 0.05);
    for bad in [Some(0.0), Some(-1.0), Some(f64::NAN)] {
        let min = min_side(bad);
        assert!(min > 0.0, "{bad:?} gave {min}");
    }
    assert_eq!(min_side(Some(f64::NAN)), 0.05);
    let crop = CropRect::starting(Some(1.0));
    let most = crop.scaled(0.0, min_side(Some(0.0))).image_transform().0;
    assert!(most > 100.0, "the zoom stopped at {most}");
}

/// Todo 4322: a move or resize button says the new box, and nothing when the box stayed put.
#[test]
fn a_button_press_says_the_new_box_and_a_no_op_says_nothing() {
    let words = ImageCropperLabels::ENGLISH;
    let crop = rect(0.0, 0.1, 0.4, 0.5);
    assert_eq!(
        press_message(
            &words,
            &Formats::AMERICAN,
            false,
            crop,
            crop.moved(-0.05, 0.0)
        ),
        None
    );
    let moved = crop.moved(0.05, 0.0);
    let said = press_message(&words, &Formats::AMERICAN, false, crop, moved);
    assert_eq!(said.as_deref(), Some("40% by 50%, at 5%, 10%"));
    assert_eq!(
        said,
        Some(value_text(&words, &Formats::AMERICAN, false, moved))
    );
    // Five 5% steps up leave a rounding error the sixth, at the edge, takes away.
    let top = (0..5).fold(rect(0.3, 0.25, 0.5, 0.5), |at, _| at.moved(0.0, -0.05));
    assert_eq!(
        press_message(
            &words,
            &Formats::AMERICAN,
            false,
            top,
            top.moved(0.0, -0.05)
        ),
        None
    );
    // Pan mode says the zoom too.
    let zoomed = crop.zoomed(2.0, (0.5, 0.5), (0.5, 0.5), 0.05);
    let said = press_message(&words, &Formats::AMERICAN, true, crop, zoomed).unwrap();
    assert!(said.contains("zoom"), "{said}");
}

/// Todo 5558: German writes its percent with a no-break space, from the formats.
#[test]
fn the_box_text_follows_the_formats_percent() {
    let crop = rect(0.05, 0.1, 0.4, 0.5);
    let german = value_text(&ImageCropperLabels::GERMAN, &Formats::GERMAN, false, crop);
    assert_eq!(german, "40\u{a0}% mal 50\u{a0}%, bei 5\u{a0}%, 10\u{a0}%");
}

/// Todo 5579: a spoken percent has no group separator.
#[test]
fn a_spoken_percent_has_no_group_separator() {
    assert_eq!(spoken_percent(&Formats::AMERICAN, 1591.2), "1591%");
    assert_eq!(spoken_percent(&Formats::GERMAN, 1591.0), "1591\u{a0}%");
}

/// The region is there before any press: one inserted with its text is not read.
#[test]
fn the_cropper_renders_an_empty_status_region() {
    use dioxus::prelude::*;
    let mut dom = VirtualDom::new(|| {
        rsx! {
            crate::LiberoProvider {
                super::ImageCropper { src: "/photo.jpg", alt: "Photo", onchange: |_| {} }
            }
        }
    });
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(html.contains(r#"role="status""#), "{html}");
}

#[test]
fn pixels_round_and_stay_inside_the_image() {
    let pixels = rect(0.5, 0.5, 0.6, 0.6).to_pixels(101, 100);
    assert_eq!((pixels.x, pixels.y), (51, 50));
    assert_eq!((pixels.width, pixels.height), (50, 50));
}

#[test]
fn the_zoom_slider_says_the_box_zoom_its_snapped_value_stands_for() {
    // 266.5%: the slider snaps ln to 0.98, whose exp alone reads 266% beside the box's 267%.
    let exact = 2.665;
    let percent = |zoom: f64| (zoom * 100.0).round();
    assert_eq!(percent(0.98_f64.exp()), 266.0);
    assert_eq!(percent(slider_zoom(0.98, exact)), 267.0);
    // A value a step away is its own zoom.
    assert_eq!(slider_zoom(0.99, exact), 0.99_f64.exp());
}

#[derive(Clone, Copy, Default, PartialEq)]
struct Setup {
    controls: ImageCropperControls,
    pan: bool,
    overhang: f64,
    backdrop: Backdrop,
}

fn render(setup: Setup) -> String {
    use dioxus::prelude::*;
    #[component]
    fn Host(setup: Setup) -> Element {
        rsx! {
            crate::LiberoProvider {
                ImageCropper {
                    src: "/photo.jpg",
                    alt: "Photo",
                    controls: setup.controls,
                    pan: setup.pan,
                    overhang: setup.overhang,
                    backdrop: setup.backdrop,
                    onchange: |_| {},
                }
            }
        }
    }
    let mut dom = VirtualDom::new_with_props(Host, HostProps { setup });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn no_overhang_is_the_plain_geometry() {
    let reach = Reach::new(0.0);
    let start = rect(0.2, 0.2, 0.4, 0.4);
    assert_eq!(
        reach.resized(start, Grip::SouthEast, (0.1, 0.2), None, 0.05),
        start.resized(Grip::SouthEast, 0.1, 0.2, None, 0.05)
    );
    assert_eq!(reach.scaled(start, 1.5, 0.05), start.scaled(1.5, 0.05));
    // Not a number, or out of range: none, or all of it.
    assert_eq!(Reach::new(f64::NAN).get(), 0.0);
    assert_eq!(Reach::new(-1.0).get(), 0.0);
    assert_eq!(Reach::new(5.0).get(), 1.0);
}

#[test]
fn an_overhang_lets_the_box_move_past_the_edge_by_that_much() {
    let reach = Reach::new(0.25);
    let start = rect(0.1, 0.1, 0.5, 0.5);
    assert_close(
        reach.resized(start, Grip::Move, (1.0, -1.0), None, 0.05),
        rect(0.75, -0.25, 0.5, 0.5),
    );
}

#[test]
fn an_overhang_lets_the_box_grow_past_the_picture() {
    let reach = Reach::new(0.25);
    let start = rect(0.2, 0.2, 0.4, 0.4);
    assert_close(
        reach.scaled(start, 10.0, 0.05),
        rect(-0.25, -0.25, 1.5, 1.5),
    );
    // An edge grip reaches the room's end, a locked corner keeps its ratio there.
    assert_close(
        reach.resized(start, Grip::East, (2.0, 0.0), None, 0.05),
        rect(0.2, 0.2, 1.05, 0.4),
    );
    let locked = reach.resized(start, Grip::NorthWest, (-2.0, -2.0), Some(1.0), 0.05);
    assert!((locked.width - locked.height).abs() < 1e-9, "{locked:?}");
    assert!(
        locked.x >= -0.25 - 1e-9 && locked.y >= -0.25 - 1e-9,
        "{locked:?}"
    );
}

#[test]
fn a_pan_can_zoom_out_to_show_the_overhang() {
    let reach = Reach::new(0.5);
    let crop = CropRect::starting(Some(1.0));
    let out = reach.zoomed(crop, 0.01, (0.5, 0.5), (0.5, 0.5), 0.05);
    assert_close(out, rect(-0.5, -0.5, 2.0, 2.0));
    assert!(out.image_transform().0 < 1.0);
    // Without room the same zoom stops at the picture.
    let plain = Reach::new(0.0).zoomed(crop, 0.01, (0.5, 0.5), (0.5, 0.5), 0.05);
    assert_close(plain, rect(0.0, 0.0, 1.0, 1.0));
}

#[test]
fn a_slider_range_reaches_into_the_overhang() {
    let reach = Reach::new(0.25);
    let (least, greatest) = reach.range(rect(0.2, 0.3, 0.4, 0.4), Grip::Move, false, (None, 0.05));
    assert!(
        (least + 0.25).abs() < 1e-9 && (greatest - 0.85).abs() < 1e-9,
        "{least} {greatest}"
    );
}

#[test]
fn controls_none_removes_the_buttons_and_false_hides_them() {
    let cropper = |controls: ImageCropperControls| {
        render(Setup {
            controls,
            ..Setup::default()
        })
    };
    assert!(cropper(ImageCropperControls::Shown).contains(r#"data-slot="controls""#));
    let state = |html: &str| {
        let from = html.find(r#"data-state=""#).unwrap() + r#"data-state=""#.len();
        html[from..].split('"').next().unwrap().to_string()
    };
    let hidden = cropper(false.into());
    assert!(
        hidden.contains(r#"data-slot="controls""#) && state(&hidden).contains("hidden-controls")
    );
    let none = cropper(ImageCropperControls::None);
    assert!(
        !none.contains(r#"data-slot="controls""#) && !state(&none).contains("hidden-controls"),
        "{none}"
    );
    assert!(none.contains(r#"role="slider""#), "the keys stay: {none}");
}

#[test]
fn controls_none_removes_the_pan_zoom_bar_too() {
    let pan = |controls: ImageCropperControls| {
        render(Setup {
            controls,
            pan: true,
            ..Setup::default()
        })
    };
    assert!(pan(ImageCropperControls::Shown).contains(r#"data-slot="zoom""#));
    assert!(!pan(ImageCropperControls::None).contains(r#"data-slot="zoom""#));
}

#[test]
fn a_backdrop_is_drawn_only_with_an_overhang_or_a_colour() {
    let html = |overhang: f64, backdrop: Backdrop| {
        render(Setup {
            overhang,
            backdrop,
            ..Setup::default()
        })
    };
    let slot = r#"data-slot="backdrop""#;
    assert!(!html(0.0, Backdrop::Checkerboard).contains(slot));
    let over = html(0.25, Backdrop::Checkerboard);
    assert!(
        over.contains(slot) && over.contains("--lsx-image-cropper-over:0.25"),
        "{over}"
    );
    assert!(over.contains("repeating-conic-gradient"));
    let flat = html(0.0, Backdrop::rgb(255, 0, 0));
    assert!(
        flat.contains(slot) && flat.contains("rgba(255, 0, 0, 1)"),
        "{flat}"
    );
}
