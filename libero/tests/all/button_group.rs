//! `ButtonGroup`'s rendered contract: a named `group` whose defaults reach the
//! buttons inside, each button's own prop winning, and seams in the CSS.

use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{ActionIcon, Button, ButtonGroup, DirectionToggle},
};

fn states(html: &str, from: &str) -> String {
    let at = html
        .find(from)
        .unwrap_or_else(|| panic!("no {from} in {html}"));
    attributes_of(&html[at..], "button")
        .remove("data-state")
        .unwrap_or_default()
}

#[test]
fn the_group_is_a_named_horizontal_group() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { "aria-label": "Alignment",
                    Button { "Left" }
                }
            }
        }
    }
    let html = body(&render(app));
    let group = attributes_of(&html, "div");
    assert_eq!(group["role"], "group");
    assert_eq!(group["aria-label"], "Alignment");
    assert!(group["data-state"].contains("horizontal"), "{group:?}");
}

#[test]
fn the_defaults_reach_each_button_and_its_own_prop_wins() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { variant: "filled", size: "lg", radius: "xl", disabled: true,
                    Button { "One" }
                    Button { variant: "outlined", size: "xs", disabled: false, "Two" }
                    ActionIcon { aria_label: "Three", "3" }
                }
            }
        }
    }
    let html = body(&render(app));
    let has = |states: &str, state: &str| states.split(' ').any(|s| s == state);

    let one = states(&html, "<button");
    for state in ["filled", "size-lg", "radius-xl", "disabled"] {
        assert!(has(&one, state), "{state}: {one}");
    }
    let two = states(&html[html.find("One").unwrap()..], "<button");
    assert!(has(&two, "outlined") && has(&two, "size-xs"), "{two}");
    assert!(!has(&two, "disabled"), "{two}");

    let icon = attributes_of(&html[html.find("Two").unwrap()..], "button");
    assert!(has(&icon["data-state"], "filled"), "{icon:?}");
    assert_eq!(icon["aria-disabled"], "true", "{icon:?}");
}

/// Todo 2815: a disabled group takes its buttons out of the Tab order, bar one that keeps it.
#[test]
fn a_button_keeps_its_tab_stop_in_a_disabled_group_by_its_own_prop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { disabled: true,
                    Button { "One" }
                    Button { focusable_when_disabled: true, "Two" }
                    ActionIcon { aria_label: "Three", focusable_when_disabled: true, "3" }
                }
            }
        }
    }
    let html = body(&render(app));

    let buttons: Vec<_> = html
        .split("<button")
        .skip(1)
        .map(|button| attributes_of(&format!("<button{button}"), "button"))
        .collect();
    assert_eq!(buttons.len(), 3, "{html}");
    assert_eq!(buttons[0]["tabindex"], "-1", "{:?}", buttons[0]);
    for soft in &buttons[1..] {
        assert!(!soft.contains_key("tabindex"), "{soft:?}");
    }
    // Todo 3838: never native, so a focused button keeps its focus.
    for button in &buttons {
        assert!(!button.contains_key("disabled"), "{button:?}");
        assert_eq!(button["aria-disabled"], "true", "{button:?}");
    }
}

/// Todo 3170: the group's `focusable_when_disabled` is every button's default, its own prop wins.
#[test]
fn a_disabled_group_forwards_focusable_when_disabled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { disabled: true, focusable_when_disabled: true,
                    Button { "One" }
                    ActionIcon { aria_label: "Two", "2" }
                    Button { focusable_when_disabled: false, "Three" }
                }
            }
        }
    }
    let html = body(&render(app));

    let buttons: Vec<_> = html
        .split("<button")
        .skip(1)
        .map(|button| attributes_of(&format!("<button{button}"), "button"))
        .collect();
    assert_eq!(buttons.len(), 3, "{html}");
    for soft in &buttons[..2] {
        assert!(!soft.contains_key("tabindex"), "{soft:?}");
        assert_eq!(soft["aria-disabled"], "true", "{soft:?}");
    }
    assert_eq!(buttons[2]["tabindex"], "-1", "{:?}", buttons[2]);
}

#[test]
fn a_forwarding_toggle_takes_the_group_variant() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ButtonGroup { variant: "tonal", DirectionToggle {} }
            }
        }
    }
    let html = body(&render(app));
    let toggle = attributes_of(&html, "button");
    assert!(toggle["data-state"].contains("tonal"), "{toggle:?}");
}

#[test]
fn outside_a_group_nothing_changes() {
    fn app() -> Element {
        rsx! { LiberoProvider { ActionIcon { aria_label: "Bare", "x" } } }
    }
    let html = body(&render(app));
    let icon = attributes_of(&html, "button");
    assert!(
        !icon
            .get("data-state")
            .is_some_and(|s| s.contains("outlined")),
        "chromeless without a variant: {icon:?}"
    );
}

#[test]
fn the_css_divides_the_inner_seam_of_a_theme_switcher_pair() {
    fn app() -> Element {
        rsx! { LiberoProvider { ButtonGroup { Button { "A" } } } }
    }
    let html = render(app);
    assert!(
        html.contains("> [role=\"group\"] > div > :is(button, a):not([data-state~=\"outlined\"])"),
        "{html}"
    );
}

#[test]
fn the_css_joins_the_seams_on_logical_sides() {
    fn app() -> Element {
        rsx! { LiberoProvider { ButtonGroup { Button { "A" } Button { "B" } } } }
    }
    let html = render(app);
    for rule in [
        ":not(:first-child):is(button, a)",
        "margin-inline-start:-1px",
        "border-start-start-radius:0",
        "border-end-end-radius:0",
        ":not([data-state~=\"outlined\"])",
    ] {
        assert!(html.contains(rule), "{rule}");
    }
}

/// Todo 3998: `value` makes the valued buttons toggles, pressed through the real events.
mod toggles {
    use crate::common::{body, nth_attributes};
    use crate::dispatch::*;

    use std::collections::HashSet;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{ActionIcon, Button, ButtonGroup},
    };

    #[derive(Clone)]
    struct Setup {
        controlled: bool,
        multiple: bool,
        initial: &'static [&'static str],
    }

    fn app(setup: Setup) -> Element {
        let mut selected = use_signal(|| {
            setup
                .initial
                .iter()
                .map(ToString::to_string)
                .collect::<HashSet<_>>()
        });
        rsx! {
            LiberoProvider {
                ButtonGroup {
                    "aria-label": "Style",
                    value: setup.controlled.then(|| selected.cloned()),
                    multiple: setup.multiple,
                    onchange: move |next| selected.set(next),
                    Button { value: "a", "A" }
                    Button { value: "b", "B" }
                    ActionIcon { value: "c", aria_label: "C", "C" }
                    Button { "Plain" }
                }
            }
        }
    }

    fn page(controlled: bool, multiple: bool, initial: &'static [&'static str]) -> Page {
        let mut page = Page::build_with_props(
            app,
            Setup {
                controlled,
                multiple,
                initial,
            },
        );
        page.render();
        page
    }

    /// Each button's `aria-pressed`, in order; `-` where it has none.
    fn pressed(page: &Page) -> Vec<String> {
        let html = body(&dioxus_ssr::render(&page.dom));
        (0..4)
            .map(|n| {
                nth_attributes(&html, "button", n)
                    .remove("aria-pressed")
                    .unwrap_or_else(|| "-".into())
            })
            .collect()
    }

    fn press(page: &mut Page, value: &str) {
        let button = page.rec.element("click", "value", value);
        page.click(button);
    }

    #[test]
    fn a_value_makes_the_valued_buttons_toggles() {
        let page = page(true, false, &["b"]);

        assert_eq!(pressed(&page), ["false", "true", "false", "-"]);
    }

    #[test]
    fn without_a_value_the_group_stays_plain() {
        let mut page = page(false, false, &["b"]);
        press(&mut page, "a");

        assert_eq!(pressed(&page), ["-", "-", "-", "-"]);
    }

    #[test]
    fn a_single_group_swaps_the_pressed_button_and_a_second_press_releases_it() {
        let mut page = page(true, false, &["b"]);

        press(&mut page, "a");
        assert_eq!(pressed(&page), ["true", "false", "false", "-"]);
        press(&mut page, "c");
        assert_eq!(pressed(&page), ["false", "false", "true", "-"]);
        press(&mut page, "c");
        assert_eq!(pressed(&page), ["false", "false", "false", "-"]);
    }

    #[test]
    fn a_multiple_group_presses_and_releases_each_button_alone() {
        let mut page = page(true, true, &["b"]);

        press(&mut page, "a");
        press(&mut page, "c");
        assert_eq!(pressed(&page), ["true", "true", "true", "-"]);
        press(&mut page, "b");
        assert_eq!(pressed(&page), ["true", "false", "true", "-"]);
    }

    fn own_selected() -> Element {
        let mut selected = use_signal(|| HashSet::from(["b".to_string()]));
        rsx! {
            LiberoProvider {
                ButtonGroup {
                    "aria-label": "Style",
                    value: selected(),
                    onchange: move |next| selected.set(next),
                    Button { value: "a", selected: false, "A" }
                    Button { value: "b", "B" }
                }
            }
        }
    }

    /// A button's own `selected` is not the group's to toggle, and the others stay as they were.
    #[test]
    fn a_buttons_own_selected_wins_and_the_group_does_not_toggle_it() {
        let mut page = Page::mount(own_selected);
        let button = page.rec.element("click", "value", "a");
        page.click(button);

        let html = body(&dioxus_ssr::render(&page.dom));
        assert_eq!(nth_attributes(&html, "button", 0)["aria-pressed"], "false");
        assert_eq!(nth_attributes(&html, "button", 1)["aria-pressed"], "true");
    }
}
