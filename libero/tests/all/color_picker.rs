use crate::common::render;

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ColorCode, ColorPicker},
};

/// The panel's one `role="slider"` holds the saturation, but Up and Down move
/// the brightness - so its text names both, or those keys announce nothing.
#[test]
fn the_saturation_thumb_says_saturation_and_brightness() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorPicker {
                    value: ColorCode::hsva(200.0, 0.4, 0.7, 1.0),
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    assert!(html.contains("aria-valuenow=40"), "{html}");
    assert!(
        html.contains(r#"aria-valuetext="Saturation 40%, brightness 70%""#),
        "{html}"
    );
}
