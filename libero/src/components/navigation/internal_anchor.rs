use dioxus::prelude::*;

use crate::{
    components::{
        HtmlTag, Input, Variables,
        common::{base_props, styling_attributes, use_style_attributes},
        layout::box_style,
    },
    sx::StaticSx,
};

fn navigation_target_href(to: NavigationTarget) -> String {
    match to {
        NavigationTarget::Internal(url) | NavigationTarget::External(url) => url,
    }
}

base_props! {
    pub(crate) struct InternalAnchorProps {
        #[props(default)]
        framework_sx: Option<&'static StaticSx>,
        #[props(into)]
        to: NavigationTarget,
        #[props(default)]
        target: Option<String>,
        #[props(default, into)]
        variables: Input<Variables>,
        /// Raw `style` declarations, appended after `variables`.
        #[props(default)]
        style: Option<String>,
        #[props(default)]
        onmounted: Option<EventHandler<MountedEvent>>,
        children: Element,
    }
}

/// `to` as a working link: the router's `Link` when one is mounted and
/// `target` allows it, else a plain `<a>`. No `onclick` - only `Button`'s
/// `<button>` case needs one.
#[component]
pub(crate) fn InternalAnchor(props: InternalAnchorProps) -> Element {
    // A hook both branches need, so it stays above the return.
    let style_attributes = use_style_attributes(
        &props.class,
        props.framework_sx,
        &props.sx,
        &props.states,
        &props.variables,
        props.style,
        true,
    );

    let is_blank = props.target.as_deref() == Some("_blank");
    let router_can_handle_target = props.target.is_none() || is_blank;

    if router_can_handle_target && try_router().is_some() {
        // `Link` renders its own `class` slot, so the class stays a prop and
        // only the other two are folded in.
        let attributes = styling_attributes(
            None,
            style_attributes.data_state,
            style_attributes.style,
            props.attributes,
        );

        return rsx! {
            Link {
                to: props.to,
                class: Some(style_attributes.class),
                new_tab: is_blank,
                onmounted: move |event| {
                    if let Some(onmounted) = &props.onmounted {
                        onmounted.call(event);
                    }
                },
                attributes,
                {props.children}
            }
        };
    }

    // Attached only when the caller wants it: an unused listener still costs a
    // diff every render.
    let mut anchor = box_style(style_attributes)
        .attr("href", navigation_target_href(props.to))
        .attr("target", props.target);
    if let Some(onmounted) = props.onmounted {
        anchor = anchor.event("onmounted", move |event: MountedEvent| {
            onmounted.call(event)
        });
    }

    anchor.render(HtmlTag::A, props.attributes, props.children)
}
