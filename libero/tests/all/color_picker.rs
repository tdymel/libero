use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ColorCode, ColorField, ColorPicker, Swatches},
};

/// Todo 710: a slider mirrors under RTL, but a colour scale's gradient runs
/// left to right, so the hue and alpha sliders stay LTR.
#[test]
fn the_colour_sliders_stay_left_to_right() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorPicker {
                    value: ColorCode::hsva(200.0, 0.4, 0.7, 0.5),
                    oninput: move |_| {},
                    with_alpha: true,
                }
            }
        }
    }

    let html = render(app);
    assert_eq!(html.matches(r#"dir="ltr""#).count(), 2, "{html}");
}

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

/// The swatch equal to the value is pressed and checked; a grey's drifted hue
/// still matches, because the comparison is by the colour drawn (todo 553).
#[test]
fn the_swatch_equal_to_the_value_is_pressed() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorPicker {
                    value: ColorCode::hsva(123.0, 0.0, 0.0, 1.0),
                    oninput: move |_| {},
                    swatches: ["#fa5252", "#000000"],
                }
            }
        }
    }
    let html = body(&render(app));
    assert!(
        html.contains(r##"aria-label="#fa5252" aria-pressed="false""##),
        "{html}"
    );
    let pressed = html
        .split(r##"aria-label="#000000" aria-pressed="true""##)
        .nth(1)
        .unwrap_or_else(|| panic!("black is not pressed: {html}"));
    assert!(pressed.split("</button>").next().unwrap().contains("<svg"));
    assert_eq!(html.matches("<svg").count(), 1, "{html}");
}

/// A caller's label replaces the hex name.
#[test]
fn a_labelled_swatch_is_named_by_its_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorPicker {
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    swatches: Swatches::labelled([("#fa5252", "Red")]),
                }
            }
        }
    }
    let html = body(&render(app));
    assert!(html.contains(r#"aria-label="Red""#), "{html}");
    assert!(!html.contains("#fa5252"), "{html}");
}

thread_local! {
    static VALUE: std::cell::Cell<Option<Signal<ColorCode>>> = const { std::cell::Cell::new(None) };
}

/// The panel skips a drag frame and only its thumb redraws, so a new `value`
/// must still reach the thumb, the panel's hue and the alpha track's color.
#[test]
fn a_new_value_reaches_the_thumb_through_the_skipped_panel() {
    use dioxus::dioxus_core::{NoOpMutations, VirtualDom};

    fn app() -> Element {
        let value = use_signal(|| ColorCode::hsva(200.0, 0.4, 0.7, 1.0));
        VALUE.set(Some(value));
        rsx! {
            LiberoProvider {
                ColorPicker { value: value(), oninput: move |_| {}, with_alpha: true }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let before = dioxus_ssr::render(&dom);
    assert!(before.contains("aria-valuenow=40"), "{before}");

    let mut value = VALUE.get().expect("the app ran");
    dom.in_runtime(|| value.set(ColorCode::hsva(120.0, 0.6, 0.5, 1.0)));
    dom.render_immediate(&mut NoOpMutations);
    let after = dioxus_ssr::render(&dom);

    assert!(after.contains("aria-valuenow=60"), "{after}");
    assert!(
        after.contains(r#"aria-valuetext="Saturation 60%, brightness 50%""#),
        "{after}"
    );
    assert!(
        after.contains("--lsx-color-picker-saturation-hue:#00ff00"),
        "{after}"
    );
    let (r, g, b, _) = ColorCode::hsva(120.0, 0.6, 0.5, 1.0).to_rgba_channels();
    assert!(
        after.contains(&format!("--lsx-color-picker-rgb:{r}, {g}, {b}")),
        "{after}"
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
