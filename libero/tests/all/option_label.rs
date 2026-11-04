//! `OptionLabel::with_icon` draws the icon hidden before the name, wherever a label goes (todo 5636).

use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{OptionLabel, Options, SegmentedControl, Tabs},
};

#[derive(Clone, Copy, PartialEq, Options)]
enum Page {
    Inbox,
    Sent,
}

fn labelled(page: Page) -> OptionLabel {
    OptionLabel::with_icon(page.label(), rsx! { svg { "data-icon": "{page.label()}" } })
}

/// The icon's hidden span right before the name, so the name stays the accessible name.
fn assert_icon_before_name(html: &str, name: &str) {
    let icon = format!(r#"<span aria-hidden="true"><svg data-icon="{name}"></svg></span>{name}"#);
    assert!(html.contains(&icon), "{icon} missing in {html}");
}

#[test]
fn a_tab_draws_its_icon_hidden_before_the_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    aria_label: "Mail",
                    value: Page::Inbox,
                    onchange: move |_| {},
                    option_label: labelled,
                    panel: |_: Page| rsx! { "panel" },
                }
            }
        }
    }
    let html = body(&render(app));
    assert_icon_before_name(&html, "Inbox");
    assert_icon_before_name(&html, "Sent");
}

#[test]
fn a_segment_draws_its_icon_hidden_before_the_name() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                SegmentedControl {
                    value: Page::Inbox,
                    onchange: move |_| {},
                    option_label: labelled,
                }
            }
        }
    }
    let html = body(&render(app));
    assert_icon_before_name(&html, "Inbox");
    assert_icon_before_name(&html, "Sent");
}

#[test]
fn the_name_stays_plain_text() {
    let label = labelled(Page::Sent);
    assert_eq!(label.name(), "Sent");
}
