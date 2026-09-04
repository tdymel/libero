use crate::common::{body, render};

use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{FocusTrap, ScrollArea, Splitter, Tree, TreeNode},
};

/// These four reach their own root element, which they used to do by
/// generating an id and querying the document for it - rendering a second
/// `id` beside a caller's, which browsers resolve to the first, breaking every
/// lookup. They go through a mounted handle now, so they emit no `id` at all
/// and a caller's is simply passed through.
#[test]
fn a_root_element_component_leaves_the_id_to_its_caller() {
    fn app() -> Element {
        rsx! {
            LiberoProvider {
                ScrollArea { id: "mine", "scrollable content" }
                Splitter {
                    id: "mine",
                    initial_size: 50.0,
                    panel_a: rsx! { div { "left" } },
                    panel_b: rsx! { div { "right" } },
                }
                Tree { id: "mine", aria_label: "Files", data: vec![TreeNode::new("a", "Alpha".to_string())] }
                FocusTrap { id: "mine", "trapped" }
            }
        }
    }

    let body = body(&render(app));

    assert_eq!(body.matches("id=\"mine\"").count(), 4);
    // The one generated id is on Splitter's pane A, not a root: its divider's
    // `aria-controls` points there.
    assert_eq!(
        body.matches("id=\"lsx-").count(),
        1,
        "a generated id shadowed the caller's:\n{body}"
    );
    assert!(body.contains("aria-controls=\"lsx-"), "{body}");
}
