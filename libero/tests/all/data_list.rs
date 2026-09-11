use crate::common::{attributes_of, body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{DataList, DataListItem},
};

#[test]
fn data_list_pairs_a_label_with_its_value() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList {
                    DataListItem { label: rsx! { "Status" }, "Active" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert!(body.contains("Status"));
    assert!(body.contains("Active"));
}

/// The term is written inline, so its styling and the caller's attributes
/// still have to land on the `<dt>`.
#[test]
fn a_terms_class_and_attributes_land_on_the_dt() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList {
                    DataListItem { label: rsx! { "Status" }, class: "term", id: "t", "Active" }
                }
            }
        }
    }

    let attributes = attributes_of(&render(app), "dt");

    assert!(attributes["class"].split(' ').any(|class| class == "term"));
    assert_eq!(attributes["id"], "t");
}

/// Only the fork splits children into a `Vec<Element>`; upstream main merges them.
#[cfg(feature = "dioxus-fork")]
#[test]
fn each_description_of_a_term_gets_its_own_dd() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                DataList {
                    DataListItem { label: rsx! { "Phone" }, "555-1234" "555-5678" }
                }
            }
        }
    }

    let html = render(app);
    let body = body(&html);

    assert_eq!(body.matches("<dt").count(), 1, "{body}");
    assert_eq!(body.matches("<dd").count(), 2, "{body}");
    assert!(body.contains("<dd>555-1234</dd>"), "{body}");
    assert!(body.contains("<dd>555-5678</dd>"), "{body}");
}
