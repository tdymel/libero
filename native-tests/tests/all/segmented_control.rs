//! `SegmentedControl` as Blitz paints it. The clicks, arrows, Enter and Tab
//! are e2e's shared scenarios (`segmented_control::`).

use dioxus::prelude::*;
use libero::components::{Options, SegmentedControl};
use native_tests::{Page, mount};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

/// Todo 849: the checked segment is `z-index: 1`, and Blitz hoists such a box
/// into its stacking context at the offsets of the layout before (blitz-dom
/// `flush_styles_to_layout` runs ahead of `resolve_layout`). The first frame
/// after a resize paints it where it was; the next frame puts it right.
#[test]
#[ignore = "Blitz paints a z-indexed box at its pre-resize place for one frame"]
fn the_checked_segment_paints_in_place_after_a_resize() {
    fn app() -> Element {
        let mut alignment = use_signal(|| Alignment::Center);
        rsx! {
            div { margin: "0 auto", width: "50%",
                SegmentedControl {
                    label: "Alignment",
                    value: alignment(),
                    onchange: move |next| alignment.set(next),
                }
            }
        }
    }
    let mut page = mount(app);
    let checked = "label[data-state~=checked]";
    let selected = |page: &Page| {
        let (x, y, _, height) = page.rect(checked);
        page.painted_pixel((x + 5.0) as u32, (y + height / 2.0) as u32)
    };
    let before = selected(&page);
    page.resize(700, 800);
    assert_eq!(
        selected(&page),
        before,
        "the checked segment moved off its box"
    );
}
