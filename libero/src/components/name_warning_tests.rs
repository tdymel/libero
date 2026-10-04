//! A role that needs an accessible name warns once when it renders without one
//! (`common::use_name_warning`), and a `javascript:` link warns in the shared
//! anchor path. Each case renders the component and reads what `warn()` saw.

use dioxus::prelude::*;

use crate::{
    LiberoProvider,
    components::{
        ActionIcon, AlphaSlider, Anchor, Audio, Avatar, Button, ButtonGroup, Carousel, Checkbox,
        ColorCode, ColorPicker, ColorSwatch, Dialog, Drawer, FloatingWindowOptions, HoverCard,
        HueSlider, ProgressBar, Radio, RadioGroup, RangeSlider, Rating, RichTextEditor, ScrollArea,
        SegmentedControl, Slider, Splitter, SpotlightOptions, Switch, Table, Tabs, Toolbar,
        ToolbarGroup, TourOptions, TourStep, Video, column,
        rich_text::{NodeViewProps, NodeViews, use_rich_text_editor},
        use_spotlight, use_tour,
    },
    hooks::{LightboxItem, LightboxOptions, use_floating_window, use_lightbox},
    utils::{take_warnings, warnings_of},
};

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
fn an_unnamed_button_group_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { ButtonGroup { Button { "Left" } } } },
        "ButtonGroup:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ButtonGroup { "aria-label": "Alignment", Button { "Left" } } } },
        "ButtonGroup:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { ButtonGroup { "aria-labelledby": "heading", Button { "Left" } } } },
        "ButtonGroup:"
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

/// Todo 1593: a group inside a toolbar is announced as just "group" without a name.
#[test]
fn an_unnamed_toolbar_group_warns() {
    let prefix = "ToolbarGroup: no `";
    assert!(warns(
        || rsx! { LiberoProvider { Toolbar { "aria-label": "Format", ToolbarGroup { ActionIcon { aria_label: "Bold", "B" } } } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Toolbar { "aria-label": "Format", ToolbarGroup { "aria-label": "Style", ActionIcon { aria_label: "Bold", "B" } } } } },
        prefix
    ));
}

/// Todo 1593: APG names the tablist; the name lands there, so the warning reads it.
#[test]
fn an_unnamed_tabs_warns() {
    let prefix = "Tabs: no `";
    fn tabs(name: Option<&'static str>, by: Option<&'static str>) -> Element {
        rsx! {
            LiberoProvider {
                Tabs::<String> {
                    aria_label: name,
                    aria_labelledby: by,
                    value: "a".to_string(),
                    options: vec!["a".to_string()],
                    onchange: |_| {},
                    panel: |_| rsx! {},
                }
            }
        }
    }
    assert!(warns(|| tabs(None, None), prefix));
    assert!(!warns(|| tabs(Some("Settings"), None), prefix));
    assert!(!warns(|| tabs(None, Some("heading")), prefix));
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

/// The localization's "Minimum"/"Maximum" stand in, but name no range (1556).
#[test]
fn an_unnamed_range_slider_warns() {
    let prefix = "RangeSlider: no `";
    assert!(warns(
        || rsx! { LiberoProvider { RangeSlider::<f64> { value: (5.0, 50.0) } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { RangeSlider::<f64> { value: (5.0, 50.0), label: "Price" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { RangeSlider::<f64> { value: (5.0, 50.0), aria_label: "Price" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! {
            LiberoProvider {
                RangeSlider::<f64> {
                    value: (5.0, 50.0),
                    aria_label_from: "Lowest price",
                    aria_label_to: "Highest price",
                }
            }
        },
        prefix
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
fn a_radio_group_option_does_not_warn() {
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

#[test]
fn an_unnamed_carousel_warns() {
    let prefix = "Carousel: no `";
    assert!(warns(
        || rsx! { LiberoProvider { Carousel { slides: vec![] } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Carousel { aria_label: "Offers", slides: vec![] } } },
        prefix
    ));
}

#[test]
fn an_unlabelled_audio_or_video_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Audio { src: "/a.mp3", label: "" } } },
        "Audio:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Audio { src: "/a.mp3", label: "Episode 1" } } },
        "Audio:"
    ));
    assert!(warns(
        || rsx! { LiberoProvider { Video { src: "/a.mp4", label: "" } } },
        "Video:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Video { src: "/a.mp4", label: "Trailer" } } },
        "Video:"
    ));
}

#[test]
fn an_unnamed_hue_or_alpha_slider_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { HueSlider { value: 0.0 } } },
        "HueSlider:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { HueSlider { value: 0.0, aria_label: "Hue" } } },
        "HueSlider:"
    ));
    assert!(warns(
        || rsx! { LiberoProvider { AlphaSlider { value: 1.0, color: ColorCode::default() } } },
        "AlphaSlider:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { AlphaSlider { value: 1.0, color: ColorCode::default(), aria_label: "Opacity" } } },
        "AlphaSlider:"
    ));
}

/// The picker names its own sliders.
#[test]
fn a_color_pickers_sliders_do_not_warn() {
    let warnings = warnings_of(
        || rsx! { LiberoProvider { ColorPicker { value: ColorCode::default(), with_alpha: true } } },
    );
    assert!(
        !warnings
            .iter()
            .any(|warning| warning.starts_with("HueSlider:") || warning.starts_with("AlphaSlider:")),
        "{warnings:?}"
    );
}

#[test]
fn an_unnamed_avatar_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Avatar { name: "" } } },
        "Avatar:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Avatar { name: "Ada" } } },
        "Avatar:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Avatar { name: "", alt: "" } } },
        "Avatar:"
    ));
}

#[test]
fn an_unnamed_table_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { Table { data: vec![1u32], columns: vec![column("N").value(|n: &u32| *n)] } } },
        "Table:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Table { caption: "Numbers", data: vec![1u32], columns: vec![column("N").value(|n: &u32| *n)] } } },
        "Table:"
    ));
}

#[test]
fn an_unnamed_rich_text_editor_warns() {
    assert!(warns(
        || rsx! { LiberoProvider { RichTextEditor {} } },
        "RichTextEditor:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { RichTextEditor { label: "Notes" } } },
        "RichTextEditor:"
    ));
}

#[test]
fn an_unnamed_floating_window_warns() {
    #[component]
    fn Window(named: bool) -> Element {
        let window = use_floating_window(
            FloatingWindowOptions {
                title: named.then(|| "Inspector".to_string()),
                ..Default::default()
            },
            |_| rsx! { "x" },
        );
        use_hook(|| window.open());
        rsx! {}
    }
    assert!(warns(
        || rsx! { LiberoProvider { Window { named: false } } },
        "FloatingWindow:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Window { named: true } } },
        "FloatingWindow:"
    ));
}

#[test]
fn an_unnamed_hover_card_warns() {
    let prefix = "HoverCard: the card is a dialog";
    assert!(warns(
        || rsx! { LiberoProvider { HoverCard { content: rsx! { "c" }, "x" } } },
        prefix
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { HoverCard { content: rsx! { "c" }, "aria-label": "Profile", "x" } } },
        prefix
    ));
}

/// The card falls back to the localized "Tour", which names no step.
#[test]
fn an_untitled_tour_step_without_an_aria_label_warns() {
    #[component]
    fn Tour(named: bool) -> Element {
        let tour = use_tour(TourOptions {
            steps: vec![TourStep::new("a").description("Untitled")],
            aria_label: named.then(|| "Getting started".to_string()),
            ..Default::default()
        });
        use_hook(|| tour.start());
        rsx! {}
    }
    assert!(warns(
        || rsx! { LiberoProvider { Tour { named: false } } },
        "use_tour:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Tour { named: true } } },
        "use_tour:"
    ));
}

/// One handle drives one editor: a second editor taking it over warns.
#[test]
fn a_rich_text_handle_shared_by_two_editors_warns() {
    #[component]
    fn Editors(two: bool) -> Element {
        let editor = use_rich_text_editor();
        rsx! {
            RichTextEditor { label: "A", handle: editor }
            if two {
                RichTextEditor { label: "B", handle: editor }
            }
        }
    }
    assert!(warns(
        || rsx! { LiberoProvider { Editors { two: true } } },
        "RichTextHandle:"
    ));
    assert!(!warns(
        || rsx! { LiberoProvider { Editors { two: false } } },
        "RichTextHandle:"
    ));
}

#[test]
fn a_code_block_node_view_warns() {
    fn view(_: NodeViewProps) -> Element {
        rsx! {}
    }
    take_warnings();
    let _ = NodeViews::new().with("callout", view);
    assert!(take_warnings().is_empty());
    let _ = NodeViews::new().with("code_block", view);
    assert!(
        take_warnings()
            .iter()
            .any(|warning| warning.starts_with("NodeViews:"))
    );
}
