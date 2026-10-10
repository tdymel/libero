use dioxus::{html::input_data::MouseButton, prelude::*, router::RouterContext};

use crate::{
    components::{
        common::{
            HtmlTag, Input, StyleAttributes, Variables, base_props, is_javascript_url,
            use_style_attributes,
        },
        layout::{BoxStyle, box_style},
    },
    sx::StaticSx,
    utils::warn,
};

fn navigation_target_href(to: NavigationTarget) -> String {
    match to {
        NavigationTarget::Internal(url) | NavigationTarget::External(url) => url,
    }
}

/// Every library link resolves here. Warned, not blocked: the caller may mean
/// it, and a URL from user data is theirs to check.
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

/// `to` as the router's `Link` when one is mounted and `target` allows it,
/// else a plain `<a>`.
#[component]
pub(crate) fn InternalAnchor(props: InternalAnchorProps) -> Element {
    let style_attributes = use_style_attributes(
        &props.class,
        props.framework_sx,
        crate::components::common::sx_source(&props.sx),
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
    onmounted: Option<impl FnMut(MountedEvent) + 'static>,
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    if let Some(message) = javascript_url_warning(&to) {
        warn(&message);
    }

    let is_blank = target
        .as_deref()
        .is_some_and(|target| target.eq_ignore_ascii_case("_blank"));
    let router = (target.is_none() || is_blank).then(try_router).flatten();

    // Attached only when the caller wants it: an unused listener still costs a
    // diff every render, and on a WebView a blocking round trip (todo 2986).
    // Without a router every string is `External`, so only an absolute URL counts as one.
    let rel = matches!(&to, NavigationTarget::External(url) if url.contains("://") || url.starts_with("//"))
        .then_some("noopener noreferrer");
    let mut anchor = match router {
        Some(router) => routed(box_style(style_attributes), router, to, is_blank),
        None => box_style(style_attributes)
            .attr_default("rel", rel)
            .attr("href", navigation_target_href(to))
            .attr("target", target),
    };
    if let Some(mut onmounted) = onmounted {
        anchor = anchor.event("onmounted", move |event: MountedEvent| onmounted(event));
    }

    anchor.render(HtmlTag::A, attributes, children)
}

/// The router's `Link` by hand, as it attaches an `onmounted` to every anchor
/// (todo 2986). Its `href`, `rel`, `aria-current` and click are kept.
fn routed(
    anchor: BoxStyle,
    router: RouterContext,
    to: NavigationTarget,
    new_tab: bool,
) -> BoxStyle {
    let is_external = matches!(to, NavigationTarget::External(_));
    let href = match &to {
        NavigationTarget::Internal(url) => router.prefix().unwrap_or_default() + url,
        NavigationTarget::External(url) => url.clone(),
    };
    let current =
        matches!(&to, NavigationTarget::Internal(url) if *url == router.full_route_string());
    anchor
        .attr("href", href)
        .attr("target", new_tab.then_some("_blank"))
        .attr_default("rel", is_external.then_some("noopener noreferrer"))
        .attr_default("aria-current", current.then_some("page"))
        .event("onclick", move |event: MouseEvent| {
            let primary = event.trigger_button() == Some(MouseButton::Primary);
            if !event.modifiers().is_empty() || !primary || new_tab || is_external {
                return;
            }
            event.prevent_default();
            router.push(to.clone());
        })
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
