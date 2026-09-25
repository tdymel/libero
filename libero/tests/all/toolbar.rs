use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{
        ActionIcon, Button, ButtonGroup, Options, Select, Toolbar, ToolbarGroup, ToolbarSeparator,
    },
};

#[derive(Clone, PartialEq, Options)]
enum Font {
    Serif,
    Sans,
}

/// Every opening tag that carries `needle`.
fn tags_with<'a>(html: &'a str, needle: &str) -> Vec<&'a str> {
    html.match_indices('<')
        .map(|(at, _)| &html[at..at + html[at..].find('>').unwrap()])
        .filter(|tag| tag.contains(needle))
        .collect()
}

/// One tab stop across every kind of item, the first; the rest roving at `-1`.
#[test]
fn a_toolbar_is_one_tab_stop_across_its_items() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Toolbar { "aria-label": "Formatting",
                    ToolbarGroup { "aria-label": "Style",
                        ActionIcon { aria_label: "Bold", "B" }
                        ActionIcon { aria_label: "Italic", "I" }
                    }
                    ToolbarSeparator {}
                    ButtonGroup { "aria-label": "Align",
                        Button { "Left" }
                        Button { "Right" }
                    }
                    Select { value: Font::Serif, onchange: move |_| {}, "aria-label": "Font" }
                }
            }
        }
    }

    let html = body(&render(app));
    let items = tags_with(&html, "data-toolbar-item=");
    assert_eq!(items.len(), 5, "{html}");
    assert!(items[0].contains(r#"tabindex="0""#), "{}", items[0]);
    for item in &items[1..] {
        assert!(item.contains(r#"tabindex="-1""#), "{item}");
    }
    assert_eq!(html.matches(r#"tabindex="0""#).count(), 1, "{html}");

    let bar = tags_with(&html, r#"role="toolbar""#);
    assert!(bar[0].contains(r#"aria-label="Formatting""#), "{html}");
    assert!(!bar[0].contains("aria-orientation"), "{html}");
    assert_eq!(tags_with(&html, r#"role="group""#).len(), 2, "{html}");
    let separator = tags_with(&html, r#"role="separator""#);
    assert!(
        separator[0].contains(r#"aria-orientation="vertical""#),
        "{}",
        separator[0]
    );
}

#[test]
fn a_vertical_toolbar_says_so_and_lays_its_separators_flat() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Toolbar { "aria-label": "Tools", orientation: "vertical",
                    ActionIcon { aria_label: "Pen", "P" }
                    ToolbarSeparator {}
                    ActionIcon { aria_label: "Eraser", "E" }
                }
            }
        }
    }

    let html = body(&render(app));
    let bar = tags_with(&html, r#"role="toolbar""#);
    assert!(bar[0].contains(r#"aria-orientation="vertical""#), "{html}");
    // `horizontal` is the separator role's default.
    let separator = tags_with(&html, r#"role="separator""#);
    assert!(!separator[0].contains("aria-orientation"), "{html}");
}

/// A disabled item stays focusable, so arrows and screen readers still find it.
#[test]
fn a_disabled_item_in_a_toolbar_stays_focusable() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Toolbar { "aria-label": "Edit",
                    Button { disabled: true, "Cut" }
                    ActionIcon { aria_label: "Paste", disabled: true, "V" }
                    Select { value: Font::Sans, onchange: move |_| {}, disabled: true, "aria-label": "Font" }
                    Button { disabled: true, focusable_when_disabled: false, "Gone" }
                }
            }
        }
    }

    let html = body(&render(app));
    let items = tags_with(&html, "data-toolbar-item=");
    assert_eq!(items.len(), 4, "{html}");
    for item in &items[..3] {
        assert!(item.contains(r#"aria-disabled="true""#), "{item}");
        assert!(!item.contains("disabled=true"), "{item}");
    }
    // The caller's explicit `false` still wins.
    assert!(items[3].contains("disabled=true"), "{}", items[3]);
}

/// Outside a toolbar nothing changes: no marker, no roving `tabindex`.
#[test]
fn a_button_outside_a_toolbar_is_untouched() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Button { "Save" }
                ActionIcon { aria_label: "Close", disabled: true, "x" }
            }
        }
    }

    let html = body(&render(app));
    assert!(!html.contains("data-toolbar-item"), "{html}");
    assert!(!html.contains("tabindex"), "{html}");
}

/// The first item gone, the next one becomes the tab stop.
#[test]
fn an_unmounted_tab_stop_hands_the_stop_on() {
    fn app() -> Element {
        let mut show = use_signal(|| true);
        use_effect(move || show.set(false));
        rsx! {
            LiberoProvider {
                Toolbar { "aria-label": "Edit",
                    if show() {
                        Button { "First" }
                    }
                    Button { "Second" }
                    Button { "Third" }
                }
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    dom.render_immediate(&mut dioxus::core::NoOpMutations);
    let html = body(&dioxus_ssr::render(&dom));

    assert!(!html.contains("First"), "{html}");
    let items = tags_with(&html, "data-toolbar-item=");
    assert_eq!(items.len(), 2, "{html}");
    assert!(items[0].contains(r#"tabindex="0""#), "{html}");
    assert!(items[1].contains(r#"tabindex="-1""#), "{html}");
}
