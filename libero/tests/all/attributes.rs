//! `BoxStyle::attr` drops a `false`/`None` attribute instead of pushing it.
//!
//! That is only sound because a shrinking attribute list still *clears* what
//! went away. SSR cannot show this - it re-serialises the tree - so the check
//! has to read the mutations dioxus emits on the re-render.

use dioxus::core::{AttributeValue, ElementId, WriteMutations};
use dioxus::prelude::*;
use libero::{
    LiberoProvider,
    components::{Button, Header, Input},
    sx::ThemeAwareValue,
};

#[derive(Default)]
struct AttributeWrites(Vec<(String, String)>);

impl WriteMutations for AttributeWrites {
    fn set_attribute(&mut self, name: &str, _ns: Option<&str>, value: &AttributeValue) {
        self.0.push((name.to_string(), format!("{value:?}")));
    }
    fn push_id(&mut self, _id: ElementId) {}
    fn set_id(&mut self, _id: ElementId) {}
    fn child(&mut self, _index: usize) {}
    fn pop(&mut self) {}
    fn create_element(&mut self, _tag: &str, _ns: Option<&str>) {}
    fn create_text(&mut self, _value: &str) {}
    fn clone(&mut self) {}
    fn append_children(&mut self, _m: usize) {}
    fn replace_with(&mut self, _m: usize) {}
    fn insert_after(&mut self, _m: usize) {}
    fn insert_before(&mut self, _m: usize) {}
    fn set_text(&mut self, _value: &str) {}
    fn add_event_listener(&mut self, _name: &str) {}
    fn remove_event_listener(&mut self, _name: &str) {}
    fn remove(&mut self) {}
}

static DISABLED: GlobalSignal<bool> = Signal::global(|| true);

fn app() -> Element {
    rsx! { LiberoProvider { Button { disabled: DISABLED(), "x" } } }
}

#[test]
fn a_disabled_button_that_becomes_enabled_clears_the_attribute() {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();

    dom.in_runtime(|| *DISABLED.write() = false);
    let mut writes = AttributeWrites::default();
    dom.render_immediate(&mut writes);

    let disabled = writes
        .0
        .iter()
        .find(|(name, _)| name == "disabled")
        .map(|(_, value)| value.as_str());

    assert_eq!(
        disabled,
        Some("None"),
        "the dropped attribute was never cleared: {:?}",
        writes.0
    );
}

static COLOR: GlobalSignal<Input<ThemeAwareValue>> = Signal::global(|| Input::from("primary"));

fn header_app() -> Element {
    rsx! { LiberoProvider { Header { color: COLOR(), "x" } } }
}

/// The per-instance CSS variables ride the `style` attribute. Dropping the
/// attribute is not enough: the dioxus interpreter puts back every style
/// property the new value omits, so the header stayed blue in the browser.
/// Each var it ever set is written again as `revert-layer`, which computes to
/// what an absent var would.
#[test]
fn a_colored_header_that_becomes_unset_reverts_its_variables() {
    let mut dom = VirtualDom::new(header_app);
    dom.rebuild_in_place();

    dom.in_runtime(|| *COLOR.write() = Input::None);
    let mut writes = AttributeWrites::default();
    dom.render_immediate(&mut writes);

    let style = writes
        .0
        .iter()
        .find(|(name, _)| name == "style")
        .map(|(_, value)| value.as_str());

    assert_eq!(
        style,
        Some(
            "Text(\"--lsx-header-background:revert-layer;--lsx-header-color:revert-layer;--lsx-focus-ring-halo:revert-layer;--lsx-anchor-color:revert-layer;--lsx-surface-label:revert-layer;--lsx-focus-contrast:revert-layer;\")"
        ),
        "the header kept its old color variables: {:?}",
        writes.0
    );
}
