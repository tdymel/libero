use dioxus::prelude::*;

use crate::components::common::{Input, base_props, styling_attributes, use_style_attributes};

base_props! {
    pub struct DataListItemProps {
        /// The term (`<dt>`), which `sx`/`class`/`states` decorate.
        label: Element,
        /// Descriptions for `label`, in a single `<dd>`. For an `id` or a style on a
        /// description, wrap it in a `div` here.
        children: Element,
    }
}

/// A term and its descriptions, in a [`DataList`](super::DataList).
///
/// ```rust
/// # use dioxus::prelude::*;
/// # use libero::components::{DataList, DataListItem};
/// # fn app() -> Element {
/// rsx! {
///     DataList {
///         DataListItem { label: rsx! { "Phone" }, "555-1234" }
///     }
/// }
/// # }
/// ```
///
/// Docs: <https://libero-ui.dev/data-display/data-list>
#[component]
pub fn DataListItem(props: DataListItemProps) -> Element {
    // Plain `dt`/`dd`, not a `Box`: a spliced `Element` would be a nested template.
    let visible = crate::components::common::with_visibility(
        &props.states,
        &props.hidden_from,
        &props.visible_from,
    );
    let style = use_style_attributes(
        &props.class,
        None,
        crate::components::common::sx_source(&props.sx),
        visible.as_ref().unwrap_or(&props.states),
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

    rsx! {
        dt { ..attributes, {props.label} }
        dd { {props.children} }
    }
}
