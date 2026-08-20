use dioxus::prelude::*;

use crate::{
    components::{
        Box, Input, States, Variables,
        common::{base_props, styling_attributes, use_style_attributes},
    },
    sx::{StaticSx, Sx},
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
        onmounted: EventHandler<MountedEvent>,
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
        // Cloned: the plain-`<a>` branch below hands the same string to `Box`,
        // which resolves the styling a second time.
        props.style.clone(),
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
                onmounted: move |event| props.onmounted.call(event),
                attributes,
                {props.children}
            }
        };
    }

    rsx! {
        Box {
            component: "a",
            class: props.class,
            sx: props.sx,
            states: props.states,
            variables: props.variables,
            style: props.style,
            framework_sx: props.framework_sx,
            href: Some(navigation_target_href(props.to)),
            target: props.target,
            onmounted: props.onmounted,
            attributes: props.attributes,
            {props.children}
        }
    }
}
