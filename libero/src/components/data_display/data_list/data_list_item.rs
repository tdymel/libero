use dioxus::prelude::*;

use crate::{
    components::{HtmlTag, Input, States, common::base_props, layout::use_box},
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
    // The `<dd>`s carry no styling of their own, so they are plain elements -
    // a `Box` per description would be a component scope for nothing.
    let term = use_box()
        .class(&props.class)
        .sx(&props.sx)
        .states(&props.states)
        .prepare()
        .render(HtmlTag::Dt, props.attributes, props.label);

    rsx! {
        {term}
        for (index, value) in props.children.into_iter().enumerate() {
            dd { key: "{index}", {value} }
        }
    }
}
