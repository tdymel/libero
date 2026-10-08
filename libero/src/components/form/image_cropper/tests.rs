use super::crop::{CropRect, Grip};

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
fn pixels_round_and_stay_inside_the_image() {
    let pixels = rect(0.5, 0.5, 0.6, 0.6).to_pixels(101, 100);
    assert_eq!((pixels.x, pixels.y), (51, 50));
    assert_eq!((pixels.width, pixels.height), (50, 50));
}
