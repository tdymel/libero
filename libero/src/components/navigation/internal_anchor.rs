use dioxus::prelude::*;

use crate::{
    CssLayer,
    components::{
        Box, Input, States, Variables,
        common::{attr, base_props},
    },
    hooks::use_css,
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
        #[props(default)]
        onmounted: EventHandler<MountedEvent>,
        children: Element,
    }
}

/// Renders `to` as a working link - the router's own `Link` when one is
/// mounted and `target` allows it (unset or `"_blank"`), else a plain `<a>`.
/// No `onclick`: only `Button`'s non-link `<button>` case needs a click
/// handler, and ripple only makes sense for a real button.
#[component]
pub(crate) fn InternalAnchor(props: InternalAnchorProps) -> Element {
    let framework_class = props
        .framework_sx
        .and_then(|sx| use_css(sx, CssLayer::Framework));
    let static_class = props
        .sx
        .as_ref()
        .and_then(|sx| use_css(sx, CssLayer::UserStatic));
    let class = props
        .class
        .unwrap_or_default()
        .with(framework_class)
        .with(static_class);

    let is_blank = props.target.as_deref() == Some("_blank");
    let router_can_handle_target = props.target.is_none() || is_blank;
    let data_state = props.states.as_ref().and_then(States::data_state);
    let style = props
        .variables
        .as_ref()
        .map(Variables::to_string)
        .filter(|style| !style.is_empty());

    if router_can_handle_target && try_router().is_some() {
        let attributes = props
            .attributes
            .into_iter()
            .chain(data_state.map(|value| attr("data-state", value)))
            .chain(style.map(|value| attr("style", value)))
            .collect::<Vec<_>>();

        return rsx! {
            Link {
                to: props.to,
                class: Some(class.to_string()),
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
            class,
            states: props.states,
            variables: props.variables,
            href: Some(navigation_target_href(props.to)),
            target: props.target,
            onmounted: props.onmounted,
            attributes: props.attributes,
            {props.children}
        }
    }
}
