//! A role that needs an accessible name warns once when it renders without one
//! (`common::use_name_warning`), and a `javascript:` link warns in the shared
//! anchor path. Each case renders the component and reads what `warn()` saw.

use dioxus::prelude::*;

use crate::{
    LiberoProvider,
    components::{
        ActionIcon, Anchor, Checkbox, ColorCode, ColorSwatch, Dialog, Drawer, ProgressBar, Radio,
        RadioGroup, Rating, ScrollArea, SegmentedControl, Slider, Splitter, SpotlightOptions,
        Switch, Toolbar, use_spotlight,
    },
    hooks::{LightboxItem, LightboxOptions, use_lightbox},
    utils::take_warnings,
};

fn warnings_of(app: fn() -> Element) -> Vec<String> {
    take_warnings();
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    take_warnings()
}

fn warns(app: fn() -> Element, component: &str) -> bool {
    warnings_of(app)
        .iter()
        .any(|warning| warning.starts_with(component))
}

#[test]
fn an_unnamed_progress_bar_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { ProgressBar { value: 40.0 } } },
        "ProgressBar:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ProgressBar { value: 40.0, aria_label: "Upload" } } },
        "ProgressBar:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ProgressBar { value: None, aria_labelledby: "heading" } } },
        "ProgressBar:"
    ));
}

/// Once per mount, not per render: a bar ticking every frame would flood.
#[test]
fn the_name_warning_does_not_repeat_on_a_re_render() {
    take_warnings();
    let mut dom = VirtualDom::new(|| {
        let mut value = use_signal(|| 0.0);
        use_hook(move || value.set(50.0));
        rsx! { LiberoProvider { ProgressBar { value: value() } } }
    });
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);

    let count = take_warnings()
        .iter()
        .filter(|warning| warning.starts_with("ProgressBar:"))
        .count();
    assert_eq!(count, 1);
}

/// Only a tab stop needs a name; an area that is none stays quiet.
#[test]
fn an_unnamed_scroll_area_tab_stop_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { ScrollArea { focusable: true, "x" } } },
        "ScrollArea:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ScrollArea { focusable: true, aria_label: "Terms", "x" } } },
        "ScrollArea:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ScrollArea { "x" } } },
        "ScrollArea:"
    ));
}

#[test]
fn an_unnamed_splitter_divider_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Splitter { initial_size: 50.0, panel_a: rsx! {}, panel_b: rsx! {} } } },
        "Splitter:"
    ));
    assert!(!warns(
        || rsx! {
            LiberoProvider {
                Splitter {
                    initial_size: 50.0,
                    aria_label: "Resize sidebar",
                    panel_a: rsx! {},
                    panel_b: rsx! {},
                }
            }
        },
        "Splitter:"
    ));
}

#[test]
fn an_unnamed_toolbar_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Toolbar { ActionIcon { aria_label: "Bold", "B" } } } },
        "Toolbar:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Toolbar { "aria-label": "Format", ActionIcon { aria_label: "Bold", "B" } } } },
        "Toolbar:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Toolbar { "aria-labelledby": "heading", ActionIcon { aria_label: "Bold", "B" } } } },
        "Toolbar:"
    ));
}

#[test]
fn an_unnamed_dialog_warns() {
    assert!(warns(|| rsx! { Dialog { "Body" } }, "Dialog:"));
    assert!(!warns(
        || rsx! { Dialog { title: "Settings", "Body" } },
        "Dialog:"
    ));
    assert!(!warns(
        || rsx! { Dialog { aria_label: "Settings", "Body" } },
        "Dialog:"
    ));
    assert!(!warns(
        || rsx! { Dialog { aria_labelledby: "heading", "Body" } },
        "Dialog:"
    ));
}

#[test]
fn an_unnamed_drawer_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Drawer { "Nav" } } },
        "Dialog:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Drawer { aria_label: "Navigation", "Nav" } } },
        "Dialog:"
    ));
}

#[test]
fn an_unnamed_slider_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Slider::<f64> { value: 5.0 } } },
        "Slider: no `"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Slider::<f64> { value: 5.0, label: "Volume" } } },
        "Slider: no `"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Slider::<f64> { value: 5.0, aria_label: "Volume" } } },
        "Slider: no `"
    ));
}

#[test]
fn an_unnamed_rating_warns_and_a_fixed_one_says_it_cannot_change() {
    assert!(warns(
        || rsx! { LiberoProvider { Rating { value: 3.0, readonly: true } } },
        "Rating: no `"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Rating { value: 3.0, readonly: true, label: "Stars" } } },
        "Rating: no `"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Rating { value: 3.0, focusable: false, aria_label: "Average" } } },
        "Rating:"
    ));
    assert!(warns(
        || rsx! { LiberoProvider { Rating { value: 3.0, aria_label: "Stars" } } },
        "Rating: without `onchange`"
    ));
}

/// The localization's label stands in, but the warning still asks for a real name,
/// like `Carousel`'s.
#[test]
fn an_unnamed_lightbox_warns() {
    #[component]
    fn Gallery(named: bool) -> Element {
        let lightbox = use_lightbox(LightboxOptions {
            aria_label: named.then(|| "Holiday photos".to_string()),
            ..LightboxOptions::default()
        });
        use_hook(move || lightbox.open_with(LightboxItem::new("a.png", "A beach")));
        rsx! {}
    }
    assert!(warns(
        || rsx! { LiberoProvider { Gallery { named: false } } },
        "Lightbox:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Gallery { named: true } } },
        "Lightbox:"
    ));
}

/// Todo 567: a picture is a tab stop, so an empty `alt` is a nameless one.
#[test]
fn a_lightbox_item_without_alt_warns() {
    #[component]
    fn Gallery(alt: &'static str) -> Element {
        let lightbox = use_lightbox(LightboxOptions {
            aria_label: Some("Holiday photos".to_string()),
            ..LightboxOptions::default()
        });
        use_hook(move || {
            lightbox.open_with(vec![
                LightboxItem::new("a.png", "A beach"),
                LightboxItem::new("b.png", alt),
            ])
        });
        rsx! {}
    }
    let empty_alt = "Lightbox: a `LightboxItem` has an empty `alt`";
    assert!(warns(
        || rsx! { LiberoProvider { Gallery { alt: " " } } },
        empty_alt
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Gallery { alt: "A dune" } } },
        empty_alt
    ));
}

#[test]
fn an_unnamed_spotlight_warns() {
    #[component]
    fn Palette(named: bool) -> Element {
        use_spotlight(SpotlightOptions {
            actions: Some(Callback::new(|_: String| Vec::new())),
            aria_label: named.then(|| "Commands".to_string()),
            ..SpotlightOptions::default()
        });
        rsx! {}
    }
    assert!(warns(
        || rsx! { LiberoProvider { Palette { named: false } } },
        "use_spotlight: no `aria_label`"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Palette { named: true } } },
        "use_spotlight: no `aria_label`"
    ));
}

#[test]
fn a_javascript_link_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Anchor { to: "javascript:alert(1)", "Site" } } },
        "Link to"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Anchor { to: "https://example.com", "Site" } } },
        "Link to"
    ));
}

#[test]
fn an_unnamed_switch_warns() {
    let prefix = "Switch: no `";
    assert!(warns(|| rsx! { LiberoProvider { Switch {} } }, prefix));
    assert!(!warns(
        || rsx! { LiberoProvider { Switch { label: "Wi-Fi" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Switch { aria_label: "Wi-Fi" } } },
        prefix
    ));
}

/// `aria_label` is required, but an empty one names nothing.
#[test]
fn an_action_icon_with_an_empty_label_warns() {
    let prefix = "ActionIcon: an empty `aria_label`";
    assert!(warns(
        || rsx! { LiberoProvider { ActionIcon { aria_label: " ", "x" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ActionIcon { aria_label: "Copy", "x" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ActionIcon { aria_label: "", "aria-labelledby": "copy", "x" } } },
        prefix
    ));
}

/// Todo 1549: only the clickable swatch is a button; a plain one is a picture.
#[test]
fn an_unnamed_clickable_swatch_warns() {
    let prefix = "ColorSwatch:";
    assert!(warns(
        || rsx! { LiberoProvider { ColorSwatch { color: ColorCode::default(), onclick: |_| {} } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ColorSwatch { color: ColorCode::default(), onclick: |_| {}, aria_label: "Blue" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ColorSwatch { color: ColorCode::default() } } },
        prefix
    ));
}

#[test]
fn an_unnamed_checkbox_warns() {
    let prefix = "Checkbox: no `";
    assert!(warns(|| rsx! { LiberoProvider { Checkbox {} } }, prefix));
    assert!(!warns(
        || rsx! { LiberoProvider { Checkbox { label: "Terms" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Checkbox { aria_label: "Terms" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Checkbox { "aria-labelledby": "terms" } } },
        prefix
    ));
}

#[test]
fn an_unnamed_radio_warns() {
    let prefix = "Radio: no `";
    assert!(warns(|| rsx! { LiberoProvider { Radio {} } }, prefix));
    assert!(!warns(
        || rsx! { LiberoProvider { Radio { label: "Monthly" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Radio { aria_label: "Monthly" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Radio { "aria-labelledby": "monthly" } } },
        prefix
    ));
}

/// A group's options carry their labels, so none of them warns.
#[test]
fn a_named_radio_group_warns_about_no_radio() {
    assert!(!warns(
        || rsx! { LiberoProvider { RadioGroup::<String> { label: "Plan", options: vec!["a".to_string()] } } },
        "Radio: no `"
    ));
}

#[test]
fn an_unnamed_radio_group_warns() {
    let prefix = "RadioGroup: no `";
    assert!(warns(
        || rsx! { LiberoProvider { RadioGroup::<String> { options: vec!["a".to_string()] } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { RadioGroup::<String> { label: "Plan", options: vec!["a".to_string()] } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { RadioGroup::<String> { "aria-label": "Plan", options: vec!["a".to_string()] } } },
        prefix
    ));
}

#[test]
fn an_unnamed_segmented_control_warns() {
    let prefix = "SegmentedControl: no `";
    assert!(warns(
        || rsx! { LiberoProvider { SegmentedControl::<String> { options: vec!["a".to_string()] } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { SegmentedControl::<String> { label: "Align", options: vec!["a".to_string()] } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { SegmentedControl::<String> { "aria-label": "Align", options: vec!["a".to_string()] } } },
        prefix
    ));
}
