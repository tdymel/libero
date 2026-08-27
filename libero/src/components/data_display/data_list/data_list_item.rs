use dioxus::prelude::*;

use crate::components::{
    HtmlTag, Input,
    common::{IntoChildren, base_props},
    layout::use_box,
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
        for (index, value) in props.children.into_children().into_iter().enumerate() {
            dd { key: "{index}", {value} }
        }
    }
}
