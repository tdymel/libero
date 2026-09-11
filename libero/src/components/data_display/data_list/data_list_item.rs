use dioxus::prelude::*;

use crate::components::{
    Input,
    common::{base_props, styling_attributes, use_style_attributes},
};

base_props! {
    pub struct DataListItemProps {
        /// The term (`<dt>`). `sx`/`class`/`states` decorate this element
        /// only - nothing wraps a term together with its descriptions.
        label: Element,
        /// Descriptions for `label`. A `<dt>` may have any number of
        /// `<dd>`s, so each child gets its own - except against upstream main,
        /// where they all collapse into a single `<dd>`.
        #[cfg(feature = "dioxus-fork")]
        children: Vec<Element>,
        #[cfg(not(feature = "dioxus-fork"))]
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

    #[cfg(feature = "dioxus-fork")]
    return rsx! {
        dt { ..attributes, {props.label} }
        for value in props.children {
            dd { {value} }
        }
    };

    // One `Element` here, so one `<dd>` and no list to diff.
    #[cfg(not(feature = "dioxus-fork"))]
    rsx! {
        dt { ..attributes, {props.label} }
        dd { {props.children} }
    }
}
