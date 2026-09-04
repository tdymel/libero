use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{OptionLabel, Options, Tabs},
};

#[derive(Clone, PartialEq, Options)]
enum Section {
    Account,
    #[option(label = "Admin area")]
    Admin,
    Billing,
}

#[test]
fn tabs_wire_the_selected_tab_to_its_panel_and_render_only_that_one() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                Tabs {
                    value: Section::Admin,
                    onchange: move |_| {},
                    disabled: vec![Section::Billing],
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
                    tabs: vec![Section::Account, Section::Billing],
                    label: |section: Section| OptionLabel::rich(
                        format!("t:{}", section.label()),
                        rsx! { span { "rich" } },
                    ),
                    panel: |_: Section| rsx! { "body" },
                }
            }
        }
    }

    let body = body(&render(app));

    // `tabs` narrows the strip; `label` names them; `render_label` fills them.
    assert_eq!(body.matches("role=\"tab\"").count(), 2);
    assert!(body.contains("aria-label=\"t:Account\""));
    assert!(body.contains("aria-label=\"t:Billing\""));
    assert_eq!(body.matches("rich").count(), 2);
    assert!(!body.contains(">Account<"));
}
