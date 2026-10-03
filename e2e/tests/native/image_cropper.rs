//! `ImageCropper` under Blitz's pointer: a drag and the wheel begun on the image beside the box
//! (2113). The browser's twin is `all/image_cropper.rs`.

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{CropRect, ImageCropper};

const PICTURE: &str = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='400' height='200'%3E%3Crect width='400' height='200' fill='%23339af0'/%3E%3C/svg%3E";

fn app() -> Element {
    let mut crop = use_signal(|| CropRect {
        x: 0.25,
        y: 0.25,
        width: 0.5,
        height: 0.5,
    });
    let percent = |fraction: f64| (fraction * 100.0).round();
    let rect = crop();
    rsx! {
        div { style: "padding: 16px; width: 400px",
            ImageCropper {
                src: PICTURE,
                alt: "A blue picture",
                value: rect,
                onchange: move |next| crop.set(next),
            }
            p { id: "crop",
                "{percent(rect.x)},{percent(rect.y)},{percent(rect.width)},{percent(rect.height)}"
            }
        }
    }
}

fn reads(page: &mut Page, expected: &str) {
    let found = page.wait_for(|page| page.text("#crop") == expected);
    assert!(found, "#crop reads {}, not {expected}", page.text("#crop"));
}

#[test]
fn a_drag_and_the_wheel_beside_the_box_move_and_scale_it() {
    let mut page = mount(app);
    // A `data:` picture decodes after mount.
    let laid_out = page.wait_for(|page| page.rect("[data-slot=image]").3 > 0.0);
    assert!(laid_out, "the picture never laid out");
    let (left, top, width, height) = page.rect("[data-slot=image]");
    // Near the left edge, clear of the box and its corners' hit areas.
    let (x, y) = ((left + width * 0.04) as f32, (top + height * 0.1) as f32);
    let (dx, dy) = ((width / 10.0) as f32, (height / 10.0) as f32);
    page.press_at(x, y);
    for step in 1..=8u8 {
        let t = f32::from(step) / 8.0;
        page.move_to(x + dx * t, y + dy * t);
    }
    page.release_at(x + dx, y + dy);
    reads(&mut page, "35,35,50,50");
    page.wheel_at(x, y, 100.0);
    reads(&mut page, "37,37,45,45");
}
