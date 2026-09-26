//! `Rating`'s markup: one `role="slider"` row with the value spoken, a
//! display-only image, and the posted value.

use crate::common::{attributes_of, render};

use dioxus::prelude::*;
use libero::{
    IconProvider, IconSet, IconSlot, LiberoProvider,
    components::{Fields, Form, Rating, SvgData},
    localization::{Formats, Localization},
};

/// The element carrying `role`, its attributes.
fn root_of(html: &str, role: &str) -> std::collections::BTreeMap<String, String> {
    let at = html.find(&format!(r#"role="{role}""#)).expect(role);
    attributes_of(&html[html[..at].rfind('<').unwrap()..], "div")
}

#[test]
fn a_rating_is_one_slider_with_its_value_spoken() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Rating { label: "Stars", value: 3.5, fractions: 2, onchange: move |_| {} }
            }
        }
    }
    let html = render(app);
    let root = root_of(&html, "slider");
    assert_eq!(root["tabindex"], "0");
    assert_eq!(root["aria-valuemin"], "0");
    assert_eq!(root["aria-valuemax"], "5");
    assert_eq!(root["aria-valuenow"], "3.5");
    assert_eq!(root["aria-valuetext"], "3.5 of 5");
    assert!(root["aria-labelledby"].ends_with("-label"), "{root:?}");
    assert!(root["data-state"].contains("editable"), "{root:?}");
    assert!(!root.contains_key("aria-required"), "{root:?}");
    assert_eq!(html.matches(r#"role="slider""#).count(), 1);

    // Five symbols, two zones each; the fourth is half filled.
    assert_eq!(html.matches(r#"data-slot="symbol""#).count(), 5);
    assert_eq!(html.matches(r#"data-slot="zone""#).count(), 10);
    assert_eq!(html.matches(r#"style="width: 100%""#).count(), 3);
    assert_eq!(html.matches(r#"style="width: 50%""#).count(), 1);
}

#[test]
fn format_and_the_localization_word_the_value() {
    fn formatted() -> Element {
        rsx! {
            LiberoProvider {
                Rating {
                    aria_label: "Stars",
                    value: 2.0,
                    format: Callback::new(|value: f64| format!("{value} stars")),
                    onchange: move |_| {},
                }
            }
        }
    }
    fn german() -> Element {
        rsx! {
            LiberoProvider { localization: &Localization::GERMAN, formats: &Formats::GERMAN,
                Rating { aria_label: "Sterne", value: 2.5, count: 10, onchange: move |_| {} }
            }
        }
    }
    assert_eq!(
        root_of(&render(formatted), "slider")["aria-valuetext"],
        "2 stars"
    );
    assert_eq!(
        root_of(&render(german), "slider")["aria-valuetext"],
        "2,5 von 10"
    );
}

#[test]
fn a_required_rating_says_so_in_its_label() {
    fn english() -> Element {
        rsx! {
            LiberoProvider {
                Rating { label: "Stars", required: true, value: 2.0, onchange: move |_| {} }
            }
        }
    }
    fn german() -> Element {
        rsx! {
            LiberoProvider { localization: &Localization::GERMAN,
                Rating { label: "Sterne", required: true, value: 2.0, onchange: move |_| {} }
            }
        }
    }
    // A slider takes no `aria-required`, so the label's text carries it.
    assert!(render(english).contains(" required<"), "no spoken word");
    assert!(render(german).contains(" erforderlich<"), "no spoken word");
}

#[test]
fn read_only_stays_a_tab_stop_and_draws_no_zones() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Rating { aria_label: "Stars", value: 4.0, readonly: true }
            }
        }
    }
    let html = render(app);
    let root = root_of(&html, "slider");
    assert_eq!(root["tabindex"], "0");
    assert_eq!(root["aria-readonly"], "true");
    assert!(!html.contains(r#"data-slot="zone""#), "{html}");
}

#[test]
fn disabled_leaves_the_tab_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Rating { aria_label: "Stars", value: 4.0, disabled: true, onchange: move |_| {} }
            }
        }
    }
    let root = root_of(&render(app), "slider");
    assert_eq!(root["tabindex"], "-1");
    assert_eq!(root["aria-disabled"], "true");
}

#[test]
fn unfocusable_is_an_image_named_by_its_value() {
    fn labelled() -> Element {
        rsx! {
            LiberoProvider {
                Rating { label: "Average", value: 4.3, focusable: false }
            }
        }
    }
    fn named() -> Element {
        rsx! {
            LiberoProvider {
                Rating { aria_label: "Average", value: 4.25, focusable: false, name: "average" }
            }
        }
    }
    let html = render(labelled);
    assert!(!html.contains(r#"role="slider""#), "{html}");
    let root = root_of(&html, "img");
    assert!(!root.contains_key("tabindex"), "{root:?}");
    assert_eq!(root["aria-label"], "4.3 of 5");
    // The label, then the image's own value.
    let parts: Vec<_> = root["aria-labelledby"].split(' ').collect();
    assert!(parts[0].ends_with("-label"), "{root:?}");
    assert_eq!(parts[1], root["id"]);

    let html = render(named);
    assert_eq!(root_of(&html, "img")["aria-label"], "Average, 4.25 of 5");
    assert!(!html.contains("<input"), "nothing to post: {html}");
}

#[test]
fn a_named_rating_posts_its_value() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Rating { aria_label: "Stars", value: 2.5, fractions: 2, name: "stars", onchange: move |_| {} }
            }
        }
    }
    let html = render(app);
    let input = attributes_of(&html[html.find("<input").unwrap()..], "input");
    assert_eq!(input["type"], "hidden");
    assert_eq!(input["name"], "stars");
    assert_eq!(input["value"], "2.5");
}

#[derive(Clone, PartialEq, Default, Fields)]
pub struct Review {
    pub stars: f64,
}

#[test]
fn a_form_binds_the_value_by_path() {
    fn app() -> Element {
        let review = use_store(|| Review { stars: 4.0 });
        rsx! {
            LiberoProvider {
                Form { value: review,
                    Rating { aria_label: "Stars", name: Review::FIELDS.stars() }
                }
            }
        }
    }
    let html = render(app);
    let root = root_of(&html, "slider");
    assert_eq!(root["aria-valuenow"], "4");
    assert!(root["data-state"].contains("editable"), "{root:?}");
    assert!(html.contains(r#"name="stars""#), "{html}");
}

#[test]
fn the_star_slot_and_the_icon_prop_draw_the_symbol() {
    const MINE: SvgData = SvgData::new(r#"<svg viewBox="0 0 24 24"><path d="M1 2h3"/></svg>"#);
    const PROP: SvgData = SvgData::new(r#"<svg viewBox="0 0 24 24"><path d="M5 6h7"/></svg>"#);
    fn slot() -> Element {
        rsx! {
            LiberoProvider {
                IconProvider { icons: IconSet::new().with(IconSlot::Star, MINE),
                    Rating { aria_label: "Stars", value: 1.0, count: 2, onchange: move |_| {} }
                }
            }
        }
    }
    fn prop() -> Element {
        rsx! {
            LiberoProvider {
                IconProvider { icons: IconSet::new().with(IconSlot::Star, MINE),
                    Rating { aria_label: "Stars", value: 1.0, count: 2, icon: PROP, onchange: move |_| {} }
                }
            }
        }
    }
    let html = render(slot);
    // Two empty glyphs plus one filled.
    assert_eq!(html.matches(MINE.body).count(), 3, "{html}");
    assert_eq!(html.matches(r#"data-slot="fill""#).count(), 1, "{html}");
    let html = render(prop);
    assert_eq!(html.matches(PROP.body).count(), 3, "{html}");
    assert!(!html.contains(MINE.body), "{html}");
}
