//! The Tour's mask technique (todo 2210 probe): an outer `100vmax` spread shadow dims all but
//! the hole, and a transparent blocker under it takes every press, the hole's too.

use dioxus::prelude::*;
use e2e::native::mount;

fn app() -> Element {
    let mut presses = use_signal(|| 0);
    rsx! {
        button {
            id: "target",
            style: "position:absolute;left:100px;top:100px;width:80px;height:40px;",
            onclick: move |_| presses += 1,
            "Target"
        }
        span { id: "presses", "{presses}" }
        // Natively `fixed` is laid out against the parent: this box stands in for `PortalOutlet`.
        div { style: "position:absolute;left:0;top:0;width:100vw;height:100vh;",
            div { id: "blocker", style: "position:fixed;inset:0;z-index:10;" }
            div {
                id: "hole",
                style: "position:fixed;left:94px;top:94px;width:92px;height:52px;border-radius:4px;box-shadow:0 0 0 100vmax rgba(0,0,0,0.5);pointer-events:none;z-index:11;",
            }
        }
    }
}

fn luma([r, g, b, _]: [u8; 4]) -> u32 {
    u32::from(r) + u32::from(g) + u32::from(b)
}

#[test]
fn a_100vmax_spread_dims_all_but_the_hole() {
    let page = mount(app);
    // Far corners of the viewport, then inside the hole beside the button.
    let pixels = page.painted_pixels(&[(5, 5), (700, 500), (96, 120), (185, 94)]);
    let (outside, far, inside, corner) = (pixels[0], pixels[1], pixels[2], pixels[3]);
    assert!(
        luma(outside) + 60 < luma(inside),
        "{outside:?} vs {inside:?}"
    );
    assert!(luma(far) + 60 < luma(inside), "{far:?} vs {inside:?}");
    // The rounded corner leaves its outer pixel dimmed.
    assert!(luma(corner) < luma(inside), "{corner:?} vs {inside:?}");
}

#[test]
fn the_blocker_takes_a_press_in_the_hole() {
    let mut page = mount(app);
    assert!(page.hits_at("#blocker", 140.0, 120.0));
    assert!(!page.hits_at("#hole", 140.0, 120.0));
    page.click_at(140.0, 120.0);
    assert_eq!(page.text("#presses"), "0");
}
