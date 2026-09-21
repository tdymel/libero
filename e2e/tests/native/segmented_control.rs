//! `SegmentedControl` as Blitz paints it. The clicks, arrows, Enter and Tab
//! are e2e's shared scenarios (`segmented_control::`).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

/// Todo 849: Blitz hoists the `z-index: 1` checked segment at the previous layout's offsets
/// (`flush_styles_to_layout` before `resolve_layout`): one stale frame after a resize.
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
