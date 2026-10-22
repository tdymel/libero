//! Blitz lays out no SVG child; the harness rebuilds a mark's rect from its attributes.

use dioxus::prelude::*;
use e2e::native::mount;

fn app() -> Element {
    rsx! {
        svg { id: "s", width: "200", height: "100", view_box: "0 0 100 50",
            rect { id: "r", x: "10", y: "5", width: "20", height: "10" }
            line { id: "l", x1: "50", y1: "0", x2: "50", y2: "40" }
            circle { id: "c", cx: "70", cy: "20", r: "5" }
            path { id: "p", d: "M10 10 L30 10 A5 5 0 0 1 35 15 V40 h-25 Z" }
        }
    }
}

fn near(got: (f64, f64, f64, f64), want: (f64, f64, f64, f64)) {
    let close = |a: f64, b: f64| (a - b).abs() < 0.6;
    assert!(
        close(got.0, want.0)
            && close(got.1, want.1)
            && close(got.2, want.2)
            && close(got.3, want.3),
        "{got:?} != {want:?}"
    );
}

/// The picture scales 2x: a mark's box is its user-space box doubled, from the svg's corner.
#[test]
fn a_mark_takes_its_box_through_the_view_box() {
    let page = mount(app);
    let (x, y, _, _) = page.rect("#s");
    near(page.rect("#r"), (x + 20.0, y + 10.0, 40.0, 20.0));
    near(page.rect("#l"), (x + 100.0, y, 0.0, 80.0));
    near(page.rect("#c"), (x + 130.0, y + 30.0, 20.0, 20.0));
    near(page.rect("#p"), (x + 20.0, y + 20.0, 50.0, 60.0));
}
