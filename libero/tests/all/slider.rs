use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{RangeSlider, Slider, SliderMark, SliderValue},
    theme::{Size, SliderDefaults, Theme},
};

#[test]
fn slider_renders_a_thumb_with_the_value_and_its_marks() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: 25.0,
                    aria_label: "Volume",
                    marks: vec![SliderMark::labeled(50.0, "half")],
                    format: Callback::new(|value: f64| format!("{value}%")),
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    // The first div is the field wrapper; the slider's own root is the second.
    let root = attributes_of(&html[html.find("<div").unwrap() + 4..], "div");

    assert_eq!(root["data-state"], "size-md marks-labeled");
    // On the bar, which redraws without the root.
    let filled = html.find("--lsx-slider-filled:").unwrap();
    let bar = attributes_of(&html[html[..filled].rfind("<div").unwrap()..], "div");
    assert!(
        bar["style"].contains("--lsx-slider-filled:0.25;"),
        "{bar:?}"
    );

    // The thumb carries the a11y contract; the mark carries its position.
    assert!(html.contains(r#"role="slider""#));
    assert!(html.contains(r#"aria-label="Volume""#));
    assert!(html.contains("aria-valuenow=25"));
    assert!(html.contains("--lsx-slider-mark-at:0.5"));
    assert!(body(&html).contains(">half<"));
    // The value bubble is a `Tooltip`.
    assert!(html.contains(r#"role="tooltip""#));
    assert!(body(&html).contains(">25%<"));
}

/// Filled is a state, not a var set only on filled marks: a raw `style` that
/// drops a declaration between renders keeps its last value in the browser.
#[test]
fn a_slider_mark_says_filled_with_a_state_and_keeps_its_style_shape() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: 25.0,
                    aria_label: "Volume",
                    marks: vec![SliderMark::new(0.0), SliderMark::new(50.0)],
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    assert!(html.contains(r#"data-state="filled" style="--lsx-slider-mark-at:0;""#));
    assert!(html.contains(r#"style="--lsx-slider-mark-at:0.5;""#));
    assert!(!html.contains("--lsx-slider-mark-fill"));
}

#[test]
fn a_range_slider_renders_two_thumbs_and_posts_both_values() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                RangeSlider {
                    label: "Price",
                    value: (20.0, 80.0),
                    min_range: 10.0,
                    name: "price",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    // The bar spans between the thumbs rather than from the track's start.
    assert!(html.contains("--lsx-slider-filled-from:0.2;"), "{html}");
    assert!(html.contains("--lsx-slider-filled-span:0.6"), "{html}");

    assert_eq!(html.matches(r#"role="slider""#).count(), 2);
    assert!(html.contains("--lsx-slider-thumb-at:0.2"));
    assert!(html.contains("--lsx-slider-thumb-at:0.8"));
    assert!(html.contains(r#"aria-label="Minimum""#));
    assert!(html.contains(r#"aria-label="Maximum""#));
    // `aria-labelledby` beats `aria-label`, so each thumb lists itself after
    // the caption: "Price Minimum", "Price Maximum".
    let thumbs: Vec<_> = html
        .match_indices(r#"role="slider""#)
        .map(|(at, _)| attributes_of(&html[html[..at].rfind('<').unwrap()..], "span"))
        .collect();
    for thumb in &thumbs {
        let label = thumb["aria-labelledby"].split(' ').collect::<Vec<_>>();
        assert_eq!(label.len(), 2, "{thumb:?}");
        assert!(label[0].ends_with("-label"), "{thumb:?}");
        assert_eq!(label[1], thumb["id"], "{thumb:?}");
    }
    assert_ne!(thumbs[0]["id"], thumbs[1]["id"]);

    // Each thumb is bounded by its neighbour, `min_range` short of it.
    assert!(html.contains("aria-valuenow=20"));
    assert!(html.contains("aria-valuemax=70"));
    assert!(html.contains("aria-valuenow=80"));
    assert!(html.contains("aria-valuemin=30"));

    // Both ends post under one name, in track order.
    let inputs: Vec<_> = html.match_indices(r#"type="hidden""#).collect();
    assert_eq!(inputs.len(), 2, "{html}");
    assert_eq!(html.matches(r#"name="price""#).count(), 2, "{html}");
    let first = html.find(r#"value="20""#).expect("the lower value");
    let second = html.find(r#"value="80""#).expect("the upper value");
    assert!(first < second, "{html}");
}

/// The value bubble is a `role="tooltip"`, and a tooltip nothing points at
/// has no owner in the accessibility tree: each thumb lists its own bubble,
/// after the field's captions.
#[test]
fn each_thumb_is_described_by_its_own_value_bubble() {
    fn single() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: 40.0,
                    label: "Volume",
                    description: "Loud is 80.",
                    oninput: move |_| {},
                }
            }
        }
    }
    fn range() -> Element {
        rsx! {
            LiberoProvider {
                RangeSlider {
                    value: (20.0, 80.0),
                    aria_label: "Price",
                    oninput: move |_| {},
                }
            }
        }
    }

    fn bubbles(html: &str) -> Vec<Vec<String>> {
        html.match_indices(r#"role="slider""#)
            .map(|(at, _)| {
                let thumb = attributes_of(&html[html[..at].rfind('<').unwrap()..], "span");
                thumb["aria-describedby"]
                    .split(' ')
                    .map(str::to_owned)
                    .collect()
            })
            .collect()
    }
    fn text_of(html: &str, id: &str) -> String {
        let at = html.find(&format!(r#"id="{id}""#)).expect(id);
        let open = html[at..].find('>').unwrap() + at + 1;
        let close = html[open..].find('<').unwrap() + open;
        html[open..close].to_owned()
    }

    let html = render(single);
    let described = bubbles(&html);
    assert_eq!(described.len(), 1, "{html}");
    assert_eq!(described[0].len(), 2, "{described:?}");
    assert_eq!(text_of(&html, &described[0][0]), "Loud is 80.");
    assert_eq!(text_of(&html, &described[0][1]), "40");
    let at = html.find(&format!(r#"id="{}""#, described[0][1])).unwrap();
    let bubble = attributes_of(&html[html[..at].rfind('<').unwrap()..], "span");
    assert_eq!(bubble["role"], "tooltip", "{bubble:?}");

    let html = render(range);
    let described = bubbles(&html);
    assert_eq!(described.len(), 2, "{html}");
    assert_eq!(described[0].len(), 1, "{described:?}");
    assert_ne!(described[0], described[1]);
    assert_eq!(text_of(&html, &described[0][0]), "20");
    assert_eq!(text_of(&html, &described[1][0]), "80");
}

/// A single thumb's `aria_label` stands in for a missing `label`, so with both
/// the label names it alone - only a range appends each thumb's own name.
#[test]
fn a_labelled_single_slider_is_named_by_its_label_alone() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider { label: "Quality", aria_label: "Quality", value: 25.0, oninput: move |_| {} }
            }
        }
    }

    let html = render(app);
    let at = html.find(r#"role="slider""#).unwrap();
    let thumb = attributes_of(&html[html[..at].rfind('<').unwrap()..], "span");
    assert!(thumb["aria-labelledby"].ends_with("-label"), "{thumb:?}");
    assert!(!thumb["aria-labelledby"].contains(' '), "{thumb:?}");
    assert!(!thumb.contains_key("id"), "{thumb:?}");
}

/// A hand-written `SliderValue`, which is what a caller with a foreign enum
/// writes - and what covers the trait's default `position`/`at`.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Tier {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl SliderValue for Tier {
    type Step = usize;

    fn options() -> Option<&'static [Self]> {
        Some(&[Self::Free, Self::Pro, Self::Team, Self::Enterprise])
    }

    fn label(&self) -> String {
        format!("{self:?}")
    }
}

/// An ordered enum makes the slider discrete: range, step grid, marks and
/// every caption come from `SliderValue::options`.
#[test]
fn a_discrete_slider_derives_its_scale_from_the_value_type() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Tier::Pro,
                    aria_label: "Tier",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);
    let root = attributes_of(&html[html.find("<div").unwrap() + 4..], "div");

    // `Pro` is the second of four options, so the scale is 0..=3.
    assert!(html.contains("--lsx-slider-filled:0.3333333333333333;"));
    assert!(html.contains("aria-valuenow=1"));
    assert!(html.contains("aria-valuemax=3"));
    assert!(html.contains(r#"aria-valuetext="Pro""#));
    assert!(root["data-state"].contains("marks-labeled"));
    for tier in ["Free", "Pro", "Team", "Enterprise"] {
        assert!(body(&html).contains(&format!(">{tier}<")));
    }
}

/// `min`/`max` are written in the value's own type, and `step` is a stride
/// over the options - `step: 1.5` on a `Tier` slider does not compile.
#[test]
fn a_discrete_sliders_bounds_are_typed_and_its_step_counts_options() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Tier::Team,
                    min: Tier::Pro,
                    max: Tier::Enterprise,
                    step: 2,
                    aria_label: "Tier",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("aria-valuemin=1"));
    assert!(html.contains("aria-valuemax=3"));
    // Marks at `Pro` and `Enterprise` only - every second option from `min`.
    assert!(body(&html).contains(">Pro<"));
    assert!(body(&html).contains(">Enterprise<"));
    assert!(!body(&html).contains(">Free<"));
    assert!(!body(&html).contains(">Team<"));
}

/// `#[derive(SliderValue)]` is the whole discrete impl: variants in
/// declaration order are the options, their names are the labels.
#[derive(Clone, Copy, Debug, PartialEq, SliderValue)]
enum Quality {
    Low,
    #[slider(label = "Med")]
    Medium,
    High,
}

#[test]
fn a_derived_slider_value_names_and_orders_its_own_options() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Quality::Medium,
                    aria_label: "Quality",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains("aria-valuemax=2"));
    assert!(html.contains("aria-valuenow=1"));
    // The overridden label reaches both the bubble and `aria-valuetext`.
    assert!(html.contains(r#"aria-valuetext="Med""#));
    for label in [">Low<", ">Med<", ">High<"] {
        assert!(body(&html).contains(label));
    }
}

#[derive(Clone, PartialEq, SliderValue)]
enum Grade {
    Low,
    High,
}

#[test]
fn a_sliders_format_prop_renames_its_mark_captions_too() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider {
                    value: Grade::Low,
                    oninput: move |_| {},
                    format: |grade: Grade| match grade {
                        Grade::Low => "Niedrig".to_string(),
                        Grade::High => "Hoch".to_string(),
                    },
                }
            }
        }
    }

    let body = body(&render(app));

    // The bubble, `aria-valuetext` and both captions speak one language.
    assert!(body.contains("Niedrig"));
    assert!(body.contains("Hoch"));
    assert!(!body.contains("Low"));
    assert!(!body.contains("High"));
}

/// `format` is the translation hook: it runs during render, so it can read a
/// locale from context, and it names both thumbs of a range.
#[test]
fn a_range_sliders_format_translates_both_thumbs_from_a_context_locale() {
    #[derive(Clone, Copy)]
    struct German(bool);

    fn app() -> Element {
        use_context_provider(|| German(true));
        rsx! {
            LiberoProvider {
                RangeSlider {
                    value: (Grade::Low, Grade::High),
                    oninput: move |_| {},
                    format: |grade: Grade| {
                        let German(german) = consume_context::<German>();
                        match (grade, german) {
                            (Grade::Low, true) => "Niedrig".to_string(),
                            (Grade::High, true) => "Hoch".to_string(),
                            (grade, false) => grade.label(),
                        }
                    },
                }
            }
        }
    }

    let html = render(app);

    assert!(html.contains(r#"aria-valuetext="Niedrig""#), "{html}");
    assert!(html.contains(r#"aria-valuetext="Hoch""#), "{html}");
    assert!(!body(&html).contains("Low"), "{html}");
}

/// `min` defaults to 0, so `max: -10.0` alone is an inverted range - and
/// `f64::clamp` panics on one. A range computed from data is ordinary
/// (`items.len() as f64 - 1.0` is `-1` for an empty list), so it has to warn
/// and draw empty rather than take the page down.
#[test]
fn an_inverted_or_non_finite_range_renders_instead_of_panicking() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Slider { max: -10.0, aria_label: "Empty", oninput: move |_| {} }
                Slider { min: 10.0, max: 0.0, aria_label: "Inverted", oninput: move |_| {} }
                Slider { min: f64::NAN, aria_label: "Broken min", oninput: move |_| {} }
                Slider { value: f64::NAN, aria_label: "Broken value", oninput: move |_| {} }
                RangeSlider {
                    min: 10.0,
                    max: 0.0,
                    aria_label: "Inverted range",
                    oninput: move |_| {},
                }
            }
        }
    }

    let html = render(app);

    // Five sliders, six thumbs: the `RangeSlider` mounts two.
    assert_eq!(html.matches(r#"role="slider""#).count(), 6, "{html}");
    // A collapsed range has no position, and nothing writes `NaN`.
    assert!(!html.contains("NaN"));
    assert!(html.contains("--lsx-slider-filled:0;"));
}

/// A theme whose slider default is `sm`, to tell the theme's step from a
/// hardcoded `md`.
static SMALL_SLIDERS: Theme = Theme {
    slider: SliderDefaults {
        size: Size::Sm,
        ..Theme::DEFAULT.slider
    },
    ..Theme::DEFAULT
};

/// The field wrapper sizes the label and captions, the core sizes the track.
/// Both read the theme's step, so they never disagree.
#[test]
fn both_halves_of_a_slider_take_the_themes_default_size() {
    fn single() -> Element {
        rsx! {
            LiberoProvider { themes: &SMALL_SLIDERS,
                Slider { label: "Volume", value: 25.0, oninput: move |_| {} }
            }
        }
    }
    fn range() -> Element {
        rsx! {
            LiberoProvider { themes: &SMALL_SLIDERS,
                RangeSlider { label: "Price", value: (20.0, 80.0), oninput: move |_| {} }
            }
        }
    }

    for html in [render(single), render(range)] {
        let wrapper = attributes_of(&html, "div");
        let root = attributes_of(&html[html.find("<div").unwrap() + 4..], "div");
        assert!(wrapper["data-state"].contains("size-sm"), "{wrapper:?}");
        assert!(root["data-state"].contains("size-sm"), "{root:?}");
    }
}
