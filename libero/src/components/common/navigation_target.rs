use dioxus::prelude::*;

use crate::components::common::{Input, input_from_str};

input_from_str!(NavigationTarget);

// Sealed: only `#[props(into)]` names it, through the impl below.
#[doc(hidden)]
#[allow(unnameable_types)]
pub struct RouteMarker;

// A typed `Route`, as on `Anchor`. `From<R: Routable>` would overlap the impls above.
impl<R: Routable> dioxus::core::SuperFrom<R, RouteMarker> for Input<NavigationTarget> {
    fn super_from(value: R) -> Self {
        Input::Value(value.into())
    }
}
