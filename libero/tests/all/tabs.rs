use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{OptionItem, OptionLabel, OptionList, Options, Tabs},
};

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    #[option(label = "Admin area")]
    Admin,
    Billing,
}

/// Every section, with the named ones flagged.
fn sections_with_disabled(off: &[Section]) -> OptionList<Section> {
    OptionList::from_options().disabling(|section| off.contains(section))
}

#[test]
fn tabs_wire_the_selected_tab_to_its_panel_and_render_only_that_one() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Admin,
                    onchange: move |_| {},
                    options: sections_with_disabled(&[Section::Billing]),
                    panel: |section: Section| match section {
                        Section::Account => rsx! { "account body" },
                        Section::Admin => rsx! { "admin body" },
                        Section::Billing => rsx! { "billing body" },
                    },
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // One button per variant, in declaration order, labelled by the derive.
    assert_eq!(body.matches("role=\"tab\"").count(), 3);
    assert!(body.contains("Account"));
    assert!(body.contains("Admin area"));

    // Only the selected panel is rendered at all.
    assert!(body.contains("admin body"));
    assert!(!body.contains("account body"));
    assert!(!body.contains("billing body"));

    // Exactly one selected tab, and it owns the roving tabindex.
    assert_eq!(body.matches("aria-selected=\"true\"").count(), 1);
    assert_eq!(body.matches("tabindex=\"0\"").count(), 2); // the tab and its panel
    assert_eq!(body.matches("aria-disabled=\"true\"").count(), 1);

    // The panel points back at the tab that controls it.
    let panel = attributes_of(&body, "div role=\"tabpanel\"");
    let tab_id = panel["aria-labelledby"].clone();
    assert!(body.contains(&format!("id=\"{tab_id}\"")));
    assert!(body.contains(&format!("aria-controls=\"{}\"", panel["id"])));

    // Only the selected tab names a panel: the others' panels are not in the
    // document, and a reference to a missing id is invalid.
    assert_eq!(body.matches("aria-controls=").count(), 1);
    assert!(body.contains(&format!(
        "id=\"{tab_id}\" aria-selected=\"true\" aria-controls=\"{}\"",
        panel["id"]
    )));
}

#[test]
fn a_rich_tab_label_draws_its_content_and_still_names_the_tab() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Account,
                    onchange: move |_| {},
                    options: vec![Section::Account, Section::Billing],
                    option_label: |section: Section| OptionLabel::rich(
                        format!("t:{}", section.label()),
                        rsx! { span { "rich" } },
                    ),
                    panel: |_: Section| rsx! { "body" },
                }
            }
        }
    }

    let body = body(&render(app));

    // `options` narrows the strip; `option_label` names them; `render_label` fills them.
    assert_eq!(body.matches("role=\"tab\"").count(), 2);
    assert!(body.contains("aria-label=\"t:Account\""));
    assert!(body.contains("aria-label=\"t:Billing\""));
    assert_eq!(body.matches("rich").count(), 2);
    assert!(!body.contains(">Account<"));
}

#[test]
fn a_value_outside_the_tabs_leaves_the_first_enabled_tab_as_the_tab_stop() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Admin,
                    onchange: move |_| {},
                    options: OptionList::new([
                        OptionItem::new(Section::Account).disabled(true),
                        Section::Billing.into(),
                    ]),
                    panel: |_: Section| rsx! {},
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    // Nothing is selected, but the strip keeps one tab stop, on the first
    // tab that can be picked.
    assert_eq!(body.matches("aria-selected=\"true\"").count(), 0);
    assert_eq!(body.matches("tabindex=\"0\"").count(), 1);
    let stop = body
        .split("<button")
        .find(|tab| tab.contains("tabindex=\"0\""))
        .unwrap();
    assert!(stop.contains("aria-label=\"Billing\""));
}

/// The arrow keys focus a tab by `{id}-tab-{n}`, and a caller's id need not be
/// a CSS identifier. The lookup selects by attribute (todo 248), which only
/// works while the tabs carry exactly that id.
#[test]
fn a_caller_id_that_is_no_css_identifier_still_names_the_tabs() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    id: "1-faq",
                    value: Section::Account,
                    onchange: move |_| {},
                    panel: |_: Section| rsx! { "body" },
                }
            }
        }
    }

    let body = body(&render(app));

    for n in 0..3 {
        assert!(body.contains(&format!("id=\"1-faq-tab-{n}\"")), "{body}");
    }
}

/// Todo 501: the caller's name lands on the tablist, which APG names, and not
/// on the role-less root, where `aria-label` is prohibited.
#[test]
fn the_callers_name_goes_to_the_tablist() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    id: "t",
                    aria_label: "Settings",
                    value: Section::Account,
                    onchange: move |_| {},
                    panel: |_: Section| rsx! { "body" },
                }
                Tabs {
                    id: "u",
                    aria_labelledby: "heading",
                    value: Section::Account,
                    onchange: move |_| {},
                    panel: |_: Section| rsx! { "body" },
                }
            }
        }
    }

    let body = body(&render(app));

    // Once each, so the root lost it.
    assert_eq!(body.matches("aria-label=\"Settings\"").count(), 1, "{body}");
    assert_eq!(
        body.matches("aria-labelledby=\"heading\"").count(),
        1,
        "{body}"
    );
    let list = attributes_of(&body, "div role=\"tablist\"");
    assert_eq!(list["aria-label"], "Settings");
    let second = &body[body.find("id=\"u\"").unwrap()..];
    assert_eq!(
        attributes_of(second, "div role=\"tablist\"")["aria-labelledby"],
        "heading"
    );
}

/// Todo 371: an `options` prop that was never set is not an empty list - it
/// still means every `Options::options()`, with nothing disabled.
#[test]
fn tabs_left_without_options_still_list_the_enums_own() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Account,
                    onchange: move |_| {},
                    panel: |_: Section| rsx! { "body" },
                }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(body.matches("role=\"tab\"").count(), 3);
    assert_eq!(body.matches("aria-disabled=\"true\"").count(), 0);
}

/// Events dispatched the way a renderer does, through [`crate::dispatch`].
mod dispatched {
    use crate::common::body;
    use crate::dispatch::*;

    use dioxus::prelude::*;
    use libero::{
        LiberoProvider,
        components::{Options, Tabs},
    };

    #[derive(Clone, PartialEq, Options)]
    enum Pane {
        First,
        Second,
    }

    #[test]
    fn clicking_a_tab_selects_it_and_swaps_the_panel() {
        fn app() -> Element {
            let mut pane = use_signal(|| Pane::First);

            rsx! {
                LiberoProvider {
                    Tabs {
                        value: pane(),
                        onchange: move |next| pane.set(next),
                        panel: |pane: Pane| match pane {
                            Pane::First => rsx! { "first body" },
                            Pane::Second => rsx! { "second body" },
                        },
                    }
                }
            }
        }

        let Page { mut dom, rec: find } = Page::build(app);
        let second = find.element("click", "aria-label", "Second");

        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("first body"));
        assert!(!html.contains("second body"));

        dom.runtime()
            .handle_event("click", Event::new(click_event(), true), second);
        dom.render_immediate(&mut dioxus::core::NoOpMutations);

        // Only the newly selected panel is in the tree - the other is not hidden,
        // it is gone.
        let html = dioxus_ssr::render(&dom);
        assert!(html.contains("second body"));
        assert!(!html.contains("first body"));
        assert_eq!(body(&html).matches("aria-selected=\"true\"").count(), 1);
    }
}
