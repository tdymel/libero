use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::Sx,
};

base_props! {
    pub struct DataListItemProps {
        /// The term (`<dt>`). `sx`/`class`/`states` decorate this element
        /// only - nothing wraps a term together with its descriptions.
        label: Element,
        /// Descriptions for `label`. A `Vec` because a `<dt>` may have any
        /// number of `<dd>`s - each child gets its own.
        children: Vec<Element>,
    }
}

/// A term and its descriptions, in a [`DataList`](super::DataList).
///
/// ```ignore
/// DataListItem {
///     label: "Phone",
///     "555-1234"
///     "555-5678"
/// }
/// ```
#[component]
pub fn DataListItem(props: DataListItemProps) -> Element {
    rsx! {
        Box {
            component: "dt",
            class: props.class,
            sx: props.sx,
            states: props.states,
            attributes: props.attributes,
            {props.label}
        }
        for (index, value) in props.children.into_iter().enumerate() {
            Box { component: "dd", key: "{index}", {value} }
        }
    }
}
