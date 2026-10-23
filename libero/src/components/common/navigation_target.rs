use dioxus::prelude::*;

use crate::components::common::{Input, input_from_str};

input_from_str!(NavigationTarget);

impl From<Option<&str>> for Input<NavigationTarget> {
    fn from(value: Option<&str>) -> Self {
        value.map_or(Self::None, Into::into)
    }
}

impl From<Option<String>> for Input<NavigationTarget> {
    fn from(value: Option<String>) -> Self {
        value.map_or(Self::None, Into::into)
    }
}

// Sealed: only `#[props(into)]` names it, through the impls below.
#[doc(hidden)]
#[allow(unnameable_types)]
pub struct RouteMarker;

#[doc(hidden)]
#[allow(unnameable_types)]
pub struct OptionRouteMarker;

// A typed `Route`, as on `Anchor`. `From<R: Routable>` would overlap the impls above.
impl<R: Routable> dioxus::core::SuperFrom<R, RouteMarker> for Input<NavigationTarget> {
    fn super_from(value: R) -> Self {
        Input::Value(value.into())
    }
}

// `None` is no link: a plain button.
impl<R: Routable> dioxus::core::SuperFrom<Option<R>, OptionRouteMarker>
    for Input<NavigationTarget>
{
    fn super_from(value: Option<R>) -> Self {
        value.map_or(Input::None, |route| Input::Value(route.into()))
    }
}
