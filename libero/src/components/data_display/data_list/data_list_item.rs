use dioxus::prelude::*;

use crate::components::common::{Input, base_props, styling_attributes, use_style_attributes};

base_props! {
    pub struct DataListItemProps {
        /// The term (`<dt>`). `sx`/`class`/`states` decorate this element
        /// only - nothing wraps a term together with its descriptions.
        label: Element,
        /// Descriptions for `label`, in a single `<dd>`: dioxus merges the
        /// children into one node.
        children: Element,
    }
}

/// A term and its descriptions, in a [`DataList`](super::DataList).
///
/// ```no_run
/// # use dioxus::prelude::*;
/// # use libero::components::{DataList, DataListItem};
/// # fn app() -> Element { rsx! { DataList {
/// DataListItem {
///     label: rsx! { "Phone" },
///     "555-1234"
///     "555-5678"
/// }
/// # } } }
/// ```
#[component]
pub fn DataListItem(props: DataListItemProps) -> Element {
    // The `<dd>`s carry no styling of their own, so they are plain elements -
    // a `Box` per description would be a component scope for nothing.
    // Always a `<dt>`, so written inline: a rendered `Element` spliced in is a
    // nested template.
    let style = use_style_attributes(
        &props.class,
        None,
        &props.sx,
        &props.states,
        &Input::None,
        None,
        true,
    );
    let attributes = styling_attributes(
        Some(style.class),
        style.data_state,
        style.style,
        props.attributes,
    );

    // One `Element` here, so one `<dd>` and no list to diff.
    rsx! {
        dt { ..attributes, {props.label} }
        dd { {props.children} }
    }
}
