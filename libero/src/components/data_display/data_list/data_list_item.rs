use dioxus::prelude::*;

use crate::{
    components::{Box, Input, States, common::base_props},
    sx::Sx,
};

base_props! {
    pub struct DataListItemProps {
        /// The term (`<dt>`) - `sx`/`class`/`attributes`/`states` decorate
        /// this element specifically, not the whole item (there's no single
        /// element wrapping a term plus its descriptions).
        label: Element,
        /// One or more descriptions (`<dd>`) for `label` - a `<dt>` can have
        /// any number of `<dd>`s, so this is a `Vec<Element>` rather than a
        /// single `Element`: writing more than one child here (literally, or
        /// from a `for` loop) gives each one its own `<dd>`, the same way
        /// writing them as ordinary rsx! children always has, with no extra
        /// ceremony over a single description.
        children: Vec<Element>,
    }
}

/// One entry of a [`DataList`](super::DataList) - a term and its
/// description(s).
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
