//! A role that needs an accessible name warns once when it renders without one
//! (`utils::use_name_warning`), and a `javascript:` link warns in the shared
//! anchor path. Each case renders the component and reads what `warn()` saw.

use dioxus::prelude::*;

use crate::{
    LiberoProvider,
    components::{Anchor, Dialog, Drawer, ProgressBar, Slider, Splitter},
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

/// A drawer's dialog cannot be named yet, so it must not nag.
#[test]
fn a_drawer_does_not_warn_about_a_name_it_cannot_take() {
    assert!(!warns(
        || rsx! {
            LiberoProvider {
                Drawer { "Nav" }
            }
        },
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
