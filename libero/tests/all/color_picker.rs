use crate::common::{attributes_of, body, render, tags_with};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        AlphaSlider, ColorCode, ColorField, ColorFieldVariant, ColorPicker, ColorSwatch, HueSlider,
        Swatches,
    },
};

/// Todo 2819: forced colours drop a swatch's background and ring unless it opts out.
#[test]
fn a_swatch_keeps_its_colour_under_forced_colours() {
    fn app() -> Element {
        rsx! {
            LiberoProvider { ColorSwatch { color: ColorCode::rgba(34, 139, 230, 1.0) } }
        }
    }

    let html = render(app);
    assert!(html.contains("forced-color-adjust:none;"), "{html}");
}

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

/// A blank label names nothing: the part keeps its default name, and a blank swatch label
/// falls back to the hex.
#[test]
fn blank_picker_labels_fall_back_to_the_defaults() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorPicker {
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    with_alpha: true,
                    saturation_label: "  ",
                    hue_label: "",
                    alpha_label: " ",
                    swatches: Swatches::labelled([("#fa5252", " ")]),
                }
            }
        }
    }
    let html = body(&render(app));
    let sliders = tags_with(&html, r#"role="slider""#);
    assert_eq!(sliders.len(), 3, "{html}");
    for slider in sliders {
        let name = slider.get("aria-label").map_or("", String::as_str);
        assert!(!name.trim().is_empty(), "{slider:?}");
    }
    assert!(html.contains(r##"aria-label="#fa5252""##), "{html}");
}

/// The standalone sliders and the panel drop a blank name instead of rendering `aria-label=""`.
#[test]
fn a_blank_slider_name_is_not_rendered() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                HueSlider { value: 10.0, aria_label: "  ", oninput: move |_| {} }
                AlphaSlider {
                    value: 0.5,
                    color: ColorCode::default(),
                    aria_label: " ",
                    oninput: move |_| {},
                }
            }
        }
    }
    let html = body(&render(app));
    let sliders = tags_with(&html, r#"role="slider""#);
    assert_eq!(sliders.len(), 2, "{html}");
    for slider in sliders {
        assert_eq!(slider.get("aria-label"), None, "{slider:?}");
    }
}

/// Todos 5121, 5122: without `oninput` a lone colour slider only shows its value, and
/// only a picker's own sliders take its `hue` and `alpha` slots.
#[test]
fn a_lone_colour_slider_is_read_only_without_oninput_and_takes_no_picker_slot() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                HueSlider { value: 10.0, aria_label: "Hue" }
                AlphaSlider { value: 0.5, color: ColorCode::default(), aria_label: "Alpha" }
            }
        }
    }
    let html = body(&render(app));
    let sliders = tags_with(&html, r#"role="slider""#);
    assert_eq!(sliders.len(), 2, "{html}");
    for slider in sliders {
        assert_eq!(
            slider.get("aria-readonly").map(String::as_str),
            Some("true"),
            "{slider:?}"
        );
    }
    for slot in ["hue", "alpha"] {
        assert!(
            tags_with(&html, &format!(r#"data-slot="{slot}""#)).is_empty(),
            "{html}"
        );
    }

    fn picker() -> Element {
        rsx! {
            LiberoProvider {
                ColorPicker { value: ColorCode::default(), oninput: move |_| {}, with_alpha: true }
            }
        }
    }
    let html = body(&render(picker));
    for slot in ["hue", "alpha"] {
        assert_eq!(
            tags_with(&html, &format!(r#"data-slot="{slot}""#)).len(),
            1,
            "{html}"
        );
    }
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

/// Todo 5633: a gradient paints the swatch, while `color` still picks the mark's black or white.
#[test]
fn a_swatch_background_replaces_the_painted_colour() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorSwatch {
                    color: ColorCode::rgba(255, 255, 255, 1.0),
                    background: "conic-gradient(red, blue)",
                }
                ColorSwatch { color: ColorCode::rgba(34, 139, 230, 1.0) }
            }
        }
    }

    let html = render(app);
    assert_eq!(
        html.matches("--lsx-color-swatch-fill:conic-gradient(red, blue)")
            .count(),
        1,
        "{html}"
    );
    assert!(html.contains("on-light"), "{html}");
}

/// Todo 5633: the swatch variant has no text input, and its button is the combobox's counterpart.
#[test]
fn a_swatch_color_field_is_a_button_over_a_dialog() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorField {
                    label: "Accent",
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    variant: ColorFieldVariant::Swatch,
                }
            }
        }
    }
    let html = body(&render(app));
    let button = attributes_of(&html, "button");

    assert!(tags_with(&html, "<input").is_empty(), "{html}");
    assert_eq!(button["aria-haspopup"], "dialog");
    assert_eq!(button["aria-expanded"], "false");
    assert!(!button.contains_key("aria-controls"), "{html}");
    assert_eq!(button["aria-labelledby"], format!("{}-label", button["id"]));
}

/// Todo 5633: without a label the swatch still has a name, and a user `aria-label` wins.
#[test]
fn a_swatch_color_field_is_always_named() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ColorField {
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    variant: ColorFieldVariant::Swatch,
                }
                ColorField {
                    aria_label: "Fill",
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    variant: ColorFieldVariant::Swatch,
                }
            }
        }
    }
    let html = body(&render(app));

    assert!(html.contains(r#"aria-label="Fill""#), "{html}");
    assert_eq!(html.matches("aria-label=").count(), 2, "{html}");
}

/// Todo 5723: `swatch_selected` checks the swatch and marks it `aria-current`; unset or `false`
/// renders the same markup as before.
#[test]
fn a_selected_swatch_color_field_is_checked_and_current() {
    fn field(selected: Option<bool>) -> Element {
        rsx! {
            LiberoProvider {
                ColorField {
                    label: "Accent",
                    value: ColorCode::default(),
                    oninput: move |_| {},
                    variant: ColorFieldVariant::Swatch,
                    swatch_selected: selected,
                }
            }
        }
    }
    let html = |selected| body(&crate::common::render_with(field, selected));
    let checked = html(Some(true));
    let button = attributes_of(&checked, "button");
    assert_eq!(button["aria-current"], "true", "{checked}");
    let inner = &checked[checked.find("<button").unwrap()..checked.find("</button>").unwrap()];
    assert!(inner.contains("<svg"), "{checked}");

    let unset = html(None);
    assert_eq!(unset, html(Some(false)));
    assert!(!unset.contains("aria-current"), "{unset}");
    let inner = &unset[unset.find("<button").unwrap()..unset.find("</button>").unwrap()];
    assert!(!inner.contains("<svg"), "{unset}");
}
