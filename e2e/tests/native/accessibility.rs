//! Blitz's stylo matches none of the accessibility media features, so libero
//! answers them itself from the system and the app's overrides (todo 954).

use dioxus::prelude::*;
use e2e::native::mount;
use libero::components::Box;
use libero::hooks::use_accessibility;
use libero::sx::sx;
use libero::theme::{AccessibilityOverrides, Contrast};

fn app() -> Element {
    let accessibility = use_accessibility();
    let set = move |overrides: AccessibilityOverrides| {
        let accessibility = accessibility.clone();
        move |_| accessibility.set_overrides(overrides)
    };
    rsx! {
        button { id: "off", onclick: set(AccessibilityOverrides {
            reduced_motion: Some(false),
            forced_colors: Some(false),
            contrast: Some(Contrast::NoPreference),
            reduced_transparency: Some(false),
        }) }
        button { id: "motion", onclick: set(AccessibilityOverrides {
            reduced_motion: Some(true),
            ..AccessibilityOverrides::default()
        }) }
        button { id: "transparency", onclick: set(AccessibilityOverrides {
            reduced_transparency: Some(true),
            ..AccessibilityOverrides::default()
        }) }
        button { id: "contrast", onclick: set(AccessibilityOverrides {
            contrast: Some(Contrast::More),
            ..AccessibilityOverrides::default()
        }) }
        button { id: "forced", onclick: set(AccessibilityOverrides {
            forced_colors: Some(true),
            ..AccessibilityOverrides::default()
        }) }
        Box {
            id: "motion-box",
            sx: sx()
                .width("10px")
                .media("(prefers-reduced-motion: reduce)", sx().width("20px"))
                .media("(prefers-reduced-motion: no-preference)", sx().height("10px"))
                .media("(min-width: 1px) and (prefers-contrast: more)", sx().height("30px")),
        }
        Box {
            id: "surface-box",
            sx: sx()
                .opacity("0.5")
                .media("(prefers-reduced-transparency: reduce)", sx().opacity("1"))
                .media("(forced-colors: active)", sx().opacity("0.25")),
        }
    }
}

#[test]
fn an_override_answers_the_media_features() {
    let mut page = mount(app);
    page.click("#off");
    assert_eq!(page.computed("#motion-box", "width"), "10px");
    assert_eq!(page.computed("#motion-box", "height"), "10px");
    assert_eq!(page.computed("#surface-box", "opacity"), "0.5");

    page.click("#motion");
    assert_eq!(page.computed("#motion-box", "width"), "20px");
    assert_eq!(page.computed("#motion-box", "height"), "auto");

    page.click("#contrast");
    assert_eq!(page.computed("#motion-box", "height"), "30px");

    page.click("#transparency");
    assert_eq!(page.computed("#surface-box", "opacity"), "1");

    page.click("#forced");
    assert_eq!(page.computed("#surface-box", "opacity"), "0.25");
}
