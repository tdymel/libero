use dioxus::prelude::*;

use crate::{
    components::{
        common::{
            HtmlTag, Input, StyleAttributes, Variables, base_props, is_javascript_url,
            styling_attributes, use_style_attributes,
        },
        layout::box_style,
    },
    sx::StaticSx,
    utils::warn,
};

fn navigation_target_href(to: NavigationTarget) -> String {
    match to {
        NavigationTarget::Internal(url) | NavigationTarget::External(url) => url,
    }
}

/// Every link in the library resolves here, so this is the one place a
/// script URL can be caught. It is passed through, not blocked: the caller
/// may mean it, and a URL from user data is theirs to check.
fn javascript_url_warning(to: &NavigationTarget) -> Option<String> {
    let (NavigationTarget::Internal(url) | NavigationTarget::External(url)) = to;
    is_javascript_url(url).then(|| {
        format!(
            "Link to `{url}`: a `javascript:` URL runs script on click. If it comes from user \
             data, check the scheme before passing it."
        )
    })
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
    let style_attributes = use_style_attributes(
        &props.class,
        props.framework_sx,
        &props.sx,
        &props.states,
        &props.variables,
        props.style,
        true,
    );

    render_anchor(
        style_attributes,
        props.to,
        props.target,
        props
            .onmounted
            .map(|onmounted| move |event| onmounted.call(event)),
        props.attributes,
        props.children,
    )
}

/// [`InternalAnchor`]'s body from styling already resolved. No hooks, so a
/// component that resolved its own styling renders the link in its own scope.
pub(crate) fn render_anchor(
    style_attributes: StyleAttributes,
    to: NavigationTarget,
    target: Option<String>,
    mut onmounted: Option<impl FnMut(MountedEvent) + 'static>,
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    if let Some(message) = javascript_url_warning(&to) {
        warn(&message);
    }

    let is_blank = target.as_deref() == Some("_blank");
    let router_can_handle_target = target.is_none() || is_blank;

    if router_can_handle_target && try_router().is_some() {
        // `Link` renders its own `class` slot, so the class stays a prop and
        // only the other two are folded in.
        let attributes = styling_attributes(
            None,
            style_attributes.data_state,
            style_attributes.style,
            attributes,
        );

        return rsx! {
            Link {
                to,
                class: Some(style_attributes.class),
                new_tab: is_blank,
                onmounted: move |event| {
                    if let Some(onmounted) = &mut onmounted {
                        onmounted(event);
                    }
                },
                attributes,
                {children}
            }
        };
    }

    // Attached only when the caller wants it: an unused listener still costs a
    // diff every render.
    let mut anchor = box_style(style_attributes)
        .attr("href", navigation_target_href(to))
        .attr("target", target);
    if let Some(mut onmounted) = onmounted {
        anchor = anchor.event("onmounted", move |event: MountedEvent| onmounted(event));
    }

    anchor.render(HtmlTag::A, attributes, children)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_javascript_target_warns() {
        let external = |url: &str| NavigationTarget::<String>::External(url.to_string());

        assert!(javascript_url_warning(&external(" JavaScript:alert(1)")).is_some());
        assert!(javascript_url_warning(&external("https://example.com")).is_none());
        assert!(javascript_url_warning(&NavigationTarget::Internal("/docs".to_string())).is_none());
    }
}
