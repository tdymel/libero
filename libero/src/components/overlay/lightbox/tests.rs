use super::{LightboxPart, zoom::*};
use crate::{components::common::part_table, hooks::DragPoint, platform::Dimensions};

const FRAME: Dimensions = Dimensions {
    width: 800.0,
    height: 600.0,
};

/// A picture exactly the frame's size: the bounds before natural sizes.
const FILLS: Fit = Fit {
    frame: FRAME,
    picture: FRAME,
};

fn size(width: f64, height: f64) -> Dimensions {
    Dimensions { width, height }
}

#[test]
fn a_small_picture_is_shown_at_its_natural_size() {
    let fit = Fit::new(FRAME, Some(size(200.0, 100.0)));

    assert_eq!(fit.picture, size(200.0, 100.0));
}

#[test]
fn a_large_picture_is_scaled_down_into_the_frame() {
    let fit = Fit::new(FRAME, Some(size(3200.0, 1200.0)));

    assert_eq!(fit.picture, size(800.0, 300.0));
}

/// Todo 252: a second `open_with` drops the zoom, even on the same index.
/// SSR cannot measure a frame, so only this decision is tested.
#[test]
fn a_second_open_drops_the_zoom_and_clamps_the_index() {
    let zoomed = Zoom {
        index: 2,
        scale: 2.0,
        x: 10.0,
        y: 0.0,
    };
    assert_eq!(reopened(zoomed, 2, 4), (2, Zoom::fitted(usize::MAX)));
    assert_eq!(reopened(zoomed, 9, 4), (3, Zoom::fitted(usize::MAX)));
    // A fitted zoom is left as it is, so reopening writes nothing.
    assert_eq!(reopened(Zoom::fitted(1), 0, 4), (0, Zoom::fitted(1)));
    assert_eq!(reopened(Zoom::fitted(1), 0, 0).0, 0);
}

/// At 3x a 200px picture is still 600px, inside the 800px frame.
#[test]
fn a_zoomed_picture_smaller_than_the_frame_cannot_move() {
    let zoomed = Zoom {
        scale: 3.0,
        ..Zoom::fitted(0)
    };
    let fit = Fit::new(FRAME, Some(size(200.0, 100.0)));

    assert_eq!(zoomed.panned(500.0, 500.0, fit), zoomed);
}

/// Scaled down to 800x300, then doubled to 1600x600: it overhangs 400px
/// sideways and nothing vertically.
#[test]
fn a_pan_is_bounded_by_the_picture_not_the_frame() {
    let zoomed = Zoom {
        scale: 2.0,
        ..Zoom::fitted(0)
    };
    let fit = Fit::new(FRAME, Some(size(3200.0, 1200.0)));

    let far = zoomed.panned(10_000.0, 10_000.0, fit);

    assert_eq!((far.x, far.y), (400.0, 0.0));
}

#[test]
fn without_a_natural_size_the_picture_fills_the_frame() {
    assert_eq!(Fit::new(FRAME, None), FILLS);
}

#[test]
fn a_fitted_picture_cannot_move() {
    let fitted = Zoom::fitted(0);

    assert_eq!(fitted.panned(40.0, -40.0, FILLS), fitted);
}

/// At scale 2 an 800px picture overhangs 400px each side, and no further.
#[test]
fn a_pan_stops_where_the_picture_would_leave_the_frame() {
    let zoomed = Zoom {
        scale: 2.0,
        ..Zoom::fitted(0)
    };

    let far = zoomed.panned(10_000.0, -10_000.0, FILLS);

    assert_eq!((far.x, far.y), (400.0, -300.0));
}

/// The key-at-the-edge rule rests on this: once clamped, another pan the
/// same way is no change at all, so the key goes to the slide instead.
#[test]
fn a_pan_at_the_edge_is_no_change() {
    let at_edge = Zoom {
        scale: 2.0,
        x: 400.0,
        ..Zoom::fitted(0)
    };

    assert_eq!(at_edge.panned(PAN_STEP, 0.0, FILLS), at_edge);
    assert_ne!(at_edge.panned(-PAN_STEP, 0.0, FILLS), at_edge);
}

/// A pan short of the edge by less than a step still moves, and lands on
/// the edge - the next press falls through.
#[test]
fn a_pan_that_reaches_the_edge_is_clamped_onto_it() {
    let near = Zoom {
        scale: 2.0,
        x: 390.0,
        ..Zoom::fitted(0)
    };

    assert_eq!(near.panned(PAN_STEP, 0.0, FILLS).x, 400.0);
}

#[test]
fn zooming_about_the_centre_does_not_move_the_picture() {
    let zoomed = Zoom::fitted(0).scaled(2.0, DragPoint { x: 0.0, y: 0.0 }, FILLS);

    assert_eq!((zoomed.scale, zoomed.x, zoomed.y), (2.0, 0.0, 0.0));
}

/// The spot under the cursor stays under it: a point 100px right of centre
/// on a fitted picture is still 100px right of centre at scale 2.
#[test]
fn zooming_about_a_point_keeps_that_point_still() {
    let point = DragPoint { x: 100.0, y: -50.0 };
    let zoomed = Zoom::fitted(0).scaled(2.0, point, FILLS);

    // Image-local position of the point, then back to the screen.
    let local = (point.x / 1.0, point.y / 1.0);
    let screen = (
        zoomed.x + zoomed.scale * local.0,
        zoomed.y + zoomed.scale * local.1,
    );
    assert_eq!(screen, (point.x, point.y));
}

/// Todo 565: a click 100px right of and 50px above the centre moves that
/// spot to the centre, within the pan bounds.
#[test]
fn a_click_centres_the_spot_clicked() {
    let zoomed = Zoom {
        scale: 2.0,
        x: 20.0,
        ..Zoom::fitted(0)
    };

    let centred = zoomed.centred_on(DragPoint { x: 100.0, y: -50.0 }, FILLS);
    assert_eq!((centred.x, centred.y), (-80.0, 50.0));

    let far = zoomed.centred_on(DragPoint { x: -1000.0, y: 0.0 }, FILLS);
    assert_eq!(far.x, 400.0);
}

#[test]
fn the_transform_divides_the_shift_by_the_scale() {
    let zoomed = Zoom {
        scale: 2.0,
        x: 100.0,
        y: -40.0,
        ..Zoom::fitted(0)
    };

    assert_eq!(zoomed.transform(), "scale(2) translate(50px, -20px)");
}

#[test]
fn a_wheel_step_stays_between_fitted_and_max_zoom() {
    assert_eq!(wheel_step(1.0, false, 3.0), WHEEL_FACTOR);
    assert_eq!(wheel_step(2.8, false, 3.0), 3.0);
    assert_eq!(wheel_step(1.1, true, 3.0), 1.0);
}

/// Todo 864: 2x, 4x, 8x, then back to fit; a lower `max_zoom` caps the
/// last step, and a wheel's in-between scale steps to the next double.
#[test]
fn a_zoom_step_doubles_up_to_max_zoom_then_fits() {
    let steps: Vec<f64> = std::iter::successors(Some(1.0), |&s| Some(zoom_step(s, 8.0)))
        .skip(1)
        .take(4)
        .collect();
    assert_eq!(steps, [2.0, 4.0, 8.0, 1.0]);
    assert_eq!(zoom_step(2.0, 3.0), 3.0);
    assert_eq!(zoom_step(3.0, 3.0), 1.0);
    assert_eq!(zoom_step(WHEEL_FACTOR, 8.0), 2.0);
    assert_eq!(zoom_step(1.0, 1.0), 1.0);
}

/// Todo 1067: fingers spread to twice their gap double the scale about their
/// starting midpoint, and the midpoint's travel pans on top.
#[test]
fn a_pinch_scales_about_its_midpoint_and_follows_it() {
    let (gap, mid) = span(DragPoint { x: 0.0, y: 0.0 }, DragPoint { x: 60.0, y: 80.0 });
    assert_eq!((gap, mid), (100.0, DragPoint { x: 30.0, y: 40.0 }));

    let origin = DragPoint { x: 100.0, y: -50.0 };
    let still = DragPoint { x: 0.0, y: 0.0 };
    let doubled = pinched(Zoom::fitted(0), 2.0, origin, still, FILLS);
    assert_eq!(doubled, Zoom::fitted(0).scaled(2.0, origin, FILLS));

    let moved = pinched(
        Zoom::fitted(0),
        2.0,
        origin,
        DragPoint { x: 10.0, y: 0.0 },
        FILLS,
    );
    assert_eq!(moved.x, doubled.x + 10.0);
}

/// Pinched back to 1x or under, the picture fits again, centred.
#[test]
fn a_pinch_closed_to_fit_centres_the_picture() {
    let zoomed = Zoom {
        scale: 2.0,
        x: 200.0,
        ..Zoom::fitted(3)
    };
    let origin = DragPoint { x: 0.0, y: 0.0 };
    assert_eq!(pinched(zoomed, 1.0, origin, origin, FILLS), Zoom::fitted(3));
}

#[test]
fn only_a_long_mostly_downward_swipe_closes() {
    assert!(swipe_closes(DragPoint { x: 10.0, y: 120.0 }));
    assert!(!swipe_closes(DragPoint { x: 0.0, y: 40.0 }));
    assert!(!swipe_closes(DragPoint { x: 200.0, y: 120.0 }));
    assert!(!swipe_closes(DragPoint { x: 0.0, y: -200.0 }));
}

/// The slot names are public: a rename here is a breaking change.
#[test]
fn the_part_table_is_stable() {
    let body = "& > [data-slot='body']";
    let frame = format!("{body} > [data-slot='stage'] [data-slot='frame']");
    let expected: Vec<(&str, String)> = vec![
        ("toolbar", "& > [data-slot='toolbar']".into()),
        (
            "zoom-out",
            "& > [data-slot='toolbar'] > [data-slot='zoom-out']".into(),
        ),
        (
            "zoom-in",
            "& > [data-slot='toolbar'] > [data-slot='zoom-in']".into(),
        ),
        (
            "close",
            "& > [data-slot='toolbar'] > [data-slot='close']".into(),
        ),
        ("body", body.into()),
        ("stage", format!("{body} > [data-slot='stage']")),
        ("frame", frame.clone()),
        ("image", format!("{frame} > [data-slot='image']")),
        ("caption", format!("{body} > [data-slot='caption']")),
        ("thumbnails", format!("{body} > [data-slot='thumbnails']")),
        (
            "thumbnail",
            format!("{body} > [data-slot='thumbnails'] [data-slot='thumbnail']"),
        ),
    ];
    let table: Vec<(&str, String)> = part_table::<LightboxPart>()
        .into_iter()
        .map(|(slot, selector)| (slot, selector.to_string()))
        .collect();

    assert_eq!(table, expected);
}
