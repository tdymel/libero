use std::{cell::OnceCell, rc::Rc};

use dioxus::{core::Runtime, html::input_data::MouseButton, prelude::*, router::RouterContext};

use crate::{
    components::{
        common::{
            HtmlTag, Input, StyleAttributes, Variables, allowed_href, base_props,
            use_style_attributes,
        },
        layout::{BoxStyle, box_style},
    },
    sx::StaticSx,
};

/// A link the router does not handle still keeps its base path, else it leaves the app
/// (todo 4855).
/// The router and the current route for a link, found once per component: `try_router` climbs
/// the scope tree and `full_route_string` reads `location`, and every link paid both per render.
#[derive(Clone)]
pub(crate) struct LinkRouter(Rc<OnceCell<Option<Found>>>);

/// The router, the current route, and the base path the history was built with.
type Found = (RouterContext, Memo<String>, Option<Rc<str>>);

/// The route as one memo, provided beside the router it reads and gone with it.
#[derive(Clone, Copy)]
struct CurrentRoute(Memo<String>);

/// A hook; the lookup waits for the first link the component renders.
pub(crate) fn use_link_router() -> LinkRouter {
    use_hook(|| LinkRouter(Rc::new(OnceCell::new())))
}

impl LinkRouter {
    pub(crate) fn router(&self) -> Option<RouterContext> {
        self.found().map(|(router, ..)| router)
    }

    /// Whether `matches` holds for the current route. Reactive, as `full_route_string` is.
    pub(crate) fn route_is(&self, matches: impl FnOnce(&str) -> bool) -> bool {
        self.found()
            .is_some_and(|(_, route, _)| matches(route.read().as_str()))
    }

    /// `url` under the router's base path: asked once, as `prefix()` climbs for the history.
    fn href(&self, url: &str) -> String {
        match self.found() {
            Some((.., Some(prefix))) => [&*prefix, url].concat(),
            _ => url.to_string(),
        }
    }

    fn found(&self) -> Option<Found> {
        self.0
            .get_or_init(|| {
                let router = try_router()?;
                let route = try_consume_context::<CurrentRoute>()
                    .map_or_else(|| provide_route(router), |CurrentRoute(route)| route);
                Some((router, route, router.prefix().map(Rc::from)))
            })
            .clone()
    }
}

/// Creates the route memo in the scope that provides `router`, for every later link under it.
fn provide_route(router: RouterContext) -> Memo<String> {
    let runtime = Runtime::current();
    let mut scope = runtime.current_scope_id();
    while runtime.has_context::<RouterContext>(scope).is_none() {
        match runtime.parent_scope(scope) {
            Some(parent) => scope = parent,
            None => break,
        }
    }
    let route = runtime.in_scope(scope, || Memo::new(move || router.full_route_string()));
    runtime.provide_context(scope, CurrentRoute(route));
    route
}

fn plain_href(to: NavigationTarget, link: &LinkRouter) -> String {
    match to {
        NavigationTarget::Internal(url) => link.href(&url),
        NavigationTarget::External(url) => url,
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

/// `to` as the router's `Link` when one is mounted and `target` allows it,
/// else a plain `<a>`.
#[component]
pub(crate) fn InternalAnchor(props: InternalAnchorProps) -> Element {
    let visible = crate::components::common::with_visibility(
        &props.states,
        &props.hidden_from,
        &props.visible_from,
    );
    let link = use_link_router();
    let style_attributes = use_style_attributes(
        &props.class,
        props.framework_sx,
        crate::components::common::sx_source(&props.sx),
        visible.as_ref().unwrap_or(&props.states),
        &props.variables,
        props.style,
        true,
    );

    render_anchor(
        &link,
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
    link: &LinkRouter,
    style_attributes: StyleAttributes,
    to: NavigationTarget,
    target: Option<String>,
    onmounted: Option<impl FnMut(MountedEvent) + 'static>,
    attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let is_blank = target
        .as_deref()
        .is_some_and(|target| target.eq_ignore_ascii_case("_blank"));
    let mounted = link.router();
    let router = mounted.filter(|_| target.is_none() || is_blank);

    // Without a router every string is `External`, so only an absolute URL counts as one.
    let rel = matches!(&to, NavigationTarget::External(url) if url.contains("://") || url.starts_with("//"))
        .then_some("noopener noreferrer");
    // Every library link resolves here: a blocked scheme leaves an inert `<a>`.
    let (NavigationTarget::Internal(url) | NavigationTarget::External(url)) = &to;
    let mut anchor = if allowed_href(url, format_args!("Link to `{url}`")) {
        match router {
            Some(router) => routed(box_style(style_attributes), link, router, to, is_blank),
            None => box_style(style_attributes)
                .attr_default("rel", rel)
                .attr("href", plain_href(to, link))
                .attr("target", target),
        }
    } else {
        box_style(style_attributes)
    };
    // Attached only when the caller wants it: an unused listener still costs a
    // diff every render, and on a WebView a blocking round trip (todo 2986).
    if let Some(mut onmounted) = onmounted {
        anchor = anchor.event("onmounted", move |event: MountedEvent| onmounted(event));
    }

    anchor.render(HtmlTag::A, attributes, children)
}

/// The router's `Link` by hand, as it attaches an `onmounted` to every anchor
/// (todo 2986). Its `href`, `rel`, `aria-current` and click are kept.
fn routed(
    anchor: BoxStyle,
    link: &LinkRouter,
    router: RouterContext,
    to: NavigationTarget,
    new_tab: bool,
) -> BoxStyle {
    let is_external = matches!(to, NavigationTarget::External(_));
    let href = match &to {
        NavigationTarget::Internal(url) => link.href(url),
        NavigationTarget::External(url) => url.clone(),
    };
    let current =
        matches!(&to, NavigationTarget::Internal(url) if link.route_is(|route| route == url));
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
