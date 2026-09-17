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
