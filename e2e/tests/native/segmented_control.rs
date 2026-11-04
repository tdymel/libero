//! `SegmentedControl` as Blitz paints it. The clicks, arrows, Enter and Tab
//! are e2e's shared scenarios (`segmented_control::`).

use dioxus::prelude::*;
use e2e::native::{Page, mount};
use libero::components::{Icon, OptionLabel, Options, SegmentedControl};

#[derive(Clone, Copy, PartialEq, Options)]
enum Alignment {
    Left,
    Center,
    Right,
}

/// Todo 849: the `z-index: 1` checked segment paints at the new layout's offsets
/// right after a resize, no stale frame (Blitz 0.3.0-beta.3).
#[test]
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

/// Todo 5701: Blitz resolves the slot colour's nested `var()` fallback, so a colourless
/// `Icon` in a segment takes the segment's text colour, and one outside keeps primary.
#[test]
fn a_colourless_icon_takes_the_segment_colour() {
    fn app() -> Element {
        rsx! {
            SegmentedControl {
                label: "Alignment",
                value: Alignment::Center,
                onchange: move |_| {},
                option_label: |alignment: Alignment| OptionLabel::with_icon(
                    alignment.label(),
                    rsx! { Icon { variant: "standard", size: "sm", "*" } },
                ),
            }
            Icon { id: "outside", variant: "standard", "*" }
        }
    }
    let page = mount(app);
    let label = "label[data-state~=checked]";
    let icon = "label[data-state~=checked] [data-slot=icon]";
    assert_eq!(page.computed(icon, "color"), page.computed(label, "color"));
    assert_ne!(
        page.computed("#outside", "color"),
        page.computed(label, "color")
    );
}
