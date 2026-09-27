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
fn pixels_round_and_stay_inside_the_image() {
    let pixels = rect(0.5, 0.5, 0.6, 0.6).to_pixels(101, 100);
    assert_eq!((pixels.x, pixels.y), (51, 50));
    assert_eq!((pixels.width, pixels.height), (50, 50));
}
