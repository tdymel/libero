use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Tooltip},
    theme::Size,
};

#[test]
fn tooltip_wraps_its_trigger_and_labels_it() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip {
                    label: rsx! { "Copy" },
                    label_id: "copy-tip",
                    open_delay: 300,
                    gap: Size::Sm,
                    Button { "C" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);
    let wrapper = attributes_of(&body, "span");
    // The wrapper is the first `<span>`, the bubble the last.
    let bubble = attributes_of(&body[body.rfind("<span").expect("the bubble")..], "span");

    assert_eq!(bubble["role"], "tooltip");
    assert_eq!(bubble["id"], "copy-tip");
    assert_eq!(bubble["data-state"], "placement-top size-sm");
    assert!(wrapper["style"].contains("--lsx-tooltip-open-delay:300ms;"));
    assert!(wrapper["style"].contains("--lsx-tooltip-gap:var(--lsx-spacing-sm);"));
    // The trigger stays a real button inside the wrapper.
    assert!(body.contains(">C<"));

    // A stretched wrapper centres the bubble on the container, not the
    // trigger - which is what a `Flex` column parent does by default.
    let wrapper_class = classes_of(&html, "span")
        .into_iter()
        .find(|class| html.contains(&format!(".{class}:hover")))
        .expect("the wrapper class");
    assert!(html.contains(&format!(
        ".{wrapper_class}{{position:relative;display:inline-block;width:max-content;"
    )));
}

/// The whole component is these two rules: hover/focus opens the bubble, and
/// an explicit `opened` has to be able to beat them at equal specificity - so
/// it must come *later* in the sheet.
#[test]
fn tooltip_opens_on_hover_and_a_controlled_state_wins_by_source_order() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip { label: rsx! { "t" }, opened: true, "x" }
            }
        }
    }

    let html = render(app);
    let wrapper = classes_of(&html, "span")
        .into_iter()
        .find(|class| html.contains(&format!(".{class}:hover")))
        .expect("the wrapper class");

    let hover = html
        .find(&format!(".{wrapper}:hover > [role=\"tooltip\"]"))
        .expect("a hover rule");
    let focus = html
        .find(&format!(".{wrapper} > :focus-visible ~ [role=\"tooltip\"]"))
        .expect("a keyboard-focus rule");
    let opened = html
        .find(&format!(
            ".{wrapper}[data-state~=\"opened\"] > [role=\"tooltip\"]"
        ))
        .expect("an opened rule");

    assert!(
        opened > hover && opened > focus,
        "the override must sort last"
    );
    assert_eq!(attributes_of(&html, "span")["data-state"], "opened");
}

/// Todo 237: a hidden bubble is still an absolutely positioned box, and that
/// widens its scroll container. At rest it is scaled to nothing, and it grows
/// back only as it becomes visible. Measured in Chromium on the Avatar page;
/// this pins the rules that measurement depends on.
#[test]
fn a_hidden_bubble_is_scaled_to_nothing() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tooltip { label: rsx! { "t" }, "x" }
            }
        }
    }

    let html = render(app);
    let wrapper = classes_of(&html, "span")
        .into_iter()
        .find(|class| html.contains(&format!(".{class}:hover")))
        .expect("the wrapper class");
    // The bubble's resting rule: the one that places it and hides it.
    let rest = html
        .split('}')
        .find(|rule| rule.contains("position:absolute;") && rule.contains("visibility:hidden;"))
        .expect("the bubble's resting rule");
    assert!(rest.contains("scale:0;"), "{rest}");

    let open = &html[html
        .find(&format!(".{wrapper}:hover > [role=\"tooltip\"]"))
        .expect("the hover rule")..];
    let open = &open[..open.find('}').unwrap()];
    assert!(open.contains("scale:1;"), "{open}");
    assert!(open.contains("scale 0s linear"), "{open}");
}
