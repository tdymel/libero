use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ColorCode, ColorField, ColorPicker},
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

/// `ColorField` follows the APG Date Picker Combobox: the input is a combobox
/// opening a dialog, and points at it only while it exists. The browser pass
/// opens it.
#[test]
fn a_closed_color_field_is_a_combobox_over_a_dialog() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorField { value: ColorCode::default(), oninput: move |_| {} }
            }
        }
    }
    let html = body(&render(app));
    let input = attributes_of(&html, "input");

    assert_eq!(input["role"], "combobox");
    assert_eq!(input["aria-haspopup"], "dialog");
    assert_eq!(input["aria-expanded"], "false");
    assert!(!input.contains_key("aria-controls"), "{html}");
}
