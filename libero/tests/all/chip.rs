use crate::common::{attributes_of, body, classes_of, render};

use dioxus::prelude::*;
use libero::{LiberoProvider, components::Chip};

/// The hidden checkbox is `position: absolute`, so the chip has to be its
/// containing block - the same defect `SegmentedControl` had, where a focused
/// input laid out against the viewport scrolls the whole document.
#[test]
fn a_selectable_chip_contains_its_hidden_checkbox() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { checked: true, onchange: move |_| {}, "tag" }
            }
        }
    }

    let html = render(app);
    let input = classes_of(&html, "input");
    assert!(
        input
            .iter()
            .any(|class| html.contains(&format!(".{class}{{position:absolute"))),
        "{input:?}"
    );

    let positioned = classes_of(&html, "span").iter().any(|class| {
        let rule = format!(".{class}{{");
        html.find(&rule).is_some_and(|at| {
            let base = &html[at..];
            base[..base.find('}').unwrap()].contains("position:relative")
        })
    });
    assert!(positioned, "no positioned class on the chip root");
}

#[test]
fn a_selectable_chip_renders_a_checkbox_its_label_points_at() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { checked: true, onchange: move |_| {}, "tag" }
            }
        }
    }

    let html = render(app);
    let input = attributes_of(&html, "input");

    assert_eq!(input["type"], "checkbox");
    assert!(html.contains("checked=true"));
    // The label points at the input, so clicking the text toggles it.
    assert_eq!(attributes_of(&html, "label")["for"], input["id"]);

    let span = attributes_of(&html, "span");
    assert_eq!(
        span["data-state"],
        "filled size-md radius-xl checked selectable"
    );
    // M3's selected filter chip is a tonal container, and one step past
    // `Tonal`'s own resting tint - so selecting an already-tonal chip still
    // reads as a change.
    assert!(span["style"].contains("--lsx-chip-container:var(--lsx-primary-fill-2);"));
    assert!(span["style"].contains("--lsx-chip-on-container:var(--lsx-primary-contrast-2);"));
    assert!(body(&html).contains(">tag<"));
}

#[test]
fn a_clickable_chip_is_a_button_and_a_linked_one_an_anchor() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { onclick: move |_| {}, "act" }
                Chip { to: "https://example.com", target: "_blank", "go" }
            }
        }
    }

    let html = render(app);

    assert_eq!(attributes_of(&html, "button")["type"], "button");
    assert_eq!(attributes_of(&html, "a")["href"], "https://example.com");
    assert_eq!(attributes_of(&html, "a")["target"], "_blank");
    // The pointer state both roots need, and no `<span>` root gets.
    assert!(attributes_of(&html, "button")["data-state"].contains("clickable"));
}

/// A row of filter chips shares one `name`, so each has to post its own
/// `value` (todo 20). Without one the attribute is absent and the browser
/// posts its default `on`, exactly as `Checkbox` does.
#[test]
fn a_chip_posts_its_value_under_the_shared_name_or_falls_back_to_on() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { name: "tags", value: "rust", "Rust" }
            }
        }
    }

    let html = render(app);
    let input = attributes_of(&html, "input");
    assert_eq!(input["name"], "tags");
    assert_eq!(input["value"], "rust");

    fn valueless() -> Element {
        rsx! {
            LiberoProvider {
                Chip { name: "agreed", "Agreed" }
            }
        }
    }

    let html = render(valueless);
    let input = attributes_of(&html, "input");
    assert_eq!(input["name"], "agreed");
    assert!(!input.contains_key("value"), "{input:?}");
}

/// Todo 636: `icon` comes right before the children, and todo 674: the
/// children are never wrapped, so a caller's icon among them keeps the gap.
#[test]
fn the_icon_sits_right_before_the_unwrapped_children() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { icon: rsx! { svg { id: "tag-icon" } }, "tag" }
                Chip {
                    icon: rsx! { svg { id: "filter-icon" } },
                    checked: false,
                    onchange: move |_| {},
                    "filter"
                }
                Chip { icon: rsx! { svg { id: "action-icon" } }, onclick: move |_| {}, "action" }
            }
        }
    }

    let html = body(&render(app));
    for icon in ["tag", "filter", "action"] {
        let pair =
            format!(r#"<span data-slot="chip-icon"><svg id="{icon}-icon"></svg></span>{icon}<"#);
        assert!(html.contains(&pair), "{pair}: {html}");
    }
    assert!(!html.contains(r#"data-slot="text""#), "{html}");
}

/// `trailing` follows the children; on a checkbox chip it sits beside the
/// `<label>`, which would otherwise take a button's click for the checkbox.
#[test]
fn trailing_follows_the_label_and_stays_out_of_a_checkbox_label() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { trailing: rsx! { button { id: "tag-x" } }, "tag" }
                Chip {
                    trailing: rsx! { button { id: "filter-x" } },
                    checked: false,
                    onchange: move |_| {},
                    "filter"
                }
            }
        }
    }

    let html = body(&render(app));
    let tag = html.find(">tag<").unwrap();
    assert!(html.find("tag-x").unwrap() > tag, "{html}");
    let label_end = html.find("</label>").unwrap();
    let filter_x = html.find("filter-x").unwrap();
    assert!(html[..=label_end].contains(">filter<"), "{html}");
    assert!(filter_x > label_end, "the x landed in the label: {html}");
    assert_eq!(
        html.matches(r#"data-slot="chip-trailing""#).count(),
        2,
        "{html}"
    );
}

/// Todo 672: a `_blank` link chip gets Anchor's new-tab cue after its
/// children, and the caller can opt out.
#[test]
fn a_blank_link_chip_adds_the_new_tab_hint_unless_opted_out() {
    fn blank() -> Element {
        rsx! {
            LiberoProvider {
                Chip { to: "https://example.com", target: "_blank", "Example" }
            }
        }
    }
    fn opted_out() -> Element {
        rsx! {
            LiberoProvider {
                Chip { to: "https://example.com", target: "_blank", new_tab_hint: false, "Example" }
            }
        }
    }
    fn same_tab() -> Element {
        rsx! {
            LiberoProvider {
                Chip { to: "https://example.com", "Example" }
            }
        }
    }

    let html = body(&render(blank));
    assert!(html.contains("(opens in a new tab)"), "{html}");
    let text_end = html.find("Example").unwrap();
    assert!(
        html.find("data-slot=\"new-tab\"").unwrap() > text_end,
        "{html}"
    );
    assert!(html.contains("<svg"), "{html}");
    for app in [opted_out as fn() -> Element, same_tab] {
        let html = body(&render(app));
        assert!(!html.contains("new tab"), "{html}");
        assert!(!html.contains("<svg"), "{html}");
    }
}

/// Todo 629: said with `aria-readonly`, as on `Checkbox`, and never with
/// `disabled`, which would drop it from the tab order and the post.
#[test]
fn a_readonly_chip_says_so_and_stays_enabled() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Chip { name: "tags", value: "rust", readonly: true, checked: true, onchange: move |_| {}, "rust" }
            }
        }
    }

    let html = render(app);
    let input = attributes_of(&html, "input");
    assert_eq!(
        input.get("aria-readonly").map(String::as_str),
        Some("true"),
        "{html}"
    );
    assert!(!input.contains_key("disabled"), "{html}");
    assert_eq!(
        input.get("name").map(String::as_str),
        Some("tags"),
        "{html}"
    );
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::{attributes_of, body};
    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Chip, Form, MultiSelect},
    };

    #[test]
    fn pressing_a_chip_x_keeps_the_focus_on_the_trigger() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    MultiSelect::<Emphasis> {
                        label: "Emphasis",
                        value: vec![Emphasis::Bold, Emphasis::Italic],
                        onchange: move |_: Vec<Emphasis>| {},
                    }
                }
            }
        }

        // One per chip, the value slot's and the chevron's.
        assert_every_press_keeps_the_focus(app, 4);
    }

    /// Todo 222: a `name` makes a `Chip` a checkbox that posts, and one with no
    /// handler keeps its own state - the way an unbound `Checkbox` does.
    #[test]
    fn a_named_chip_posts_and_toggles_on_its_own() {
        fn app() -> Element {
            rsx! {
                LiberoProvider {
                    Chip { name: "open", "Open" }
                }
            }
        }

        let Page { mut dom, rec: find } = Page::build(app);

        let html = dioxus_ssr::render(&dom);
        let input = attributes_of(&body(&html), "input");
        assert_eq!(input.get("type").map(String::as_str), Some("checkbox"));
        assert_eq!(input.get("name").map(String::as_str), Some("open"));
        assert_eq!(checked_states(&html), [false]);

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), last_click(&find));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [true]);
    }

    #[derive(Clone, PartialEq, Default, libero::components::Fields)]
    pub struct Filters {
        pub open: bool,
    }

    /// Todo 222: a path `name` binds a `Chip` to the `Form` around it, as on
    /// `Checkbox` - the click writes the form's value, and the chip renders it.
    #[test]
    fn a_chip_with_a_path_name_writes_its_forms_value() {
        fn app() -> Element {
            let filters = use_store(Filters::default);
            rsx! {
                LiberoProvider {
                    Form { value: filters,
                        Chip { name: Filters::FIELDS.open(), "Open" }
                    }
                }
            }
        }

        let Page { mut dom, rec: find } = Page::build(app);
        assert_eq!(checked_states(&dioxus_ssr::render(&dom)), [false]);

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), last_click(&find));
        dom.render_immediate(&mut dioxus::core::NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert_eq!(checked_states(&html), [true]);
        let input = attributes_of(&body(&html), "input");
        assert_eq!(input.get("name").map(String::as_str), Some("open"));
    }
}

static NO_CHECK: libero::theme::Theme = libero::theme::Theme {
    chip: libero::theme::ChipDefaults {
        selected_check: false,
        ..libero::theme::Theme::DEFAULT.chip
    },
    ..libero::theme::Theme::DEFAULT
};

/// Todo 2707: a selected filter chip draws a hidden check, so selection is not colour
/// alone; the checkbox keeps the state, and the theme can drop the check.
#[test]
fn a_selected_chip_draws_a_hidden_check() {
    fn row() -> Element {
        rsx! {
            Chip { checked: true, onchange: move |_| {}, "on" }
            Chip { checked: false, onchange: move |_| {}, "off" }
            Chip { onclick: move |_| {}, "action" }
        }
    }
    fn app() -> Element {
        rsx! { LiberoProvider { {row()} } }
    }
    fn plain() -> Element {
        rsx! { LiberoProvider { themes: &NO_CHECK, {row()} } }
    }

    let html = body(&render(app));
    assert_eq!(
        html.matches("data-slot=\"chip-check\"").count(),
        1,
        "{html}"
    );
    let before = &html[..html.find("data-slot=\"chip-check\"").unwrap()];
    let check = &html[before.rfind("<span").unwrap()..];
    assert_eq!(attributes_of(check, "span")["aria-hidden"], "true");
    // Inside the checked chip's label.
    assert!(
        before
            .rfind("<label")
            .is_some_and(|at| !before[at..].contains("</label>"))
    );
    assert!(!body(&render(plain)).contains("chip-check"));
}
